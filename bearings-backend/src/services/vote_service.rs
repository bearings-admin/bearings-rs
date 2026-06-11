//! Governance voting logic.

use crate::error::AppError;
use crate::repositories::vote_repo::VoteRepository;

/// The outcome of attempting to cast a vote. The HTTP layer maps each variant
/// to a status code + response body; the logic here is transport-agnostic.
#[derive(Debug, PartialEq)]
pub enum VoteOutcome {
    /// Vote value was not yes / no / abstain.
    InvalidVote,
    /// Voter id is unknown or not a verified holder.
    VoterNotFound,
    /// Proposal id does not exist.
    ProposalNotFound { proposal_id: i64 },
    /// Proposal is not open for voting.
    ProposalClosed { status: String, yes: i32, no: i32 },
    /// This holder already voted on this proposal.
    Duplicate { yes: i32, no: i32 },
    /// Vote recorded; `passed` is true if it crossed the threshold.
    Recorded { passed: bool, yes: i32, no: i32, total: i32, min_votes: i32 },
}

const VALID_VOTES: [&str; 3] = ["yes", "no", "abstain"];

/// Orchestrates a vote across the [`VoteRepository`]. Generic over the repo so
/// it can be exercised with a fake in tests.
pub struct VoteService<R: VoteRepository> {
    repo: R,
}

impl<R: VoteRepository> VoteService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn cast(&self, proposal_id: i64, voter_id: i64, vote: &str) -> Result<VoteOutcome, AppError> {
        if !VALID_VOTES.contains(&vote) {
            return Ok(VoteOutcome::InvalidVote);
        }

        let voter = match self.repo.find_verified_voter(voter_id).await? {
            Some(v) => v,
            None => return Ok(VoteOutcome::VoterNotFound),
        };
        let weight = voter.token_balance.unwrap_or(1);

        let proposal = match self.repo.find_proposal(proposal_id).await? {
            Some(p) => p,
            None => return Ok(VoteOutcome::ProposalNotFound { proposal_id }),
        };
        if proposal.status.as_deref() != Some("open") {
            return Ok(VoteOutcome::ProposalClosed {
                status: proposal.status.clone().unwrap_or_default(),
                yes:    proposal.vote_yes.unwrap_or(0),
                no:     proposal.vote_no.unwrap_or(0),
            });
        }

        if !self.repo.record_vote(proposal_id, voter_id, vote, weight).await? {
            return Ok(VoteOutcome::Duplicate {
                yes: proposal.vote_yes.unwrap_or(0),
                no:  proposal.vote_no.unwrap_or(0),
            });
        }

        // Re-fetch DB-authoritative counts. The `update_proposal_votes` trigger on
        // proposal_votes INSERT maintains them atomically; never read-modify-write here.
        let fresh = self.repo.find_proposal(proposal_id).await?.unwrap_or(proposal);
        let yes = fresh.vote_yes.unwrap_or(0);
        let no  = fresh.vote_no.unwrap_or(0);
        let threshold_pct = fresh.vote_threshold_pct.unwrap_or(60);
        let min_votes     = fresh.vote_min_count.unwrap_or(10);
        let total = yes + no;
        let passed = total >= min_votes && (yes * 100 / total.max(1)) >= threshold_pct;

        self.repo.update_proposal_status(proposal_id, if passed { "passed" } else { "open" }).await?;

        Ok(VoteOutcome::Recorded { passed, yes, no, total, min_votes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::vote_repo::{ProposalVoteState, Voter, VoteRepository};
    use async_trait::async_trait;

    /// A configurable in-memory repository — lets us test the governance logic
    /// without a database. This is the payoff of depending on the trait.
    #[derive(Default)]
    struct FakeVoteRepo {
        voter:     Option<Voter>,
        proposal:  Option<ProposalVoteState>,
        record_ok: bool,
    }

    #[async_trait]
    impl VoteRepository for FakeVoteRepo {
        async fn find_verified_voter(&self, _: i64) -> Result<Option<Voter>, AppError> {
            Ok(self.voter.clone())
        }
        async fn find_proposal(&self, _: i64) -> Result<Option<ProposalVoteState>, AppError> {
            Ok(self.proposal.clone())
        }
        async fn record_vote(&self, _: i64, _: i64, _: &str, _: i32) -> Result<bool, AppError> {
            Ok(self.record_ok)
        }
        async fn update_proposal_status(&self, _: i64, _: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn voter(balance: i32) -> Voter {
        Voter { id: 1, token_balance: Some(balance), verified: Some(true) }
    }
    fn proposal(status: &str, yes: i32, no: i32) -> ProposalVoteState {
        ProposalVoteState {
            id: 1, status: Some(status.into()),
            vote_yes: Some(yes), vote_no: Some(no),
            vote_threshold_pct: Some(60), vote_min_count: Some(10),
        }
    }

    #[tokio::test]
    async fn rejects_invalid_vote_value() {
        let svc = VoteService::new(FakeVoteRepo::default());
        assert_eq!(svc.cast(1, 1, "maybe").await.unwrap(), VoteOutcome::InvalidVote);
    }

    #[tokio::test]
    async fn rejects_unknown_voter() {
        let svc = VoteService::new(FakeVoteRepo { voter: None, ..Default::default() });
        assert_eq!(svc.cast(1, 1, "yes").await.unwrap(), VoteOutcome::VoterNotFound);
    }

    #[tokio::test]
    async fn rejects_vote_on_closed_proposal() {
        let svc = VoteService::new(FakeVoteRepo {
            voter: Some(voter(5)),
            proposal: Some(proposal("passed", 8, 2)),
            record_ok: true,
        });
        match svc.cast(1, 1, "yes").await.unwrap() {
            VoteOutcome::ProposalClosed { status, .. } => assert_eq!(status, "passed"),
            other => panic!("expected ProposalClosed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn duplicate_vote_is_reported() {
        let svc = VoteService::new(FakeVoteRepo {
            voter: Some(voter(5)),
            proposal: Some(proposal("open", 1, 0)),
            record_ok: false, // insert rejected -> duplicate
        });
        assert!(matches!(svc.cast(1, 1, "yes").await.unwrap(), VoteOutcome::Duplicate { .. }));
    }

    #[tokio::test]
    async fn records_vote_below_threshold_keeps_open() {
        let svc = VoteService::new(FakeVoteRepo {
            voter: Some(voter(1)),
            proposal: Some(proposal("open", 3, 2)), // 5 votes < min 10
            record_ok: true,
        });
        match svc.cast(1, 1, "yes").await.unwrap() {
            VoteOutcome::Recorded { passed, .. } => assert!(!passed),
            other => panic!("expected Recorded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn records_vote_crossing_threshold_passes() {
        let svc = VoteService::new(FakeVoteRepo {
            voter: Some(voter(1)),
            proposal: Some(proposal("open", 9, 1)), // 10 votes, 90% yes >= 60%
            record_ok: true,
        });
        match svc.cast(1, 1, "yes").await.unwrap() {
            VoteOutcome::Recorded { passed, total, .. } => { assert!(passed); assert_eq!(total, 10); }
            other => panic!("expected Recorded, got {other:?}"),
        }
    }
}
