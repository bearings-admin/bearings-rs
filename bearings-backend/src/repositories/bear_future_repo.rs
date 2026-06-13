//! Data access for Bear Future funding proposals and the operational ledger.

use super::clause;
use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use bearings_shared::models::{BearFutureProposal, OperationalLedger};

#[async_trait]
pub trait BearFutureRepository: Send + Sync {
    /// Proposals, newest first, optionally filtered by status.
    async fn find_proposals(
        &self,
        status: Option<String>,
        limit: u32,
    ) -> Result<Vec<BearFutureProposal>, AppError>;
    /// Funded proposals only (with on-chain tx_hash).
    async fn funded_proposals(&self) -> Result<Vec<BearFutureProposal>, AppError>;
    /// Public operational ledger entries, newest first.
    async fn ledger(&self, limit: u32) -> Result<Vec<OperationalLedger>, AppError>;
}

pub struct SupabaseBearFutureRepository {
    db: SupabaseClient,
}
impl SupabaseBearFutureRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BearFutureRepository for SupabaseBearFutureRepository {
    async fn find_proposals(
        &self,
        status: Option<String>,
        limit: u32,
    ) -> Result<Vec<BearFutureProposal>, AppError> {
        let mut url = format!(
            "{}/rest/v1/bear_future_proposals?select=*&order=created_at.desc&limit={}",
            self.db.url, limit
        );
        if let Some(s) = status {
            url.push_str(&clause("status", "eq", &s));
        }
        self.db.get_json::<Vec<BearFutureProposal>>(&url).await
    }

    async fn funded_proposals(&self) -> Result<Vec<BearFutureProposal>, AppError> {
        let url = format!(
            "{}/rest/v1/bear_future_proposals?select=*&status=eq.funded&order=created_at.desc",
            self.db.url
        );
        self.db.get_json::<Vec<BearFutureProposal>>(&url).await
    }

    async fn ledger(&self, limit: u32) -> Result<Vec<OperationalLedger>, AppError> {
        let url = format!(
            "{}/rest/v1/operational_ledger?select=tx_date,direction,amount_usdc,amount_usd,vendor,category,description,tx_hash&order=tx_date.desc&limit={}",
            self.db.url, limit
        );
        self.db.get_json::<Vec<OperationalLedger>>(&url).await
    }
}
