//! Hilo de trabajo: consulta a Kodi y publica en Discord. Se comunica por canales.

use crate::config::Config;
use crate::discord_ipc::DiscordIpc;
use crate::kodi::Kodi;
use crate::kodi_ws::KodiWs;
use crate::presence;
use crate::tmdb::Tmdb;
use serde_json::Value;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    NotConfigured,
    ConfigError(String),
    Paused,
    DiscordClosed,
    WaitingKodi,
    KodiIdle,
    Watching(String),
}

pub enum Cmd {
    SetPaused(bool),
    /// Nueva configuración: se aplica sin cerrar la conexión con Discord
    /// (Discord tarda en aceptar una conexión que se reabre al instante).
    Config(Box<Config>),
    /// Kodi avisó por WebSocket de un cambio en la reproducción: consultar ya.
    Wake,
    Quit,
}

pub struct Worker {
    tx: Sender<Cmd>,
    handle: JoinHandle<()>,
}

impl Worker {
    pub fn spawn(cfg: Config, report: impl Fn(Status) + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        let wake = tx.clone();
        let handle = std::thread::spawn(move || run(cfg, rx, wake, report));
        Self { tx, handle }
    }

    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }

    pub fn reconfigure(&self, cfg: Config) {
        self.send(Cmd::Config(Box::new(cfg)));
    }

    /// Pide cerrar y espera a que se limpie la actividad.
    pub fn stop(self) {
        let _ = self.tx.send(Cmd::Quit);
        let _ = self.handle.join();
    }
}

/// Espera `d` atendiendo comandos. Devuelve `true` si hay que salir.
/// Una configuración nueva se deja en `pending` y la espera se corta para aplicarla ya.
fn wait(rx: &Receiver<Cmd>, d: Duration, paused: &mut bool, pending: &mut Option<Config>) -> bool {
    match rx.recv_timeout(d) {
        Ok(Cmd::Quit) | Err(RecvTimeoutError::Disconnected) => true,
        Ok(Cmd::SetPaused(p)) => {
            *paused = p;
            false
        }
        Ok(Cmd::Config(c)) => {
            *pending = Some(*c);
            false
        }
        Ok(Cmd::Wake) => {
            // Debounce: Kodi suele mandar varios eventos seguidos y el item tarda un instante en estar listo.
            loop {
                match rx.recv_timeout(Duration::from_millis(400)) {
                    Ok(Cmd::Wake) => {}
                    Ok(Cmd::SetPaused(p)) => *paused = p,
                    Ok(Cmd::Config(c)) => *pending = Some(*c),
                    Ok(Cmd::Quit) | Err(RecvTimeoutError::Disconnected) => return true,
                    Err(RecvTimeoutError::Timeout) => return false,
                }
            }
        }
        Err(RecvTimeoutError::Timeout) => false,
    }
}

/// Como `wait`, pero ignora los avisos del WebSocket: con Discord cerrado no hay nada que adelantar.
fn wait_full(rx: &Receiver<Cmd>, d: Duration, paused: &mut bool, pending: &mut Option<Config>) -> bool {
    let end = Instant::now() + d;
    loop {
        let Some(left) = end.checked_duration_since(Instant::now()).filter(|l| !l.is_zero()) else {
            return false;
        };
        match rx.recv_timeout(left) {
            Ok(Cmd::Wake) => {}
            Ok(Cmd::SetPaused(p)) => {
                *paused = p;
                return false;
            }
            Ok(Cmd::Config(c)) => {
                *pending = Some(*c);
                return false;
            }
            Ok(Cmd::Quit) | Err(RecvTimeoutError::Disconnected) => return true,
            Err(RecvTimeoutError::Timeout) => return false,
        }
    }
}

fn start_ws(cfg: &Config, wake: &Sender<Cmd>) -> Option<KodiWs> {
    (cfg.use_websocket && cfg.websocket_port != 0)
        .then(|| KodiWs::spawn(cfg.kodi_host.trim().to_string(), cfg.websocket_port, wake.clone()))
}

/// Con el WebSocket activo el polling solo es respaldo.
const BACKUP_POLL: Duration = Duration::from_secs(15);

fn run(mut cfg: Config, rx: Receiver<Cmd>, wake: Sender<Cmd>, report: impl Fn(Status)) {
    let mut kodi = Kodi::new(cfg.kodi_host.trim(), cfg.kodi_port, &cfg.kodi_user, &cfg.kodi_password);
    let mut tmdb = Tmdb::new(&cfg.tmdb_api_key);
    let mut discord = DiscordIpc::new(&cfg.discord_client_id);
    let mut connected = false;
    let mut discord_down_logged = false;
    let mut use_name = true;
    // `None` = aún no se ha enviado nada desde la última conexión.
    let mut last: Option<Option<Value>> = None;
    let mut kodi_ok: Option<bool> = None;
    let mut paused = false;
    let mut poll = Duration::from_secs(cfg.poll_seconds.max(1));
    let mut lang = crate::i18n::normalize(&cfg.language);
    let mut pending: Option<Config> = None;
    let mut ws = start_ws(&cfg, &wake);
    let retry = Duration::from_secs(10);

    let mut shown: Option<Status> = None;
    let mut set_status = |s: Status| {
        if shown.as_ref() != Some(&s) {
            report(s.clone());
            shown = Some(s);
        }
    };

    log::info!("Butaca: Kodi {}:{} -> Discord (language: {lang})", cfg.kodi_host.trim(), cfg.kodi_port);
    loop {
        if let Some(new) = pending.take() {
            if new.discord_client_id != cfg.discord_client_id {
                discord.close();
                discord = DiscordIpc::new(&new.discord_client_id);
                connected = false;
            }
            kodi = Kodi::new(new.kodi_host.trim(), new.kodi_port, &new.kodi_user, &new.kodi_password);
            tmdb = Tmdb::new(&new.tmdb_api_key);
            poll = Duration::from_secs(new.poll_seconds.max(1));
            lang = crate::i18n::normalize(&new.language);
            ws = start_ws(&new, &wake);
            kodi_ok = None;
            last = None; // se vuelve a publicar con el idioma y los datos nuevos
            log::info!("Settings applied: Kodi {}:{} (language: {lang})", new.kodi_host.trim(), new.kodi_port);
            cfg = new;
        }

        if paused {
            if connected && last != Some(None) {
                match discord.set_activity(None) {
                    Ok(_) => last = Some(None),
                    Err(e) => {
                        log::warn!("No connection to Discord: {e}");
                        discord.close();
                        connected = false;
                    }
                }
            }
            set_status(Status::Paused);
            if wait(&rx, Duration::from_secs(3600), &mut paused, &mut pending) {
                break;
            }
            if !paused {
                log::info!("Presence resumed");
                last = None;
            }
            continue;
        }

        if !connected {
            match discord.connect() {
                Ok(user) => {
                    log::info!("Connected to Discord as {user}");
                    connected = true;
                    discord_down_logged = false;
                    last = None;
                }
                Err(e) => {
                    if !discord_down_logged {
                        log::warn!("No connection to Discord: {e}. Retrying every 10 s");
                        discord_down_logged = true;
                    }
                    set_status(Status::DiscordClosed);
                    if wait_full(&rx, retry, &mut paused, &mut pending) {
                        break;
                    }
                    continue;
                }
            }
        }

        let np = match kodi.now_playing() {
            Ok(np) => {
                if kodi_ok != Some(true) {
                    log::info!("Kodi reachable");
                }
                kodi_ok = Some(true);
                np
            }
            Err(e) => {
                if kodi_ok != Some(false) {
                    log::warn!("Kodi is not responding ({e}); clearing the status");
                }
                kodi_ok = Some(false);
                None
            }
        };

        let show_idle = np.is_none() && kodi_ok == Some(true) && cfg.show_idle_status;
        let activity = match &np {
            Some(np) => Some(presence::build_activity(np, &mut tmdb, use_name, lang)),
            None if show_idle => Some(presence::idle_activity(use_name, lang)),
            None => None,
        };
        let unchanged = last
            .as_ref()
            .is_some_and(|l| presence::same_activity(activity.as_ref(), l.as_ref()));

        if !unchanged {
            let sent = discord.set_activity(activity.as_ref()).and_then(|ok| {
                if !ok && activity.as_ref().is_some_and(|a| a.get("name").is_some()) {
                    use_name = false;
                    log::warn!("Discord does not accept 'name' over IPC; using status_display_type only");
                    let fallback = match &np {
                        Some(np) => presence::build_activity(np, &mut tmdb, use_name, lang),
                        None => presence::idle_activity(use_name, lang),
                    };
                    discord.set_activity(Some(&fallback)).map(|_| Some(fallback))
                } else {
                    Ok(activity.clone())
                }
            });
            match sent {
                Ok(final_activity) => {
                    let shown = final_activity.as_ref().map(|a| {
                        format!("{} | {}", a["details"].as_str().unwrap_or(""), a["state"].as_str().unwrap_or(""))
                    });
                    log::info!("Activity: {}", shown.as_deref().unwrap_or("(empty)"));
                    last = Some(final_activity);
                }
                Err(e) => {
                    log::warn!("No connection to Discord: {e}. Retrying every 10 s");
                    discord_down_logged = true;
                    discord.close();
                    connected = false;
                    set_status(Status::DiscordClosed);
                    if wait_full(&rx, retry, &mut paused, &mut pending) {
                        break;
                    }
                    continue;
                }
            }
        }

        set_status(match (&np, &activity) {
            (Some(_), Some(a)) => Status::Watching(
                a["name"].as_str().or(a["details"].as_str()).unwrap_or("").to_string(),
            ),
            _ if kodi_ok == Some(true) => Status::KodiIdle,
            _ => Status::WaitingKodi,
        });

        let next = match &ws {
            Some(w) if w.is_connected() => poll.max(BACKUP_POLL),
            _ => poll,
        };
        if wait(&rx, next, &mut paused, &mut pending) {
            break;
        }
    }

    if connected {
        let _ = discord.set_activity(None);
    }
    discord.close();
    log::info!("Butaca closed");
}
