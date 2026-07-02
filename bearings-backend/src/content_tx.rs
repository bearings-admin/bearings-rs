//! Runtime cache for translated DB prose — Layer 2 of the i18n plan.
//!
//! Static UI chrome is baked into `i18n.rs`; dynamic DB content (event / place / club /
//! campaign descriptions) can't be — it's translated ahead of time by
//! `scripts/translate.py` into the `content_translations` table, loaded into memory here
//! (refreshed by [`crate::spawn_content_refresh`]), and looked up at render time via
//! [`tc`]. English, empty text, or a cache miss returns the original text unchanged, so
//! the render path is a pure in-memory lookup that never blocks on the network.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// One `content_translations` row, as read from PostgREST for the refresh.
#[derive(serde::Deserialize)]
pub struct ContentTxRow {
    pub target_lang: String,
    pub source_text: String,
    pub translated_text: String,
}

fn cache() -> &'static RwLock<HashMap<(String, String), String>> {
    static C: OnceLock<RwLock<HashMap<(String, String), String>>> = OnceLock::new();
    C.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Translate dynamic content prose into `lang`. Returns the original text unchanged for
/// English, empty input, or a cache miss.
pub fn tc(text: &str, lang: &str) -> String {
    if lang == "en" || text.trim().is_empty() {
        return text.to_string();
    }
    cache()
        .read()
        .unwrap()
        .get(&(lang.to_string(), text.to_string()))
        .cloned()
        .unwrap_or_else(|| text.to_string())
}

/// Replace the in-memory cache from `(target_lang, source_text, translated_text)` rows.
pub fn load(rows: Vec<ContentTxRow>) {
    let mut m = HashMap::new();
    for r in rows {
        m.insert((r.target_lang, r.source_text), r.translated_text);
    }
    *cache().write().unwrap() = m;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_and_miss_fall_back_to_original() {
        load(vec![ContentTxRow {
            target_lang: "de".into(),
            source_text: "Annual bear run.".into(),
            translated_text: "Jährlicher Bären-Lauf.".into(),
        }]);
        assert_eq!(tc("Annual bear run.", "en"), "Annual bear run."); // English passthrough
        assert_eq!(tc("Annual bear run.", "de"), "Jährlicher Bären-Lauf."); // hit
        assert_eq!(tc("Uncached text.", "de"), "Uncached text."); // miss -> original
        assert_eq!(tc("", "de"), ""); // empty
    }
}
