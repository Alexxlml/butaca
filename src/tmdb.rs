//! Cliente opcional de TMDb. Sin API key no hace ninguna llamada.

use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Default)]
pub struct MovieInfo {
    pub year: Option<i64>,
    pub directors: Option<Vec<String>>,
}

pub struct Tmdb {
    agent: ureq::Agent,
    api_key: String,
    posters: HashMap<(&'static str, String), Option<String>>,
    movies: HashMap<String, MovieInfo>,
}

impl Tmdb {
    pub fn new(api_key: &str) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build()
            .into();
        Self { agent, api_key: api_key.trim().to_string(), posters: HashMap::new(), movies: HashMap::new() }
    }

    fn get(&self, path: &str, extra: &str) -> Result<Value, String> {
        let url = format!("https://api.themoviedb.org/3/{path}?api_key={}{extra}", self.api_key);
        let mut resp = self.agent.get(&url).call().map_err(|e| sanitize(&e.to_string(), &self.api_key))?;
        resp.body_mut().read_json().map_err(|e| e.to_string())
    }

    /// `kind` es `"movie"` o `"tv"`.
    pub fn poster(&mut self, kind: &'static str, id: &str) -> Option<String> {
        if self.api_key.is_empty() || id.is_empty() {
            return None;
        }
        let key = (kind, id.to_string());
        if let Some(v) = self.posters.get(&key) {
            return v.clone();
        }
        let poster = match self.get(&format!("{kind}/{id}"), "") {
            Ok(d) => d["poster_path"].as_str().map(|p| format!("https://image.tmdb.org/t/p/w500{p}")),
            Err(e) => {
                log::warn!("TMDb failed for {kind} {id}: {e}");
                None
            }
        };
        self.posters.insert(key, poster.clone());
        poster
    }

    /// Año y directores (en alfabeto latino) de una película.
    pub fn movie_info(&mut self, id: &str) -> MovieInfo {
        if self.api_key.is_empty() || id.is_empty() {
            return MovieInfo::default();
        }
        if let Some(v) = self.movies.get(id) {
            return v.clone();
        }
        let info = match self.get(&format!("movie/{id}"), "&append_to_response=credits") {
            Ok(d) => {
                let year = d["release_date"].as_str().and_then(|s| s.get(..4)).and_then(|y| y.parse().ok());
                let directors: Vec<String> = d["credits"]["crew"]
                    .as_array()
                    .map(|c| {
                        c.iter()
                            .filter(|p| p["job"] == "Director")
                            .filter_map(|p| p["name"].as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                MovieInfo { year, directors: (!directors.is_empty()).then_some(directors) }
            }
            Err(e) => {
                log::warn!("TMDb (movie {id}) failed: {e}");
                MovieInfo::default()
            }
        };
        self.movies.insert(id.to_string(), info.clone());
        info
    }
}

/// Evita que la API key aparezca en el log si el error incluye la URL.
fn sanitize(msg: &str, key: &str) -> String {
    if key.is_empty() { msg.to_string() } else { msg.replace(key, "***") }
}
