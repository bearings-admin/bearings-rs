//! Supabase writer for the treasury agent.
//! Service-role key gives write access for logging transactions,
//! updating platform_settings, and managing governance records.
//!
//! The reqwest::Client is stored on the struct — creating one
//! per call is expensive (new TCP connection pool each time).

use crate::error::AgentError;
use bearings_shared::models::OperationalLedger;
use serde::Serialize;
use serde_json::json;

pub struct SupabaseWriter {
    pub url: String,
    service_key: String,
    anon_key: String,
    client: reqwest::Client,
}

impl SupabaseWriter {
    pub fn from_env() -> anyhow::Result<Self> {
        use anyhow::Context;
        Ok(Self {
            url: std::env::var("SUPABASE_URL").context("SUPABASE_URL not set")?,
            service_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .context("SUPABASE_SERVICE_ROLE_KEY not set")?,
            anon_key: std::env::var("SUPABASE_ANON_KEY").context("SUPABASE_ANON_KEY not set")?,
            client: reqwest::Client::new(),
        })
    }

    /// Build a pre-configured request to the Supabase REST API.
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{}/rest/v1/{}", self.url, path))
            .header("apikey", &self.service_key)
            .header("Authorization", format!("Bearer {}", self.service_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
    }

    /// GET from any table path using the anon key (public reads).
    pub async fn get(&self, path: &str) -> Result<Vec<serde_json::Value>, AgentError> {
        let entries: Vec<serde_json::Value> = self
            .client
            .get(format!("{}/rest/v1/{}", self.url, path))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await?
            .json()
            .await?;
        Ok(entries)
    }

    /// INSERT a JSON body into a table.
    pub async fn insert<B: Serialize>(&self, table: &str, body: &B) -> Result<(), AgentError> {
        self.request(reqwest::Method::POST, table)
            .json(body)
            .send()
            .await?;
        Ok(())
    }

    /// PATCH (partial update) at a filtered path.
    /// path example: "governance_token_holders?id=eq.42"
    pub async fn patch<B: Serialize>(&self, path: &str, body: &B) -> Result<(), AgentError> {
        self.request(reqwest::Method::PATCH, path)
            .json(body)
            .send()
            .await?;
        Ok(())
    }

    /// Check whether a transaction hash is already in the ledger.
    pub async fn ledger_entry_exists(&self, tx_hash: &str) -> Result<bool, AgentError> {
        let path = format!(
            "operational_ledger?tx_hash=eq.{}&select=id&limit=1",
            tx_hash
        );
        let entries = self.get(&path).await?;
        Ok(!entries.is_empty())
    }

    /// Insert a new operational ledger entry.
    /// The `id` field is GENERATED ALWAYS by the database — set to 0 in the struct.
    pub async fn insert_ledger_entry(&self, entry: &OperationalLedger) -> Result<(), AgentError> {
        self.insert("operational_ledger", entry).await
    }

    /// Update a key/value pair in platform_settings.
    pub async fn update_platform_setting(&self, key: &str, value: &str) -> Result<(), AgentError> {
        let path = format!("platform_settings?key=eq.{}", key);
        self.patch(&path, &json!({ "value": value })).await
    }
}
