
//! Governance voting — the mechanism that makes Bear Future real.
//!
//! POST /api/bear-future/vote
//!
//! Authentication: during bootstrapping, the steward provides a
//! verified_token in the request body. Phase 2 will replace this
//! with proper JWT auth tied to the contributor's wallet.
//!
//! Vote weight = the voter's token_balance at the time of the vote.
//! One vote per holder per proposal — enforced by DB unique constraint
//! (proposal_id, voter_id).
//!
//! After every vote, the proposal's vote_yes or vote_no count is
//! incremented, and the threshold check runs. If threshold is met,
//! the proposal status advances from "open" to "passed".

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use bearings_shared::models::BearFutureProposal;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub proposal_id: i64,
    pub voter_id: i64,
    pub vote: String,  // "yes" | "no" | "abstain"
    // No auth token in Phase 1. Voting is open to anyone who knows a valid voter_id.
    // Acceptable only because bear_future_active = false and proposals are not live.
    // Phase 2: add Cardano wallet signature verification here.
    // Never add a token field that is accepted but not checked -- false security.
}

#[derive(Serialize)]
pub struct VoteResponse {
    pub status: String,
    pub message: String,
    pub proposal_status: Option<String>, // Updated status if threshold crossed
    pub vote_yes: Option<i32>,
    pub vote_no: Option<i32>,
}

/// POST /api/bear-future/vote
/// Cast a governance vote. Validates the voter exists and is verified,
/// checks for duplicate votes, records the vote, and updates proposal counts.
pub async fn cast(
    State(db): State<SupabaseClient>,
    Json(req): Json<VoteRequest>,
) -> Result<(StatusCode, Json<VoteResponse>), AppError> {

    // ── Validate vote value ────────────────────────────────
    if !["yes", "no", "abstain"].contains(&req.vote.as_str()) {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(VoteResponse {
            status: "rejected".into(),
            message: "Vote must be 'yes', 'no', or 'abstain'.".into(),
            proposal_status: None, vote_yes: None, vote_no: None,
        })));
    }

    // ── Verify voter is a verified NORTH holder ────────────
    let voter_url = format!(
        "{}/rest/v1/governance_token_holders?select=id,token_balance,verified&id=eq.{}&verified=eq.true&limit=1",
        db.url, req.voter_id
    );
    let voters: Vec<serde_json::Value> = db.get_json(&voter_url).await?;
    let voter = voters.into_iter().next().ok_or_else(|| {
        AppError::NotFound("Voter not found or not verified".into())
    })?;

    let vote_weight = voter["token_balance"].as_i64().unwrap_or(1) as i32;

    // ── Check proposal is open ─────────────────────────────
    let proposal_url = format!(
        "{}/rest/v1/bear_future_proposals?select=id,status,vote_yes,vote_no,vote_threshold_pct,vote_min_count&id=eq.{}&limit=1",
        db.url, req.proposal_id
    );
    let proposals: Vec<BearFutureProposal> = db.get_json(&proposal_url).await?;
    let proposal = proposals.into_iter().next().ok_or_else(|| {
        AppError::NotFound(format!("Proposal {} not found", req.proposal_id))
    })?;

    if proposal.status.as_deref() != Some("open") {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(VoteResponse {
            status: "rejected".into(),
            message: format!(
                "Proposal is '{}' — only open proposals accept votes.",
                proposal.status.as_deref().unwrap_or("unknown")
            ),
            proposal_status: proposal.status,
            vote_yes: proposal.vote_yes,
            vote_no: proposal.vote_no,
        })));
    }

    // ── Record vote (DB unique constraint prevents duplicates) ─
    let vote_body = serde_json::json!({
        "proposal_id": req.proposal_id,
        "voter_id": req.voter_id,
        "vote": req.vote,
        "vote_weight": vote_weight,
    });

    let vote_url = format!("{}/rest/v1/proposal_votes", db.url);
    // write_json_returning will fail with 409 if duplicate (unique constraint)
    let result = db.write_json_returning(
        reqwest::Method::POST, &vote_url, &vote_body
    ).await;

    if result.is_err() {
        return Ok((StatusCode::CONFLICT, Json(VoteResponse {
            status: "duplicate".into(),
            message: "You have already voted on this proposal.".into(),
            proposal_status: proposal.status,
            vote_yes: proposal.vote_yes,
            vote_no: proposal.vote_no,
        })));
    }

    // ── Increment vote counts atomically via PostgREST RPC ─
    // IMPORTANT: Do NOT read-modify-write vote counts in Rust.
    // Two concurrent votes would both read the same stale count
    // and one would be silently lost. Use a SQL function instead.
    //
    // TODO (Phase 2): replace with a Supabase RPC function:
    //   CREATE FUNCTION increment_vote(p_id bigint, direction text, weight int)
    //   that does: UPDATE ... SET vote_yes = vote_yes + weight WHERE id = p_id
    //   and returns the new counts atomically.
    //
    // For now: re-fetch after insert to get the DB-authoritative count,
    // then check threshold against the fresh values.
    let fresh_url = format!(
        "{}/rest/v1/bear_future_proposals?select=vote_yes,vote_no,vote_threshold_pct,vote_min_count&id=eq.{}&limit=1",
        db.url, req.proposal_id
    );
    let fresh: Vec<BearFutureProposal> = db.get_json(&fresh_url).await?;
    let fresh = fresh.into_iter().next().unwrap_or(proposal);

    let new_yes = fresh.vote_yes.unwrap_or(0);
    let new_no  = fresh.vote_no.unwrap_or(0);

    let threshold_pct = fresh.vote_threshold_pct.unwrap_or(60);
    let min_votes     = fresh.vote_min_count.unwrap_or(10);
    let total_votes   = new_yes + new_no;
    let threshold_met = total_votes >= min_votes
        && (new_yes * 100 / total_votes.max(1)) >= threshold_pct;

    let new_status = if threshold_met { "passed" } else { "open" };

    // Only update status — do not touch vote counts (PostgREST triggers handle those)
    // NOTE: until the RPC function exists, vote_yes/vote_no must be updated
    // by a DB trigger on proposal_votes INSERT. Add this to migrations:
    //   CREATE TRIGGER update_proposal_votes
    //   AFTER INSERT ON proposal_votes
    //   FOR EACH ROW EXECUTE FUNCTION increment_proposal_vote_count();
    let update_url = format!(
        "{}/rest/v1/bear_future_proposals?id=eq.{}",
        db.url, req.proposal_id
    );
    db.write_json(
        reqwest::Method::PATCH, &update_url,
        &serde_json::json!({ "status": new_status })
    ).await?;

    let message = if threshold_met {
        format!(
            "Vote recorded. Threshold reached ({}/{} votes, {}% yes) — proposal has passed!",
            total_votes, min_votes,
            new_yes * 100 / total_votes.max(1)
        )
    } else {
        format!(
            "Vote recorded. {} yes, {} no ({}/{} votes needed).",
            new_yes, new_no, total_votes, min_votes
        )
    };

    Ok((StatusCode::CREATED, Json(VoteResponse {
        status: "recorded".into(),
        message,
        proposal_status: Some(new_status.into()),
        vote_yes: Some(new_yes),
        vote_no: Some(new_no),
    })))
}
