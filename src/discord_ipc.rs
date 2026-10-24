//! Cliente del IPC local de Discord (named pipe `\\?\pipe\discord-ipc-N`).
//! Protocolo basado en service.discord.richpresence de Hiumee (MIT).

use serde_json::{json, Value};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;

pub struct DiscordIpc {
    client_id: String,
    pipe: Option<File>,
}

fn conn_err(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::ConnectionAborted, msg.into())
}

impl DiscordIpc {
    pub fn new(client_id: &str) -> Self {
        Self { client_id: client_id.to_string(), pipe: None }
    }

    /// Abre la primera pipe disponible y hace el handshake. Devuelve el nombre de usuario.
    pub fn connect(&mut self) -> io::Result<String> {
        let pipe = (0..10)
            .find_map(|i| {
                OpenOptions::new().read(true).write(true).open(format!(r"\\?\pipe\discord-ipc-{i}")).ok()
            })
            .ok_or_else(|| conn_err("Discord pipe not found (is Discord open?)"))?;
        self.pipe = Some(pipe);

        let hello = json!({"v": 1, "client_id": self.client_id});
        let result = self.send(&hello, OP_HANDSHAKE).and_then(|_| self.recv());
        let (op, data) = match result {
            Ok(r) => r,
            Err(e) => {
                self.close();
                return Err(e);
            }
        };
        if op != OP_FRAME || data["evt"] != "READY" {
            self.close();
            return Err(conn_err(format!("Handshake rejected: {data}")));
        }
        Ok(data["data"]["user"]["username"].as_str().unwrap_or("?").to_string())
    }

    fn send(&mut self, payload: &Value, op: u32) -> io::Result<()> {
        let body = serde_json::to_vec(payload).map_err(io::Error::other)?;
        let mut buf = Vec::with_capacity(8 + body.len());
        buf.extend_from_slice(&op.to_le_bytes());
        buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
        buf.extend_from_slice(&body);
        let pipe = self.pipe.as_mut().ok_or_else(|| conn_err("Not connected"))?;
        pipe.write_all(&buf)?;
        pipe.flush()
    }

    fn recv(&mut self) -> io::Result<(u32, Value)> {
        let pipe = self.pipe.as_mut().ok_or_else(|| conn_err("Not connected"))?;
        let mut head = [0u8; 8];
        pipe.read_exact(&mut head)?;
        let op = u32::from_le_bytes(head[..4].try_into().unwrap());
        let len = u32::from_le_bytes(head[4..].try_into().unwrap()) as usize;
        let mut body = vec![0u8; len];
        pipe.read_exact(&mut body)?;
        let data = serde_json::from_slice(&body).map_err(io::Error::other)?;
        Ok((op, data))
    }

    /// Envía un frame y lee respuestas hasta encontrar la de su nonce.
    fn send_recv(&mut self, payload: &Value) -> io::Result<Value> {
        self.send(payload, OP_FRAME)?;
        let nonce = payload["nonce"].clone();
        loop {
            let (op, data) = self.recv()?;
            if op == OP_CLOSE {
                return Err(conn_err(format!("Discord closed the connection: {data}")));
            }
            if data["nonce"] == nonce {
                return Ok(data);
            }
        }
    }

    /// `Ok(false)` si Discord rechazó la actividad. `None` limpia el estado.
    pub fn set_activity(&mut self, activity: Option<&Value>) -> io::Result<bool> {
        let mut args = json!({"pid": std::process::id()});
        if let Some(a) = activity {
            args["activity"] = a.clone();
        }
        let resp = self.send_recv(&json!({
            "cmd": "SET_ACTIVITY",
            "args": args,
            "nonce": uuid::Uuid::new_v4().to_string(),
        }))?;
        if resp["evt"] == "ERROR" {
            log::warn!("Discord rejected the activity: {}", resp["data"]);
            return Ok(false);
        }
        Ok(true)
    }

    pub fn close(&mut self) {
        self.pipe = None;
    }
}
