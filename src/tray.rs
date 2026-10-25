//! Ícono en la bandeja del sistema y event loop de la app.

use crate::autostart;
use crate::config::{self, Config, Loaded, Problem};
use crate::i18n;
use crate::presence::clip;
use crate::settings;
use crate::single_instance::Instance;
use crate::worker::{Cmd, Status, Worker};
use std::sync::Arc;
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, TrayIconBuilder, TrayIconEvent};

enum UserEvent {
    Menu(MenuEvent),
    Tray(TrayIconEvent),
    Status(Status),
    /// Se guardó una configuración nueva desde la ventana.
    ConfigSaved(Config),
    /// Otra instancia de Butaca pidió abrir la configuración.
    OpenSettings,
}

const APP: &str = "Butaca";

/// Aviso legal del cuadro "Acerca de": se deja en inglés, tal cual.
const LEGAL: &str = "Kodi® is a registered trademark of the XBMC Foundation. Discord is a trademark of Discord Inc. \
    This project is not affiliated with, endorsed by, or sponsored by either.";

fn status_text(s: &Status) -> String {
    match s {
        Status::NotConfigured => i18n::t("status.not_configured"),
        Status::ConfigError(e) => i18n::tf("status.config_error", &[("error", e)]),
        Status::Paused => i18n::t("status.paused"),
        Status::DiscordClosed => i18n::t("status.discord_closed"),
        Status::WaitingKodi => i18n::t("status.waiting_kodi"),
        Status::KodiIdle => i18n::t("status.kodi_idle"),
        Status::Watching(t) => i18n::tf("status.watching", &[("title", t)]),
    }
}

fn tooltip(s: &Status) -> String {
    clip(&i18n::tf("tray.tooltip", &[("status", &status_text(s))]), 120)
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Cuadro de diálogo en un hilo aparte para no bloquear el event loop.
fn message_box(title: &str, body: String) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
    let title = wide(title);
    std::thread::spawn(move || {
        let b = wide(&body);
        unsafe { MessageBoxW(std::ptr::null_mut(), b.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION) };
    });
}

const RELEASES_URL: &str = "https://github.com/Alexxlml/butaca/releases/latest";

fn open_releases() {
    let _ = std::process::Command::new("explorer").arg(RELEASES_URL).spawn();
}

fn open_logs() {
    let _ = std::process::Command::new("explorer").arg(config::config_dir()).spawn();
}

fn status_for(cfg: &Config) -> Status {
    match cfg.problem() {
        None => Status::WaitingKodi,
        Some(Problem::NoHost) => Status::NotConfigured,
        Some(Problem::BadClientId) => Status::ConfigError(i18n::t("status.invalid_client_id")),
    }
}

pub fn run(instance: Instance) {
    autostart::refresh_path();
    let (current, first_run) = match config::load() {
        Ok(Loaded::Ready(c)) => (c, false),
        Ok(Loaded::Created) => (Config::default(), true),
        Err(e) => {
            log::error!("{e}");
            (Config::default(), false)
        }
    };
    i18n::set(&current.language);
    let status = status_for(&current);
    let first_run = first_run || status == Status::NotConfigured;
    let mut cfg = current.problem().is_none().then(|| current.clone());
    let mut current = current;

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    {
        let p = proxy.clone();
        MenuEvent::set_event_handler(Some(move |e| {
            let _ = p.send_event(UserEvent::Menu(e));
        }));
        let p = proxy.clone();
        TrayIconEvent::set_event_handler(Some(move |e| {
            let _ = p.send_event(UserEvent::Tray(e));
        }));
    }

    let open_settings = {
        let p = proxy.clone();
        move |cfg: Config| {
            let p = p.clone();
            settings::open(
                cfg,
                Arc::new(move |c| {
                    let _ = p.send_event(UserEvent::ConfigSaved(c));
                }),
            );
        }
    };

    {
        let p = proxy.clone();
        instance.on_open_request(move || {
            let _ = p.send_event(UserEvent::OpenSettings);
        });
    }

    let status_item = MenuItem::new(status_text(&status), false, None);
    let settings_item = MenuItem::new(i18n::t("tray.settings"), true, None);
    let pause_item = MenuItem::new(i18n::t("tray.pause"), cfg.is_some(), None);
    let logs_item = MenuItem::new(i18n::t("tray.open_logs"), true, None);
    let updates_item = MenuItem::new(i18n::t("tray.updates"), true, None);
    let about_item = MenuItem::new(i18n::t("tray.about"), true, None);
    let quit_item = MenuItem::new(i18n::t("tray.quit"), true, None);
    let menu = Menu::new();
    let _ = menu.append_items(&[
        &status_item,
        &PredefinedMenuItem::separator(),
        &settings_item,
        &pause_item,
        &logs_item,
        &updates_item,
        &about_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ]);

    // El ícono va embebido en el exe (ver build.rs); el recurso 1 es el .ico.
    let icon = Icon::from_resource(1, Some((32, 32))).expect("could not load the embedded icon");
    let mut tray = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip(tooltip(&status))
            .with_icon(icon)
            .build()
            .expect("could not create the tray icon"),
    );

    let spawn_worker = {
        let p = proxy.clone();
        move |c: Config| {
            let p = p.clone();
            Worker::spawn(c, move |s| {
                let _ = p.send_event(UserEvent::Status(s));
            })
        }
    };
    let mut worker = cfg.take().map(&spawn_worker);
    let mut paused = false;
    // Último estado informado por el hilo de trabajo (para repintar los textos al cambiar de idioma).
    let mut last_status = status.clone();

    // Primera ejecución: sin host configurado, se abre la configuración sola.
    if first_run {
        open_settings(current.clone());
    }

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        let Event::UserEvent(ev) = event else { return };

        // Vuelve a escribir todos los textos de la bandeja (tras un cambio de estado o de idioma).
        let refresh = |status: &Status, paused: bool, tray: &Option<tray_icon::TrayIcon>| {
            status_item.set_text(status_text(status));
            settings_item.set_text(i18n::t("tray.settings"));
            pause_item.set_text(i18n::t(if paused { "tray.resume" } else { "tray.pause" }));
            logs_item.set_text(i18n::t("tray.open_logs"));
            updates_item.set_text(i18n::t("tray.updates"));
            about_item.set_text(i18n::t("tray.about"));
            quit_item.set_text(i18n::t("tray.quit"));
            if let Some(tray) = tray {
                let _ = tray.set_tooltip(Some(tooltip(status)));
            }
        };

        match ev {
            UserEvent::Status(s) => {
                refresh(&s, paused, &tray);
                last_status = s;
            }
            UserEvent::OpenSettings => open_settings(current.clone()),
            UserEvent::ConfigSaved(c) => {
                i18n::set(&c.language);
                paused = false;
                current = c.clone();
                if c.problem().is_none() {
                    pause_item.set_enabled(true);
                    match &worker {
                        // Con el hilo ya en marcha se conserva la conexión con Discord.
                        Some(w) => {
                            w.send(Cmd::SetPaused(false));
                            w.reconfigure(c);
                        }
                        None => {
                            last_status = status_for(&c);
                            worker = Some(spawn_worker(c));
                        }
                    }
                } else {
                    if let Some(w) = worker.take() {
                        w.stop();
                    }
                    pause_item.set_enabled(false);
                    last_status = status_for(&c);
                }
                refresh(&last_status, paused, &tray);
            }
            UserEvent::Tray(TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }) => {
                open_settings(current.clone())
            }
            UserEvent::Tray(_) => {}
            UserEvent::Menu(e) => {
                if e.id == *settings_item.id() {
                    open_settings(current.clone());
                } else if e.id == *pause_item.id() {
                    if let Some(w) = &worker {
                        paused = !paused;
                        pause_item.set_text(i18n::t(if paused { "tray.resume" } else { "tray.pause" }));
                        w.send(Cmd::SetPaused(paused));
                    }
                } else if e.id == *logs_item.id() {
                    open_logs();
                } else if e.id == *updates_item.id() {
                    open_releases();
                } else if e.id == *about_item.id() {
                    message_box(APP, format!(
                            "{}\n{}\n\n{LEGAL}",
                            i18n::t("tray.about_body"),
                            i18n::tf("tray.about_version", &[("version", env!("CARGO_PKG_VERSION"))])
                        ));
                } else if e.id == *quit_item.id() {
                    // Limpia la actividad antes de salir y quita el ícono de la bandeja.
                    if let Some(w) = worker.take() {
                        w.stop();
                    }
                    tray.take();
                    *flow = ControlFlow::Exit;
                }
            }
        }
    });
}
