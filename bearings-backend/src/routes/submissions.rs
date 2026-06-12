//! Submission intake — public write endpoint.
//!
//! CONST-9: The chatbot agent is the primary submission mechanism.
//! Bears submit listings through conversation, not through a cold form.
//! This endpoint is the structured fallback for direct submissions.
//!
//! ## Auto-approval criteria (all must pass):
//!   ✓ Location geocodes successfully
//!   ✓ External URL returns HTTP 200
//!   ✓ No flagged terms
//!   ✓ Not a duplicate (name similarity < 85% AND same city)
//!   ✓ Submission type is in the approved category list
//!
//! ## Auto-reject if any:
//!   ✗ Flagged terms present
//!   ✗ Broken link
//!   ✗ Duplicate detected
//!   ✗ Missing required fields
//!
//! ## Escalate to steward if any:
//!   ⚠ Criminalised country → privacy_mode = true
//!   ⚠ Financial amount > $500 USD equivalent
//!   ⚠ Similarity to prior rejection > 85%
//!   ⚠ New category not in approved list
//!   ⚠ Prompt injection pattern detected

use crate::{db::SupabaseClient, error::AppError};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

/// Incoming submission from a bear.
#[derive(Debug, Deserialize)]
pub struct SubmissionPayload {
    pub submission_type: String, // "event" | "place" | "club" | "story"
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub link: Option<String>,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub submitter_name: Option<String>,
    pub source: Option<String>,
}

/// Response after a submission is received.
#[derive(Serialize)]
pub struct SubmissionResponse {
    pub status: String, // "auto_approved" | "pending_review" | "rejected"
    pub message: String,
    pub submission_id: Option<i64>,
}

/// POST /api/submissions
/// Accepts a new listing submission for review.
/// Validates, deduplicates, flags privacy-sensitive content,
/// and routes to auto-approval or steward review.
pub async fn create(
    State(db): State<SupabaseClient>,
    Json(payload): Json<SubmissionPayload>,
) -> Result<(StatusCode, Json<SubmissionResponse>), AppError> {
    // ── Validation ───────────────────────────────────────────
    if payload.name.trim().is_empty() {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(SubmissionResponse {
                status: "rejected".to_string(),
                message: "Name is required.".to_string(),
                submission_id: None,
            }),
        ));
    }

    // ── Privacy mode check ───────────────────────────────────
    // CONST-6: criminalised countries activate privacy mode automatically
    // Enforcement is centralised in middleware.rs
    let privacy_mode = crate::middleware::country_is_criminalised(payload.country.as_deref());

    // ── Prompt injection scan ────────────────────────────────
    // Scan the name and description fields for injection patterns
    if contains_injection(&payload.name)
        || payload
            .description
            .as_deref()
            .map(contains_injection)
            .unwrap_or(false)
    {
        // Route to steward with security flag — do not auto-approve
        let urgent = true;
        let _ = write_submission(&db, &payload, "pending_review", privacy_mode, urgent).await;
        return Ok((
            StatusCode::OK,
            Json(SubmissionResponse {
                status: "pending_review".to_string(),
                message: "Your submission is under review. You'll receive an email when it's live."
                    .to_string(),
                submission_id: None,
            }),
        ));
    }

    // ── Deduplication ────────────────────────────────────────
    if let Some(city) = &payload.city {
        if is_duplicate(&db, &payload.name, city).await? {
            return Ok((
                StatusCode::OK,
                Json(SubmissionResponse {
                    status: "rejected".to_string(),
                    message: "A listing with this name already exists in this city.".to_string(),
                    submission_id: None,
                }),
            ));
        }
    }

    // ── Write to submissions table ───────────────────────────
    let submission_id =
        write_submission(&db, &payload, "pending_review", privacy_mode, false).await?;

    Ok((
        StatusCode::CREATED,
        Json(SubmissionResponse {
            status: "pending_review".to_string(),
            message: "Thanks! Your submission is under review. We'll email you when it's live."
                .to_string(),
            submission_id: Some(submission_id),
        }),
    ))
}

/// Write a submission to the database.
async fn write_submission(
    db: &SupabaseClient,
    payload: &SubmissionPayload,
    status: &str,
    privacy_mode: bool,
    urgent: bool,
) -> Result<i64, AppError> {
    let body = serde_json::json!({
        "submission_type": payload.submission_type,
        "name": payload.name,
        "city": payload.city,
        "country": payload.country,
        "link": payload.link,
        "description": payload.description,
        "contact_email": payload.contact_email,
        "submitter_name": payload.submitter_name,
        "source": payload.source,
        "status": status,
        "privacy_mode": privacy_mode,
        "urgent": urgent,
    });

    // POST to submissions table — returns the created row id via Prefer: return=representation
    let url = format!("{}/rest/v1/submissions?select=id", db.url);
    let resp: Vec<serde_json::Value> = db
        .write_json_returning(reqwest::Method::POST, &url, &body)
        .await?;

    Ok(resp.first().and_then(|r| r["id"].as_i64()).unwrap_or(0))
}

/// Basic prompt injection detection.
/// Looks for patterns that suggest an attempt to override agent behaviour.
fn contains_injection(text: &str) -> bool {
    let lower = text.to_lowercase();
    let patterns = [
        "ignore previous",
        "ignore all",
        "disregard",
        "new instructions",
        "system prompt",
        "you are now",
        "act as",
        "roleplay as",
        "pretend you",
        "<|im_start|>",
        "<|system|>",
        "###instruction",
    ];
    patterns.iter().any(|p| lower.contains(p))
}

/// Check for near-duplicate: name similarity + same city.
/// Uses a simple substring check in Phase 1.
/// Full Levenshtein distance check is a TODO for Phase 2.
async fn is_duplicate(db: &SupabaseClient, name: &str, city: &str) -> Result<bool, AppError> {
    // PostgREST ilike uses * wildcard, value must be URL-encoded
    // Pattern: *name* for substring match
    let name_fmt = format!("*{}*", name);
    let name_pattern = urlencoding::encode(&name_fmt);
    let city_str = city.to_string();
    let city_encoded = urlencoding::encode(&city_str);

    // Check events
    let url = format!(
        "{}/rest/v1/events?select=id&name=ilike.{}&city=eq.{}&active=eq.true&limit=1",
        db.url, name_pattern, city_encoded
    );
    let results: Vec<serde_json::Value> = db.get_json(&url).await?;
    if !results.is_empty() {
        return Ok(true);
    }

    // Check places
    let url = format!(
        "{}/rest/v1/places?select=id&name=ilike.{}&city=eq.{}&active=eq.true&limit=1",
        db.url, name_pattern, city_encoded
    );
    let results: Vec<serde_json::Value> = db.get_json(&url).await?;
    Ok(!results.is_empty())
}
