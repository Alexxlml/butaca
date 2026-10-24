//! "Iniciar con Windows": valor `Butaca` en `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "Butaca";

fn current_command() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(format!("\"{}\"", exe.display()))
}

pub fn is_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_READ)
        .and_then(|k| k.get_value::<String, _>(VALUE))
        .is_ok()
}

pub fn set(enabled: bool) -> Result<(), String> {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        .map_err(|e| e.to_string())?;
    if enabled {
        let cmd = current_command().ok_or("Could not get the program path")?;
        key.set_value(VALUE, &cmd).map_err(|e| e.to_string())
    } else {
        match key.delete_value(VALUE) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Si el exe se movió, actualiza la ruta registrada (solo si ya estaba activado).
pub fn refresh_path() {
    let Some(cmd) = current_command() else { return };
    let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_SET_VALUE) else {
        return;
    };
    if let Ok(old) = key.get_value::<String, _>(VALUE) {
        if old != cmd && key.set_value(VALUE, &cmd).is_ok() {
            log::info!("Updated the path of the start-with-Windows entry");
        }
    }
}
