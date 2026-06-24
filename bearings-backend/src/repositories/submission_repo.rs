//! Data access for the `submissions` table (public intake) plus the
//! duplicate-detection read it relies on.
//!
//! Handlers/services depend on `SubmissionRepository`, not on `SupabaseClient`,
//! so the intake logic can be unit-tested against a fake (see
//! `services::submission_service`).

use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A row to insert into `submissions`. The service builds it; the repo writes it.
#[derive(Debug, Clone, Serialize)]
pub struct NewSubmission {
    pub submission_type: String,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub link: Option<String>,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub submitter_name: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub privacy_mode: bool,
    pub urgent: bool,
}

/// Minimal projection for `select=id` reads/writes (replaces `serde_json::Value`).
#[derive(Debug, Deserialize)]
struct IdRow {
    id: i64,
}

#[async_trait]
pub trait SubmissionRepository: Send + Sync {
    /// Insert a submission, returning its new id.
    async fn insert(&self, new: NewSubmission) -> Result<i64, AppError>;

    /// True if an active event or place already exists with a similar name in
    /// the same city (substring `ilike` match — Phase-1 dedup).
    async fn duplicate_exists(&self, name: &str, city: &str) -> Result<bool, AppError>;
}

pub struct SupabaseSubmissionRepository {
    db: SupabaseClient,
}

impl SupabaseSubmissionRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SubmissionRepository for SupabaseSubmissionRepository {
    async fn insert(&self, new: NewSubmission) -> Result<i64, AppError> {
        // POST returns the created row id via `Prefer: return=representation`.
        let url = format!("{}/rest/v1/submissions?select=id", self.db.url);
        let rows: Vec<IdRow> = self
            .db
            .write_json_returning(reqwest::Method::POST, &url, &new)
            .await?;
        Ok(rows.first().map(|r| r.id).unwrap_or(0))
    }

    async fn duplicate_exists(&self, name: &str, city: &str) -> Result<bool, AppError> {
        // PostgREST `ilike` uses `*` wildcards; both values are URL-encoded.
        let name_pattern = urlencoding::encode(&format!("*{name}*")).into_owned();
        let city_encoded = urlencoding::encode(city).into_owned();
        for table in ["events", "places"] {
            let url = format!(
                "{}/rest/v1/{}?select=id&name=ilike.{}&city=eq.{}&active=eq.true&limit=1",
                self.db.url, table, name_pattern, city_encoded
            );
            let rows: Vec<IdRow> = self.db.get_json(&url).await?;
            if !rows.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
