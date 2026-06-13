//! Bear Future — funding-proposal and treasury read endpoints.
//! Data access in `repositories::bear_future_repo`; this layer maps HTTP <-> repo
//! and applies privacy redaction (CONST-6) before responding.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::bear_future_repo::{BearFutureRepository, SupabaseBearFutureRepository};
use crate::repositories::transparency_repo::{
    SupabaseTransparencyRepository, TransparencyRepository,
};
use axum::{
    extract::{Query, State},
    Json,
};
use bearings_shared::models::{BearFutureProposal, OperationalLedger};
use serde::{Deserialize, Serialize};

/// The public Base/USDC wallet that keeps the lights on (manual, steward-run).
#[derive(Serialize)]
pub struct TreasurySummary {
    pub lights_wallet_balance_usd: f64,
    pub lights_wallet_address: Option<String>,
    pub lights_wallet_chain: String,
    pub lights_wallet_updated: Option<String>,
}

/// GET /api/treasury — the Base/USDC "keep the lights on" wallet.
pub async fn treasury(State(db): State<SupabaseClient>) -> Result<Json<TreasurySummary>, AppError> {
    let repo = SupabaseTransparencyRepository::new(db);
    let w = repo.wallet().await?;
    Ok(Json(TreasurySummary {
        lights_wallet_balance_usd: w.balance_usd,
        lights_wallet_address: (!w.address.is_empty()).then_some(w.address),
        lights_wallet_chain: w.chain,
        lights_wallet_updated: (!w.updated.is_empty()).then_some(w.updated),
    }))
}

#[derive(Deserialize)]
pub struct ProposalsQuery {
    pub status: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/bear-future — proposals; privacy_mode rows are redacted (CONST-6).
pub async fn proposals(
    State(db): State<SupabaseClient>,
    Query(params): Query<ProposalsQuery>,
) -> Result<Json<Vec<BearFutureProposal>>, AppError> {
    let repo = SupabaseBearFutureRepository::new(db);
    let proposals = repo
        .find_proposals(params.status, params.limit.unwrap_or(50))
        .await?;

    let redacted = proposals
        .into_iter()
        .map(|mut p| {
            if p.privacy_mode.unwrap_or(false) {
                p.receiving_wallet = Some("redacted — privacy mode".to_string());
            }
            p
        })
        .collect();
    Ok(Json(redacted))
}

/// GET /api/bear-future/funded — funded proposals with on-chain proof.
pub async fn funded(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<BearFutureProposal>>, AppError> {
    let repo = SupabaseBearFutureRepository::new(db);
    Ok(Json(repo.funded_proposals().await?))
}

#[derive(Deserialize)]
pub struct LedgerQuery {
    pub limit: Option<u32>,
}

/// GET /api/bear-future/ledger — public operational ledger (transparency layer).
pub async fn ledger(
    State(db): State<SupabaseClient>,
    Query(params): Query<LedgerQuery>,
) -> Result<Json<Vec<OperationalLedger>>, AppError> {
    let repo = SupabaseBearFutureRepository::new(db);
    Ok(Json(repo.ledger(params.limit.unwrap_or(50)).await?))
}
