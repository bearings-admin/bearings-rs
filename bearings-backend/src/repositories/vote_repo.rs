//! Data access for governance voting: token holders, proposals, and vote records.

use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;

/// A verified governance token holder eligible to vote.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Voter {
    pub id: i64,
    pub token_balance: Option<i32>,
    pub verified: Option<bool>,
}

/// The vote-relevant state of a proposal.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProposalVoteState {
    pub id: i64,
    pub status: Option<String>,
    pub vote_yes: Option<i32>,
    pub vote_no: Option<i32>,
    pub vote_threshold_pct: Option<i32>,
    pub vote_min_count: Option<i32>,
}

#[async_trait]
pub trait VoteRepository: Send + Sync {
    /// A verified voter by id, or `None` if not found / not verified.
    async fn find_verified_voter(&self, voter_id: i64) -> Result<Option<Voter>, AppError>;
    /// A proposal's vote state by id.
    async fn find_proposal(&self, proposal_id: i64) -> Result<Option<ProposalVoteState>, AppError>;
    /// Record a vote. Returns `false` if the insert was rejected (e.g. the
    /// (proposal_id, voter_id) unique constraint — i.e. a duplicate vote).
    async fn record_vote(
        &self,
        proposal_id: i64,
        voter_id: i64,
        vote: &str,
        weight: i32,
    ) -> Result<bool, AppError>;
    /// Set a proposal's status.
    async fn update_proposal_status(&self, proposal_id: i64, status: &str) -> Result<(), AppError>;
}

pub struct SupabaseVoteRepository {
    db: SupabaseClient,
}
impl SupabaseVoteRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl VoteRepository for SupabaseVoteRepository {
    async fn find_verified_voter(&self, voter_id: i64) -> Result<Option<Voter>, AppError> {
        let url = format!(
            "{}/rest/v1/governance_token_holders?select=id,token_balance,verified&id=eq.{}&verified=eq.true&limit=1",
            self.db.url, voter_id
        );
        let mut rows: Vec<Voter> = self.db.get_json(&url).await?;
        Ok(rows.pop())
    }

    async fn find_proposal(&self, proposal_id: i64) -> Result<Option<ProposalVoteState>, AppError> {
        let url = format!(
            "{}/rest/v1/bear_future_proposals?select=id,status,vote_yes,vote_no,vote_threshold_pct,vote_min_count&id=eq.{}&limit=1",
            self.db.url, proposal_id
        );
        let mut rows: Vec<ProposalVoteState> = self.db.get_json(&url).await?;
        Ok(rows.pop())
    }

    async fn record_vote(
        &self,
        proposal_id: i64,
        voter_id: i64,
        vote: &str,
        weight: i32,
    ) -> Result<bool, AppError> {
        let url = format!("{}/rest/v1/proposal_votes", self.db.url);
        let body = serde_json::json!({
            "proposal_id": proposal_id,
            "voter_id":    voter_id,
            "vote":        vote,
            "vote_weight": weight,
        });
        // A unique-constraint violation comes back as a non-representation body
        // that fails to deserialize — treat any insert error as "already voted".
        Ok(self
            .db
            .write_json_returning(reqwest::Method::POST, &url, &body)
            .await
            .is_ok())
    }

    async fn update_proposal_status(&self, proposal_id: i64, status: &str) -> Result<(), AppError> {
        let url = format!(
            "{}/rest/v1/bear_future_proposals?id=eq.{}",
            self.db.url, proposal_id
        );
        self.db
            .write_json(
                reqwest::Method::PATCH,
                &url,
                &serde_json::json!({ "status": status }),
            )
            .await
    }
}
