//! Data access for the Bear Future governance + treasury tables.

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use bearings_shared::models::{BearFutureProposal, OperationalLedger};
use crate::db::SupabaseClient;
use crate::error::AppError;
use super::clause;

/// Public view of a NORTH token holder — wallet addresses are never selected.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicTokenHolder {
    pub display_name:             Option<String>,
    pub contributor_tier:         Option<String>,
    pub verified_role_description: Option<String>,
    pub token_balance:            Option<i32>,
    pub verified:                 Option<bool>,
}

#[async_trait]
pub trait BearFutureRepository: Send + Sync {
    /// Treasury-related rows from platform_settings, as a key -> value map.
    async fn treasury_settings(&self) -> Result<HashMap<String, String>, AppError>;
    /// Proposals, newest first, optionally filtered by status.
    async fn find_proposals(&self, status: Option<String>, limit: u32) -> Result<Vec<BearFutureProposal>, AppError>;
    /// Funded proposals only (with on-chain tx_hash).
    async fn funded_proposals(&self) -> Result<Vec<BearFutureProposal>, AppError>;
    /// Verified NORTH token holders (no wallet addresses).
    async fn token_holders(&self) -> Result<Vec<PublicTokenHolder>, AppError>;
    /// Public operational ledger entries, newest first.
    async fn ledger(&self, limit: u32) -> Result<Vec<OperationalLedger>, AppError>;
}

pub struct SupabaseBearFutureRepository { db: SupabaseClient }
impl SupabaseBearFutureRepository { pub fn new(db: SupabaseClient) -> Self { Self { db } } }

#[async_trait]
impl BearFutureRepository for SupabaseBearFutureRepository {
    async fn treasury_settings(&self) -> Result<HashMap<String, String>, AppError> {
        let url = format!(
            "{}/rest/v1/platform_settings?select=key,value&key=in.(treasury_balance_ada,operational_balance_ada,governance_token_name,governance_dao_threshold,treasury_phase,treasury_wallet_ada,operational_wallet_ada,bear_future_active)",
            self.db.url
        );
        #[derive(Deserialize)]
        struct Setting { key: String, value: Option<String> }
        let rows: Vec<Setting> = self.db.get_json(&url).await?;
        Ok(rows.into_iter()
            .filter_map(|s| s.value.map(|v| (s.key, v)))
            .collect())
    }

    async fn find_proposals(&self, status: Option<String>, limit: u32) -> Result<Vec<BearFutureProposal>, AppError> {
        let mut url = format!(
            "{}/rest/v1/bear_future_proposals?select=*&order=created_at.desc&limit={}",
            self.db.url, limit
        );
        if let Some(s) = status { url.push_str(&clause("status", "eq", &s)); }
        self.db.get_json::<Vec<BearFutureProposal>>(&url).await
    }

    async fn funded_proposals(&self) -> Result<Vec<BearFutureProposal>, AppError> {
        let url = format!(
            "{}/rest/v1/bear_future_proposals?select=*&status=eq.funded&order=created_at.desc",
            self.db.url
        );
        self.db.get_json::<Vec<BearFutureProposal>>(&url).await
    }

    async fn token_holders(&self) -> Result<Vec<PublicTokenHolder>, AppError> {
        let url = format!(
            "{}/rest/v1/governance_token_holders?select=display_name,contributor_tier,verified_role_description,token_balance,verified&verified=eq.true&order=token_balance.desc",
            self.db.url
        );
        self.db.get_json::<Vec<PublicTokenHolder>>(&url).await
    }

    async fn ledger(&self, limit: u32) -> Result<Vec<OperationalLedger>, AppError> {
        let url = format!(
            "{}/rest/v1/operational_ledger?select=tx_date,direction,amount_ada,amount_usd,vendor,category,description,tx_hash&order=tx_date.desc&limit={}",
            self.db.url, limit
        );
        self.db.get_json::<Vec<OperationalLedger>>(&url).await
    }
}
