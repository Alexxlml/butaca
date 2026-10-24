//! Construcción de la actividad de Discord a partir de lo que reproduce Kodi.

use crate::i18n;
use crate::kodi::NowPlaying;
use crate::tmdb::Tmdb;
use percent_encoding::percent_decode_str;
use serde_json::{json, Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// Asset subido a la app de Discord, para cuando no hay póster público.
const FALLBACK_IMAGE: &str = "butaca";

/// `image://https%3a%2f%2f...jpg/`  ->  `https://...jpg`
pub fn decode_kodi_art(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    Some(match url.strip_prefix("image://") {
        Some(rest) => percent_decode_str(rest.trim_end_matches('/')).decode_utf8_lossy().into_owned(),
        None => url.to_string(),
    })
}

fn text(v: &Value) -> Option<&str> {
    v.as_str().filter(|s| !s.is_empty())
}

fn pick_poster(np: &NowPlaying, tmdb: &mut Tmdb) -> Option<String> {
    let is_ep = np.item["type"] == "episode";
    let art = &np.item["art"];
    let candidates: Vec<&Value> = if is_ep {
        vec![&art["tvshow.poster"], &np.show_art["poster"], &art["season.poster"]]
    } else {
        vec![&art["poster"], &art["thumb"]]
    };
    for c in candidates {
        if let Some(u) = text(c).and_then(decode_kodi_art) {
            if (u.starts_with("http://") || u.starts_with("https://")) && u.chars().count() <= 256 {
                return Some(u);
            }
        }
    }
    let (ids, kind) = if is_ep { (&np.show_uniqueid, "tv") } else { (&np.item["uniqueid"], "movie") };
    text(&ids["tmdb"]).and_then(|id| tmdb.poster(kind, id))
}

fn to_seconds(t: &Value) -> i64 {
    t["hours"].as_i64().unwrap_or(0) * 3600 + t["minutes"].as_i64().unwrap_or(0) * 60 + t["seconds"].as_i64().unwrap_or(0)
}

pub fn fmt_time(s: i64) -> String {
    let (h, rem) = (s / 3600, s % 3600);
    let (m, s) = (rem / 60, rem % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m:02}:{s:02}") }
}

/// Recorta a `n` caracteres con "…".
pub fn clip(text: &str, n: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= n {
        text.to_string()
    } else {
        text.chars().take(n - 1).collect::<String>() + "…"
    }
}

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// `lang` es el idioma del texto que se publica en Discord.
pub fn build_activity(np: &NowPlaying, tmdb: &mut Tmdb, use_name: bool, lang: &str) -> Value {
    build_activity_at(np, tmdb, use_name, lang, now_secs())
}

fn build_activity_at(np: &NowPlaying, tmdb: &mut Tmdb, use_name: bool, lang: &str, now: i64) -> Value {
    let item = &np.item;
    let kind = item["type"].as_str().unwrap_or("");
    let file_name = text(&item["file"]).map(|f| f.rsplit(['/', '\\']).next().unwrap_or(f)).unwrap_or("");
    let title = text(&item["title"]).or_else(|| text(&item["label"])).unwrap_or(file_name).to_string();

    let (header, details, state, large_text);
    match kind {
        "episode" => {
            details = text(&item["showtitle"]).unwrap_or(&title).to_string();
            // Solo el título del episodio: Discord recorta el texto (no lo desplaza).
            state = Some(title.clone());
            // Discord reconoce este formato (con ceros a la izquierda) y muestra "T01E10".
            let season = format!("{:02}", item["season"].as_i64().unwrap_or(0).max(0));
            let episode = format!("{:02}", item["episode"].as_i64().unwrap_or(0).max(0));
            large_text = i18n::tf_in(lang, "presence.season_episode", &[("season", &season), ("episode", &episode)]);
            header = details.clone();
        }
        "movie" => {
            header = title.clone();
            // TMDb tiene prioridad sobre Kodi en año y director.
            let info = text(&item["uniqueid"]["tmdb"]).map(|id| tmdb.movie_info(id)).unwrap_or_default();
            let year = info.year.or_else(|| item["year"].as_i64().filter(|y| *y > 0));
            let dirs: Vec<String> = info.directors.unwrap_or_else(|| {
                item["director"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|d| d.as_str().map(String::from)).collect())
                    .unwrap_or_default()
            });
            let directors = dirs.iter().take(2).cloned().collect::<Vec<_>>().join(", ");
            let parts: Vec<String> =
                [year.map(|y| y.to_string()), (!directors.is_empty()).then_some(directors)].into_iter().flatten().collect();
            details = if parts.is_empty() { title.clone() } else { parts.join(" · ") };
            let genres = item["genre"]
                .as_array()
                .map(|a| a.iter().filter_map(|g| g.as_str()).take(3).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            state = (!genres.is_empty()).then_some(genres);
            large_text = match year {
                Some(y) => format!("{title} ({y})"),
                None => title.clone(),
            };
        }
        _ => {
            header = title.clone();
            details = title.clone();
            state = None;
            large_text = "Butaca".to_string();
        }
    }

    let mut a = Map::new();
    a.insert("type".into(), json!(3));
    if use_name {
        a.insert("details".into(), json!(clip(&details, 128)));
        a.insert("name".into(), json!(clip(&header, 128)));
        a.insert("status_display_type".into(), json!(0));
    } else {
        // Respaldo: sin `name`, para películas `details` pasa a ser el título.
        let d = if kind == "movie" { &header } else { &details };
        a.insert("details".into(), json!(clip(d, 128)));
        a.insert("status_display_type".into(), json!(2));
    }
    if let Some(s) = state {
        a.insert("state".into(), json!(clip(&s, 128)));
    }

    let mut assets = Map::new();
    assets.insert("large_text".into(), json!(clip(&large_text, 128)));
    assets.insert("large_image".into(), json!(pick_poster(np, tmdb).unwrap_or_else(|| FALLBACK_IMAGE.into())));
    a.insert("assets".into(), Value::Object(assets));

    let cur = to_seconds(&np.props["time"]);
    let total = to_seconds(&np.props["totaltime"]);
    if np.props["speed"].as_f64() == Some(0.0) {
        let paused = i18n::tf_in(lang, "presence.paused", &[("current", &fmt_time(cur)), ("total", &fmt_time(total))]);
        a.insert("state".into(), json!(clip(&paused, 128)));
    } else if total > 0 {
        let start = now - cur;
        a.insert("timestamps".into(), json!({"start": start, "end": start + total}));
    }
    Value::Object(a)
}

/// Actividad opcional cuando Kodi está conectado pero no reproduce nada.
pub fn idle_activity(use_name: bool, lang: &str) -> Value {
    let mut a = json!({
        "type": 3,
        "details": i18n::t_in(lang, "presence.idle"),
        "assets": {"large_image": FALLBACK_IMAGE, "large_text": "Butaca"},
    });
    if use_name {
        a["name"] = json!("Butaca");
        a["status_display_type"] = json!(0);
    } else {
        a["status_display_type"] = json!(2);
    }
    a
}

/// Compara ignorando pequeñas derivas de los timestamps (evita spam a Discord).
pub fn same_activity(a: Option<&Value>, b: Option<&Value>) -> bool {
    let (a, b) = match (a, b) {
        (None, None) => return true,
        (Some(a), Some(b)) => (a, b),
        _ => return false,
    };
    let (mut a2, mut b2) = (a.clone(), b.clone());
    let ta = a2.as_object_mut().and_then(|o| o.remove("timestamps"));
    let tb = b2.as_object_mut().and_then(|o| o.remove("timestamps"));
    if a2 != b2 {
        return false;
    }
    match (ta, tb) {
        (None, None) => true,
        (Some(x), Some(y)) => (x["start"].as_i64().unwrap_or(0) - y["start"].as_i64().unwrap_or(0)).abs() <= 3,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn np(item: Value, speed: i64) -> NowPlaying {
        NowPlaying {
            item,
            props: json!({
                "time": {"hours": 0, "minutes": 10, "seconds": 5},
                "totaltime": {"hours": 1, "minutes": 0, "seconds": 0},
                "speed": speed
            }),
            show_art: json!({}),
            show_uniqueid: json!({}),
        }
    }

    #[test]
    fn decodes_kodi_art() {
        assert_eq!(
            decode_kodi_art("image://https%3a%2f%2fimage.tmdb.org%2ft%2fp%2fx.jpg/").as_deref(),
            Some("https://image.tmdb.org/t/p/x.jpg")
        );
        assert_eq!(decode_kodi_art(""), None);
    }

    #[test]
    fn formats_time_and_clips() {
        assert_eq!(fmt_time(65), "01:05");
        assert_eq!(fmt_time(3725), "1:02:05");
        assert_eq!(clip("abcdef", 4), "abc…");
        assert_eq!(clip("  hi ", 4), "hi");
    }

    #[test]
    fn episode_playing() {
        let n = np(json!({"type": "episode", "title": "Piloto", "showtitle": "Serie", "season": 1, "episode": 10}), 1);
        let a = build_activity_at(&n, &mut Tmdb::new(""), true, "es", 1000);
        assert_eq!(a["name"], "Serie");
        assert_eq!(a["details"], "Serie");
        assert_eq!(a["state"], "Piloto");
        assert_eq!(a["assets"]["large_text"], "Temporada 01, episodio 10");
        assert_eq!(a["assets"]["large_image"], "butaca");
        assert_eq!(a["timestamps"]["start"], 1000 - 605);
        assert_eq!(a["timestamps"]["end"], 1000 - 605 + 3600);
    }

    #[test]
    fn presence_text_follows_the_language() {
        let n = np(json!({"type": "episode", "title": "T", "showtitle": "S", "season": 2, "episode": 3}), 0);
        let de = build_activity_at(&n, &mut Tmdb::new(""), true, "de", 1000);
        assert_eq!(de["assets"]["large_text"], "Staffel 02, Folge 03");
        assert_eq!(de["state"], "Pausiert · 10:05 / 1:00:00");
        let en = build_activity_at(&n, &mut Tmdb::new(""), true, "en", 1000);
        assert_eq!(en["assets"]["large_text"], "Season 02, episode 03");
        assert_eq!(idle_activity(true, "it")["details"], "In attesa di riproduzione");
        // Discord solo reconoce "episode" en minúscula y sin acento: por eso no es "épisode".
        let fr = build_activity_at(&n, &mut Tmdb::new(""), true, "fr", 1000);
        assert_eq!(fr["assets"]["large_text"], "Saison 02, episode 03");
    }

    #[test]
    fn movie_paused() {
        let n = np(
            json!({"type": "movie", "title": "Peli", "year": 1999, "director": ["A", "B", "C"], "genre": ["x", "y", "z", "w"]}),
            0,
        );
        let a = build_activity_at(&n, &mut Tmdb::new(""), true, "es", 1000);
        assert_eq!(a["details"], "1999 · A, B");
        assert_eq!(a["assets"]["large_text"], "Peli (1999)");
        assert_eq!(a["state"], "En pausa · 10:05 / 1:00:00");
        assert!(a.get("timestamps").is_none());
    }

    #[test]
    fn movie_fallback_without_name() {
        let n = np(json!({"type": "movie", "title": "Peli", "year": 1999, "genre": ["x"]}), 1);
        let a = build_activity_at(&n, &mut Tmdb::new(""), false, "es", 1000);
        assert!(a.get("name").is_none());
        assert_eq!(a["details"], "Peli");
        assert_eq!(a["status_display_type"], 2);
        assert_eq!(a["state"], "x");
    }

    #[test]
    fn other_video_uses_file_name() {
        let n = np(json!({"type": "unknown", "title": "", "file": "smb://srv/a/video.mkv"}), 1);
        let a = build_activity_at(&n, &mut Tmdb::new(""), true, "es", 1000);
        assert_eq!(a["details"], "video.mkv");
        assert_eq!(a["assets"]["large_text"], "Butaca");
    }

    #[test]
    fn public_kodi_poster_preferred() {
        let n = np(
            json!({"type": "movie", "title": "P", "art": {"poster": "image://https%3a%2f%2fx.test%2fp.jpg/"}}),
            1,
        );
        let a = build_activity_at(&n, &mut Tmdb::new(""), true, "es", 1000);
        assert_eq!(a["assets"]["large_image"], "https://x.test/p.jpg");
    }

    #[test]
    fn same_activity_tolerates_drift() {
        let a = json!({"details": "x", "timestamps": {"start": 100, "end": 200}});
        let b = json!({"details": "x", "timestamps": {"start": 102, "end": 202}});
        let c = json!({"details": "x", "timestamps": {"start": 110, "end": 210}});
        assert!(same_activity(Some(&a), Some(&b)));
        assert!(!same_activity(Some(&a), Some(&c)));
        assert!(!same_activity(Some(&a), None));
        assert!(same_activity(None, None));
    }
}
