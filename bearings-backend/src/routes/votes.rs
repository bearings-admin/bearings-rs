//! POST /api/bear-future/vote — cast a governance vote.
//!
//! Thin HTTP layer: parse the request, delegate to [`VoteService`], map the
//! outcome to a status code + body. Governance logic lives in the service;
//! data access in `repositories::vote_repo`.
//!
//! Auth: none in Phase 1 (bear_future_active = false, proposals are not live).
//! Phase 2 will verify a Cardano wallet signature before the service call.
//! Never accept an auth token that is not actually checked — that is false security.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::vote_repo::SupabaseVoteRepository;
use crate::services::vote_service::{VoteOutcome, VoteService};

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub proposal_id: i64,
    pub voter_id:    i64,
    pub vote:        String, // "yes" | "no" | "abstain"
}

#[derive(Serialize)]
pub struct VoteResponse {
    pub status:          String,
    pub message:         String,
    pub proposal_status: Option<String>,
    pub vote_yes:        Option<i32>,
    pub vote_no:         Option<i32>,
}

/// POST /api/bear-future/vote
pub async fn cast(
    State(db): State<SupabaseClient>,
    Json(req): Json<VoteRequest>,
) -> Result<(StatusCode, Json<VoteResponse>), AppError> {
    let service = VoteService::new(SupabaseVoteRepository::new(db));
    let outcome = service.cast(req.proposal_id, req.voter_id, &req.vote).await?;
    map_outcome(outcome)
}

/// Map a [`VoteOutcome`] to an HTTP status + response body.
fn map_outcome(outcome: VoteOutcome) -> Result<(StatusCode, Json<VoteResponse>), AppError> {
    let resp = |status: StatusCode, s: &str, message: String,
                pstatus: Option<String>, yes: Option<i32>, no: Option<i32>| {
        Ok((status, Json(VoteResponse {
            status: s.into(), message, proposal_status: pstatus, vote_yes: yes, vote_no: no,
        })))
    };

    match outcome {
        VoteOutcome::InvalidVote => resp(
            StatusCode::UNPROCESSABLE_ENTITY, "rejected",
            "Vote must be 'yes', 'no', or 'abstain'.".into(), None, None, None,
        ),
        VoteOutcome::VoterNotFound =>
            Err(AppError::NotFound("Voter not found or not verified".into())),
        VoteOutcome::ProposalNotFound { proposal_id } =>
            Err(AppError::NotFound(format!("Proposal {proposal_id} not found"))),
        VoteOutcome::ProposalClosed { status, yes, no } => resp(
            StatusCode::UNPROCESSABLE_ENTITY, "rejected",
            format!("Proposal is '{status}' — only open proposals accept votes."),
            Some(status), Some(yes), Some(no),
        ),
        VoteOutcome::Duplicate { yes, no } => resp(
            StatusCode::CONFLICT, "duplicate",
            "You have already voted on this proposal.".into(),
            Some("open".into()), Some(yes), Some(no),
        ),
        VoteOutcome::Recorded { passed, yes, no, total, min_votes } => {
            let pct = yes * 100 / total.max(1);
            let message = if passed {
                format!("Vote recorded. Threshold reached ({total}/{min_votes} votes, {pct}% yes) — proposal has passed!")
            } else {
                format!("Vote recorded. {yes} yes, {no} no ({total}/{min_votes} votes needed).")
            };
            resp(StatusCode::CREATED, "recorded", message,
                 Some(if passed { "passed" } else { "open" }.into()), Some(yes), Some(no))
        }
    }
}
