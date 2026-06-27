//! Supabase client — thin wrapper over PostgREST (Supabase's REST API).
//!
//! ## Design choice: PostgREST vs SQLx
//!
//! URLs are built with `format!()` strings in each zone file rather than a
//! type-safe query builder. This is intentional at this stage:
//!
//! - **Portability**: PostgREST runs over HTTPS with just a URL + API key.
//!   No direct PostgreSQL port access needed — works on any VPS or Supabase Cloud.
//! - **Simplicity**: Each zone's SELECT clause is readable alongside the handler
//!   that uses it. No hidden N+1 queries, no lazy loading surprises.
//!
//! ## The "silent failure" concern — and why it's addressed
//!
//! A typo in a `select=` field name previously failed silently:
//!   `ev["nme"].as_str()` → returns `""` with no error.
//!
//! With typed structs (`EventRow`, `PlaceRow`, etc.) this is now a visible error:
//!   a missing required field causes `serde` to return `Err`, which propagates
//!   to the caller as a logged error response — not silent data corruption.
//!
//! Optional fields (`Option<String>`) that are absent from a partial SELECT
//! deserialize as `None` — expected and correct.
//!
//! ## Remaining runtime-only risks
//!
//! - Typo in table name → 404 from PostgREST (logged, not silent)
//! - Typo in filter column → 400 from PostgREST (logged, not silent)
//! - Schema drift (column renamed in DB but not in struct) → serde error
//!
//! All of these produce visible errors at runtime. For compile-time safety,
//! SQLx macros (`sqlx::query_as!`) would catch field names at `cargo build` time.
//! Migration path: Supabase exposes a direct PostgreSQL connection pooler,
//! so SQLx is compatible with Supabase as the database host.
//!
//! ## One method per access pattern
//! - `get_json<T>(url)` — read, deserialise into typed T
//! - `post_rpc(name, body)` — call a PostgREST RPC function
//! - `write_json_returning(method, url, body)` — write, return created rows

use crate::cache::TtlCache;
use crate::error::AppError;
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct SupabaseClient {
    pub url: String,
    pub anon_key: String,
    pub service_key: String,
    client: reqwest::Client,
    cache: Arc<TtlCache>,
}

impl SupabaseClient {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("SUPABASE_URL").context("SUPABASE_URL not set")?,
            anon_key: std::env::var("SUPABASE_ANON_KEY").context("SUPABASE_ANON_KEY not set")?,
            service_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .context("SUPABASE_SERVICE_ROLE_KEY not set")?,
            client: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(5))
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .context("failed to build HTTP client")?,
            // 30s TTL: bounds staleness while absorbing bursts of identical reads.
            cache: Arc::new(TtlCache::new(Duration::from_secs(30))),
        })
    }

    /// Execute a GET request against the Supabase REST API and deserialise the response.
    /// Used by all read-only route handlers.
    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, AppError> {
        // Warm cache hit: skip the ~40ms PostgREST round-trip entirely.
        if let Some(body) = self.cache.get(url) {
            return serde_json::from_str(&body).map_err(|e| AppError::Internal(e.into()));
        }

        let response = self
            .client
            .get(url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(AppError::Database)?;

        let status = response.status();
        let body = response.text().await.map_err(AppError::Database)?;
        if !status.is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Supabase error {}: {}",
                status,
                body
            )));
        }

        let value = serde_json::from_str::<T>(&body).map_err(|e| AppError::Internal(e.into()))?;
        self.cache.put(url.to_string(), body);
        Ok(value)
    }

    /// Like `get_json` but with the service key (RLS-bypassing) and no caching —
    /// for admin-only reads of locked tables (e.g. `candidate_title_holders`, which
    /// holds identity data and deliberately has no public-read policy).
    pub async fn get_json_service<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, AppError> {
        let response = self
            .client
            .get(url)
            .header("apikey", &self.service_key)
            .header("Authorization", format!("Bearer {}", self.service_key))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(AppError::Database)?;
        let status = response.status();
        let body = response.text().await.map_err(AppError::Database)?;
        if !status.is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Supabase error {}: {}",
                status,
                body
            )));
        }
        serde_json::from_str::<T>(&body).map_err(|e| AppError::Internal(e.into()))
    }

    /// POST to a Supabase RPC endpoint and deserialise the response.
    /// Used for stored-function calls like places_nearby.
    /// Uses the anon key — RPC functions called from public API.
    pub async fn post_rpc<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        rpc_name: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let url = format!("{}/rest/v1/rpc/{}", self.url, rpc_name);
        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(AppError::Database)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Supabase RPC error {}: {}",
                status,
                body
            )));
        }

        response.json::<T>().await.map_err(AppError::Database)
    }

    /// POST with service key, return the created rows deserialised into `Vec<T>`.
    /// Used by the submission repository to get the generated id back.
    pub async fn write_json_returning<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: &B,
    ) -> Result<Vec<T>, AppError> {
        let response = self
            .client
            .request(method, url)
            .header("apikey", &self.service_key)
            .header("Authorization", format!("Bearer {}", self.service_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(body)
            .send()
            .await
            .map_err(AppError::Database)?;
        response.json::<Vec<T>>().await.map_err(AppError::Database)
    }

    /// Fire-and-forget write (PATCH/POST) with no response body. Used by the
    /// admin zone (`ssr/zones/admin.rs`) for feed/candidate updates.
    pub async fn write_json<B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: &B,
    ) -> Result<(), AppError> {
        self.client
            .request(method, url)
            .header("apikey", &self.service_key)
            .header("Authorization", format!("Bearer {}", self.service_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(body)
            .send()
            .await
            .map_err(AppError::Database)?;
        Ok(())
    }
}

/// Log-and-default for fallible reads. Turns a failed query into an empty result
/// **and a warning log**, instead of a silently empty page section. Use in SSR
/// zones where a degraded render beats a 500, but the failure must still be seen.
pub(crate) trait LogErr<T> {
    fn or_log(self, context: &str) -> T;
}

impl<T: Default> LogErr<T> for Result<T, AppError> {
    fn or_log(self, context: &str) -> T {
        self.unwrap_or_else(|e| {
            tracing::warn!(context, error = %e, "query failed; rendering empty section");
            T::default()
        })
    }
}
