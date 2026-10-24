//! Textos traducidos. Los archivos `locales/*.json` van embebidos en el exe (sigue siendo un solo archivo).
//! Para añadir un idioma: crea `locales/xx.json` con las mismas claves que `en.json` y regístralo en `LANGUAGES`.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub const DEFAULT: &str = "en";

/// (código, nombre en su propio idioma, contenido del archivo)
const LANGUAGES: &[(&str, &str, &str)] = &[
    ("en", "English", include_str!("../locales/en.json")),
    ("es", "Español", include_str!("../locales/es.json")),
    ("fr", "Français", include_str!("../locales/fr.json")),
    ("de", "Deutsch", include_str!("../locales/de.json")),
    ("it", "Italiano", include_str!("../locales/it.json")),
];

type Catalog = HashMap<String, String>;

fn catalogs() -> &'static HashMap<&'static str, Catalog> {
    static CATALOGS: OnceLock<HashMap<&'static str, Catalog>> = OnceLock::new();
    CATALOGS.get_or_init(|| {
        LANGUAGES
            .iter()
            .map(|(code, _, json)| (*code, serde_json::from_str(json).expect("locale JSON inválido")))
            .collect()
    })
}

fn current_lock() -> &'static RwLock<&'static str> {
    static CURRENT: OnceLock<RwLock<&'static str>> = OnceLock::new();
    CURRENT.get_or_init(|| RwLock::new(DEFAULT))
}

/// Idiomas disponibles: (código, nombre nativo).
pub fn languages() -> impl Iterator<Item = (&'static str, &'static str)> {
    LANGUAGES.iter().map(|(c, n, _)| (*c, *n))
}

/// Normaliza un código (acepta "es-MX", "FR"…); si no existe, inglés.
pub fn normalize(code: &str) -> &'static str {
    let base = code.trim().split(['-', '_']).next().unwrap_or("").to_ascii_lowercase();
    LANGUAGES.iter().map(|(c, _, _)| *c).find(|c| *c == base).unwrap_or(DEFAULT)
}

/// Idioma de la interfaz (bandeja, ventana, mensajes de error).
pub fn set(code: &str) {
    if let Ok(mut c) = current_lock().write() {
        *c = normalize(code);
    }
}

pub fn current() -> &'static str {
    current_lock().read().map(|c| *c).unwrap_or(DEFAULT)
}

/// Texto de `key` en el idioma `code`; si falta, el inglés; si tampoco, la propia clave.
pub fn t_in(code: &str, key: &str) -> String {
    let c = catalogs();
    c.get(normalize(code))
        .and_then(|m| m.get(key))
        .or_else(|| c.get(DEFAULT).and_then(|m| m.get(key)))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Como `t_in`, sustituyendo `{nombre}` por su valor.
pub fn tf_in(code: &str, key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t_in(code, key);
    for (name, value) in args {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

pub fn t(key: &str) -> String {
    t_in(current(), key)
}

pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    tf_in(current(), key, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(s: &str) -> Vec<String> {
        let mut v: Vec<String> = s
            .split('{')
            .skip(1)
            .filter_map(|p| p.split_once('}').map(|(n, _)| n.to_string()))
            .collect();
        v.sort();
        v
    }

    #[test]
    fn every_language_has_the_same_keys_and_placeholders() {
        let en = &catalogs()[DEFAULT];
        for (code, _, _) in LANGUAGES {
            let cat = &catalogs()[code];
            for (key, text) in en {
                let other = cat.get(key).unwrap_or_else(|| panic!("{code}: falta la clave {key}"));
                assert_eq!(placeholders(text), placeholders(other), "{code}: marcadores distintos en {key}");
            }
            for key in cat.keys() {
                assert!(en.contains_key(key), "{code}: clave sobrante {key}");
            }
        }
    }

    #[test]
    fn falls_back_to_english_and_normalizes() {
        assert_eq!(normalize("es-MX"), "es");
        assert_eq!(normalize("ZH"), "en");
        assert_eq!(t_in("xx", "tray.quit"), "Exit");
        assert_eq!(t_in("it", "tray.quit"), "Esci");
        assert_eq!(tf_in("de", "settings.test_ok", &[("version", "Kodi 22.0")]), "Verbindung erfolgreich: Kodi 22.0");
        assert_eq!(t_in("en", "no.existe"), "no.existe");
    }
}
