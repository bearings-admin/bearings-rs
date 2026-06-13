//! Application configuration — validated at startup.
//!
//! All environment variables are checked here at startup rather than
//! scattered across individual handlers. If a required var is missing,
//! the process fails immediately with a clear error message.
//!
//! Usage:
//!   let config = Config::from_env()?;
//!   // Pass into Axum state alongside SupabaseClient

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    // Server
    pub port: u16,
    pub host: String,

    // Supabase (also on SupabaseClient — config adds validation)
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_service_key: String,

    // Feature flags — read from platform_settings at startup
    pub bear_future_active: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .context("PORT must be a valid port number")?,

            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),

            supabase_url: std::env::var("SUPABASE_URL").context("SUPABASE_URL is required")?,

            supabase_anon_key: std::env::var("SUPABASE_ANON_KEY")
                .context("SUPABASE_ANON_KEY is required")?,

            supabase_service_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .context("SUPABASE_SERVICE_ROLE_KEY is required")?,

            // Feature flags start at safe defaults.
            // Call config.load_feature_flags(&db).await after construction
            // to sync with platform_settings. Kept separate so Config::from_env()
            // stays synchronous (no async in constructors).
            bear_future_active: false,
        })
    }

    /// Load live feature flags from platform_settings.
    /// Call this once at startup after SupabaseClient is ready.
    /// Silently keeps defaults if Supabase is unreachable.
    pub async fn load_feature_flags(&mut self, client: &reqwest::Client, anon_key: &str) {
        let url = format!(
            "{}/rest/v1/platform_settings?select=key,value&key=in.(bear_future_active)",
            self.supabase_url
        );
        let Ok(resp) = client
            .get(&url)
            .header("apikey", anon_key)
            .header("Authorization", format!("Bearer {}", anon_key))
            .send()
            .await
        else {
            tracing::warn!("Could not reach platform_settings — using default feature flags");
            return;
        };

        #[derive(serde::Deserialize)]
        struct Setting {
            key: String,
            value: Option<String>,
        }

        let Ok(settings) = resp.json::<Vec<Setting>>().await else {
            return;
        };

        for s in settings {
            if let ("bear_future_active", Some(v)) = (s.key.as_str(), s.value.as_deref()) {
                self.bear_future_active = v == "true";
            }
        }

        tracing::info!(
            "Feature flags loaded — bear_future: {}",
            self.bear_future_active
        );
    }

    /// Log current config at startup (without secrets).
    pub fn log_startup(&self) {
        tracing::info!("bearings-backend configuration:");
        tracing::info!("  Port:             {}", self.port);
        tracing::info!("  Supabase:         {}", self.supabase_url);
        tracing::info!("  Bear Future:      {}", self.bear_future_active);
    }
}
