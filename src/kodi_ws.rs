//! Escucha las notificaciones JSON-RPC de Kodi por WebSocket (puerto 9090).
//! Solo sirve de aviso: cada evento de reproducción despierta al hilo de trabajo para que consulte
//! a Kodi por HTTP al instante. El polling sigue como respaldo.

use crate::worker::Cmd;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use tungstenite::{Error, Message};

const EVENTS: [&str; 6] = [
    "Player.OnPlay",
    "Player.OnPause",
    "Player.OnResume",
    "Player.OnStop",
    "Player.OnSeek",
    "Player.OnSpeedChanged",
];
const RETRY: Duration = Duration::from_secs(10);
const READ_TICK: Duration = Duration::from_millis(500);

pub struct KodiWs {
    stop: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl KodiWs {
    pub fn spawn(host: String, port: u16, wake: Sender<Cmd>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let connected = Arc::new(AtomicBool::new(false));
        let (s, c) = (stop.clone(), connected.clone());
        let handle = std::thread::spawn(move || run(&host, port, &wake, &s, &c));
        Self { stop, connected, handle: Some(handle) }
    }

    /// `true` mientras hay un WebSocket abierto con Kodi.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}

impl Drop for KodiWs {
    fn drop(&mut self) {
        // No se espera al hilo: termina solo en cuanto vea la bandera (≤ 500 ms o el fin del reintento).
        self.stop.store(true, Ordering::Relaxed);
        drop(self.handle.take());
    }
}

fn run(host: &str, port: u16, wake: &Sender<Cmd>, stop: &AtomicBool, connected: &AtomicBool) {
    let mut logged_down = false;
    while !stop.load(Ordering::Relaxed) {
        match session(host, port, wake, stop, connected) {
            Ok(()) => {}
            Err(e) => {
                if !logged_down {
                    log::info!("Kodi WebSocket unavailable ({e}); using polling only");
                    logged_down = true;
                }
            }
        }
        if connected.swap(false, Ordering::Relaxed) {
            log::info!("Kodi WebSocket closed; back to polling");
            logged_down = true;
        }
        // Reintento en tramos cortos para reaccionar a `stop`.
        for _ in 0..(RETRY.as_millis() / READ_TICK.as_millis()) {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            std::thread::sleep(READ_TICK);
        }
    }
}

/// Comprueba si Kodi acepta el WebSocket (para "Probar conexión").
pub fn probe(host: &str, port: u16) -> Result<(), String> {
    let addr = (host, port).to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("no address")?;
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;
    tungstenite::client(format!("ws://{host}:{port}/jsonrpc").as_str(), stream)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn session(host: &str, port: u16, wake: &Sender<Cmd>, stop: &AtomicBool, connected: &AtomicBool) -> Result<(), String> {
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("no address")?;
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
    let url = format!("ws://{host}:{port}/jsonrpc");
    let (mut ws, _) = tungstenite::client(url.as_str(), stream).map_err(|e| e.to_string())?;
    ws.get_ref().set_read_timeout(Some(READ_TICK)).map_err(|e| e.to_string())?;

    if !connected.swap(true, Ordering::Relaxed) {
        log::info!("Kodi WebSocket connected ({host}:{port})");
    }
    // Avisa de inmediato: el estado pudo cambiar mientras no había WebSocket.
    let _ = wake.send(Cmd::Wake);

    while !stop.load(Ordering::Relaxed) {
        match ws.read() {
            Ok(Message::Text(t)) => {
                if is_player_event(&t) && wake.send(Cmd::Wake).is_err() {
                    return Ok(()); // el worker ya no existe
                }
            }
            Ok(Message::Close(_)) => return Ok(()),
            Ok(_) => {}
            Err(Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(Error::ConnectionClosed | Error::AlreadyClosed) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        }
    }
    let _ = ws.close(None);
    Ok(())
}

fn is_player_event(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v["method"].as_str().map(|m| EVENTS.contains(&m)))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::is_player_event;

    #[test]
    fn recognizes_player_notifications() {
        assert!(is_player_event(r#"{"jsonrpc":"2.0","method":"Player.OnPause","params":{}}"#));
        assert!(is_player_event(r#"{"jsonrpc":"2.0","method":"Player.OnSeek","params":{}}"#));
        assert!(!is_player_event(r#"{"jsonrpc":"2.0","method":"Application.OnVolumeChanged","params":{}}"#));
        assert!(!is_player_event("not json"));
    }
}
