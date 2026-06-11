//! A minimal TTL cache for Supabase read results.
//!
//! Keyed by request URL, holding the raw JSON body. Entries expire `ttl` after
//! insertion, so staleness is bounded by the TTL — we trade strict consistency
//! for far fewer network round-trips, which suits a slowly-changing public
//! directory. Reads dominate; the rare writes (votes, upvotes, submissions) use
//! other methods and surface on the next TTL refresh.
//!
//! One `Mutex<HashMap>` is plenty here: the key set is small (a handful of query
//! URLs) and the critical section is a clone of a `String`. If contention ever
//! showed up in a profile, a sharded or lock-free cache (e.g. `moka`) would drop
//! in behind this same interface.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct TtlCache {
    ttl: Duration,
    entries: Mutex<HashMap<String, (Instant, String)>>,
}

impl TtlCache {
    pub fn new(ttl: Duration) -> Self {
        Self { ttl, entries: Mutex::new(HashMap::new()) }
    }

    /// The cached body for `key` if present and not yet expired.
    pub fn get(&self, key: &str) -> Option<String> {
        let mut entries = self.entries.lock().expect("cache mutex poisoned");
        if let Some((inserted, body)) = entries.get(key) {
            if inserted.elapsed() < self.ttl {
                return Some(body.clone());
            }
        }
        entries.remove(key); // absent or expired — keep the map from growing stale
        None
    }

    pub fn put(&self, key: String, body: String) {
        self.entries
            .lock()
            .expect("cache mutex poisoned")
            .insert(key, (Instant::now(), body));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_value_within_ttl() {
        let c = TtlCache::new(Duration::from_secs(60));
        c.put("k".into(), "v".into());
        assert_eq!(c.get("k"), Some("v".to_string()));
    }

    #[test]
    fn expires_after_ttl() {
        let c = TtlCache::new(Duration::from_millis(10));
        c.put("k".into(), "v".into());
        std::thread::sleep(Duration::from_millis(25));
        assert_eq!(c.get("k"), None);
    }

    #[test]
    fn miss_returns_none() {
        let c = TtlCache::new(Duration::from_secs(60));
        assert_eq!(c.get("absent"), None);
    }
}
