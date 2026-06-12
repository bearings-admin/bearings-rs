//! Bear Future — governance and treasury read endpoints.
//! Data access in `repositories::bear_future_repo`; this layer maps HTTP <-> repo
//! and applies privacy redaction (CONST-6) before responding.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::bear_future_repo::{
    BearFutureRepository, PublicTokenHolder, SupabaseBearFutureRepository,
};
use axum::{
    extract::{Query, State},
    Json,
};
use bearings_shared::models::{BearFutureProposal, OperationalLedger};
use serde::{Deserialize, Serialize};

/// Treasury summary for the Bear Future "The Pot" section.
#[derive(Serialize)]
pub struct TreasurySummary {
    pub community_treasury_ada: f64,
    pub operational_wallet_ada: f64,
    pub governance_token_total_minted: i64,
    pub treasury_phase: i32,
    pub community_wallet_address: Option<String>,
    pub operational_wallet_address: Option<String>,
    pub bear_future_active: bool,
}

/// GET /api/treasury — live treasury balances from platform_settings.
pub async fn treasury(State(db): State<SupabaseClient>) -> Result<Json<TreasurySummary>, AppError> {
    let repo = SupabaseBearFutureRepository::new(db);
    let s = repo.treasury_settings().await?;
    let num = |k: &str, default: f64| s.get(k).and_then(|v| v.parse().ok()).unwrap_or(default);

    Ok(Json(TreasurySummary {
        community_treasury_ada: num("treasury_balance_ada", 0.0),
        operational_wallet_ada: num("operational_balance_ada", 0.0),
        governance_token_total_minted: s
            .get("governance_token_total_minted")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        treasury_phase: s
            .get("treasury_phase")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        community_wallet_address: s.get("treasury_wallet_ada").cloned(),
        operational_wallet_address: s.get("operational_wallet_ada").cloned(),
        bear_future_active: s
            .get("bear_future_active")
            .map(|v| v == "true")
            .unwrap_or(false),
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

/// GET /api/bear-future/token-holders — verified NORTH holders (no wallets).
pub async fn token_holders(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<PublicTokenHolder>>, AppError> {
    let repo = SupabaseBearFutureRepository::new(db);
    Ok(Json(repo.token_holders().await?))
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
