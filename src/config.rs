//! Configuración en `%APPDATA%\Butaca\config.json`.
//! La contraseña de Kodi NO se guarda en el archivo: vive en el Administrador de credenciales de Windows.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Client ID público de la app de Discord del proyecto ("TV").
pub const DEFAULT_CLIENT_ID: &str = "1563628762875302008";

const KEYRING_SERVICE: &str = "Butaca";
const KEYRING_USER: &str = "kodi-password";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub kodi_host: String,
    pub kodi_port: u16,
    pub kodi_user: String,
    /// Solo en memoria. Si un `config.json` antiguo la trae, se migra al llavero y se borra del archivo.
    #[serde(skip_serializing)]
    pub kodi_password: String,
    /// Opcional; vacía por defecto. Cada usuario pone la suya.
    pub tmdb_api_key: String,
    /// Opción avanzada.
    pub discord_client_id: String,
    pub show_idle_status: bool,
    /// Idioma de la interfaz y del texto de la presencia ("en", "es", "fr", "de", "it").
    pub language: String,
    pub poll_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kodi_host: String::new(),
            kodi_port: 8080,
            kodi_user: "kodi".into(),
            kodi_password: String::new(),
            tmdb_api_key: String::new(),
            discord_client_id: DEFAULT_CLIENT_ID.into(),
            show_idle_status: false,
            language: crate::i18n::DEFAULT.into(),
            poll_seconds: 3,
        }
    }
}

/// Qué impide arrancar el trabajo.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Problem {
    NoHost,
    BadClientId,
}

impl Config {
    pub fn problem(&self) -> Option<Problem> {
        if self.kodi_host.trim().is_empty() {
            Some(Problem::NoHost)
        } else if self.discord_client_id.is_empty() || !self.discord_client_id.chars().all(|c| c.is_ascii_digit()) {
            Some(Problem::BadClientId)
        } else {
            None
        }
    }
}

pub fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    base.join("Butaca")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())
}

fn read_password() -> String {
    match entry().and_then(|e| e.get_password().map_err(|e| e.to_string())) {
        Ok(p) => p,
        Err(e) => {
            // "No matching entry" es lo normal si nunca se guardó una contraseña.
            log::debug!("No password in the credential store: {e}");
            String::new()
        }
    }
}

fn write_password(password: &str) -> Result<(), String> {
    let entry = entry()?;
    if password.is_empty() {
        return match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    entry.set_password(password).map_err(|e| e.to_string())
}

pub enum Loaded {
    Ready(Config),
    /// Se creó un `config.json` de ejemplo; el usuario debe configurarlo.
    Created,
}

pub fn load() -> Result<Loaded, String> {
    let path = config_path();
    if !path.exists() {
        write_file(&Config::default())?;
        return Ok(Loaded::Created);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    let mut cfg: Config = serde_json::from_str(&text).map_err(|e| format!("{} is not valid: {e}", path.display()))?;

    if cfg.kodi_password.is_empty() {
        cfg.kodi_password = read_password();
    } else {
        // Migración desde versiones que guardaban la contraseña en el archivo.
        match write_password(&cfg.kodi_password).and_then(|()| save(&cfg)) {
            Ok(()) => log::info!("The Kodi password was moved to the Windows Credential Manager"),
            Err(e) => log::warn!("Could not move the password to the credential store ({e}); it stays in config.json"),
        }
    }
    Ok(Loaded::Ready(cfg))
}

/// Guarda `config.json` (sin contraseña) y la contraseña en el llavero.
pub fn save(cfg: &Config) -> Result<(), String> {
    write_password(&cfg.kodi_password).map_err(|e| format!("Could not save the password: {e}"))?;
    write_file(cfg)
}

fn write_file(cfg: &Config) -> Result<(), String> {
    std::fs::create_dir_all(config_dir()).map_err(|e| format!("Could not create {}: {e}", config_dir().display()))?;
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    let path = config_path();
    std::fs::write(&path, json).map_err(|e| format!("Could not write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    /// Comprueba que el llavero de Windows guarda, lee y borra (usa un servicio de prueba aparte).
    #[test]
    fn keyring_roundtrip() {
        let e = keyring::Entry::new("Butaca-test", "roundtrip").unwrap();
        e.set_password("secreto-de-prueba").unwrap();
        assert_eq!(e.get_password().unwrap(), "secreto-de-prueba");
        e.delete_credential().unwrap();
        assert!(matches!(e.get_password(), Err(keyring::Error::NoEntry)));
    }
}
