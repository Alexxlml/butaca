//! Cliente JSON-RPC (HTTP) de Kodi.

use crate::i18n;
use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;

pub struct Kodi {
    agent: ureq::Agent,
    url: String,
    auth: Option<String>,
}

/// Lo que se está reproduciendo ahora mismo.
pub struct NowPlaying {
    pub item: Value,
    pub props: Value,
    pub show_art: Value,
    pub show_uniqueid: Value,
}

impl Kodi {
    pub fn new(host: &str, port: u16, user: &str, password: &str) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build()
            .into();
        let auth = (!user.is_empty()).then(|| {
            let token = base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"));
            format!("Basic {token}")
        });
        Self { agent, url: format!("http://{host}:{port}/jsonrpc"), auth }
    }

    pub fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
        let mut req = self.agent.post(&self.url);
        if let Some(a) = &self.auth {
            req = req.header("Authorization", a);
        }
        let mut resp = req.send_json(&body).map_err(|e| e.to_string())?;
        let mut data: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
        if let Some(err) = data.get("error") {
            return Err(format!("{method}: {err}"));
        }
        Ok(data["result"].take())
    }

    /// Devuelve el video en reproducción, o `None` si no hay.
    pub fn now_playing(&self) -> Result<Option<NowPlaying>, String> {
        let players = self.call("Player.GetActivePlayers", json!({}))?;
        let video = players
            .as_array()
            .and_then(|a| a.iter().find(|p| p["type"] == "video"));
        let Some(pid) = video.and_then(|p| p["playerid"].as_i64()) else {
            return Ok(None);
        };
        let item = self
            .call(
                "Player.GetItem",
                json!({"playerid": pid, "properties": [
                    "title", "showtitle", "season", "episode", "year", "genre", "director",
                    "art", "uniqueid", "tvshowid", "file"
                ]}),
            )?["item"]
            .take();
        let props = self.call(
            "Player.GetProperties",
            json!({"playerid": pid, "properties": ["time", "totaltime", "speed"]}),
        )?;

        let (mut show_art, mut show_uniqueid) = (json!({}), json!({}));
        let tvshowid = item["tvshowid"].as_i64().unwrap_or(-1);
        if item["type"] == "episode" && tvshowid > 0 {
            match self.call(
                "VideoLibrary.GetTVShowDetails",
                json!({"tvshowid": tvshowid, "properties": ["uniqueid", "art"]}),
            ) {
                Ok(mut r) => {
                    let show = r["tvshowdetails"].take();
                    if show["art"].is_object() {
                        show_art = show["art"].clone();
                    }
                    if show["uniqueid"].is_object() {
                        show_uniqueid = show["uniqueid"].clone();
                    }
                }
                Err(e) => log::warn!("Could not read the TV show details: {e}"),
            }
        }
        Ok(Some(NowPlaying { item, props, show_art, show_uniqueid }))
    }
}

/// Traduce errores técnicos de la conexión a algo que el usuario pueda actuar.
pub fn friendly_error(e: &str, lang: &str) -> String {
    if e.contains("401") {
        i18n::t_in(lang, "kodi.err_auth")
    } else if e.contains("timeout") {
        i18n::t_in(lang, "kodi.err_timeout")
    } else if e.contains("404") {
        i18n::t_in(lang, "kodi.err_not_found")
    } else {
        i18n::tf_in(lang, "kodi.err_connect", &[("error", e)])
    }
}

impl Kodi {
    /// `JSONRPC.Ping` + `Application.GetProperties`; devuelve p. ej. "Kodi 21.2".
    pub fn ping(&self) -> Result<String, String> {
        self.call("JSONRPC.Ping", json!({}))?;
        let r = self.call("Application.GetProperties", json!({"properties": ["name", "version"]}))?;
        let name = r["name"].as_str().unwrap_or("Kodi");
        let (major, minor) = (r["version"]["major"].as_i64(), r["version"]["minor"].as_i64());
        Ok(match (major, minor) {
            (Some(a), Some(b)) => format!("{name} {a}.{b}"),
            _ => name.to_string(),
        })
    }
}
