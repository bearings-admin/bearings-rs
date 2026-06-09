
//! Supabase client — thin, no ORM.
//! One method: get_json<T>(url) returns T deserialized from the PostgREST response.
//! All query building happens in the route handlers — explicit, readable, debuggable.

use anyhow::{Context, Result};
use crate::error::AppError;

#[derive(Clone)]
pub struct SupabaseClient {
    pub url: String,
    pub anon_key: String,
    pub service_key: String,
    client: reqwest::Client,
}

impl SupabaseClient {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("SUPABASE_URL")
                .context("SUPABASE_URL not set")?,
            anon_key: std::env::var("SUPABASE_ANON_KEY")
                .context("SUPABASE_ANON_KEY not set")?,
            service_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .context("SUPABASE_SERVICE_ROLE_KEY not set")?,
            client: reqwest::Client::new(),
        })
    }

    /// Execute a GET request against the Supabase REST API and deserialise the response.
    /// Used by all read-only route handlers.
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, AppError> {
        let response = self.client
            .get(url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(AppError::Database)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(
                anyhow::anyhow!("Supabase error {}: {}", status, body)
            ));
        }

        response.json::<T>().await.map_err(AppError::Database)
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
        let response = self.client
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
            return Err(AppError::Internal(
                anyhow::anyhow!("Supabase RPC error {}: {}", status, body)
            ));
        }

        response.json::<T>().await.map_err(AppError::Database)
    }

    /// POST with service key, return the created rows.
    /// Used by submissions.rs to get the generated ID back.
    pub async fn write_json_returning<B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: &B,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let response = self.client
            .request(method, url)
            .header("apikey", &self.service_key)
            .header("Authorization", format!("Bearer {}", self.service_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(body)
            .send()
            .await
            .map_err(AppError::Database)?;
        response.json().await.map_err(AppError::Database)
    }

    // write_json kept for future authenticated writes (voting, submissions)
    // Currently unused — will be used when auth is added in Phase 2
    #[allow(dead_code)]
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
