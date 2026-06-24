//! Submission intake — public write endpoint.
//!
//! CONST-9: The chatbot agent is the primary submission mechanism; bears submit
//! through conversation, not a cold form. This endpoint is the structured
//! fallback. It is HTTP-only: map the DTO → service input, and the service's
//! outcome → HTTP. All validation / privacy / dedup / persistence logic lives in
//! `services::submission_service` (and is unit-tested there against a fake repo).

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::submission_repo::SupabaseSubmissionRepository;
use crate::services::submission_service::{
    DefaultSubmissionService, SubmissionInput, SubmissionOutcome, SubmissionService,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Incoming submission from a bear (the HTTP DTO).
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
    pub status: String, // "rejected" | "pending_review"
    pub message: String,
    pub submission_id: Option<i64>,
}

/// POST /api/submissions — accept a new listing submission for review.
pub async fn create(
    State(db): State<SupabaseClient>,
    Json(payload): Json<SubmissionPayload>,
) -> Result<(StatusCode, Json<SubmissionResponse>), AppError> {
    let repo = Arc::new(SupabaseSubmissionRepository::new(db));
    let service = DefaultSubmissionService::new(repo);

    let outcome = service
        .submit(SubmissionInput {
            submission_type: payload.submission_type,
            name: payload.name,
            city: payload.city,
            country: payload.country,
            link: payload.link,
            description: payload.description,
            contact_email: payload.contact_email,
            submitter_name: payload.submitter_name,
            source: payload.source,
        })
        .await?;

    let (code, status, message, submission_id) = match outcome {
        SubmissionOutcome::InvalidName => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "rejected",
            "Name is required.",
            None,
        ),
        SubmissionOutcome::Duplicate => (
            StatusCode::OK,
            "rejected",
            "A listing with this name already exists in this city.",
            None,
        ),
        SubmissionOutcome::FlaggedForReview => (
            StatusCode::OK,
            "pending_review",
            "Your submission is under review. You'll receive an email when it's live.",
            None,
        ),
        SubmissionOutcome::Accepted { id } => (
            StatusCode::CREATED,
            "pending_review",
            "Thanks! Your submission is under review. We'll email you when it's live.",
            Some(id),
        ),
    };

    Ok((
        code,
        Json(SubmissionResponse {
            status: status.to_string(),
            message: message.to_string(),
            submission_id,
        }),
    ))
}
