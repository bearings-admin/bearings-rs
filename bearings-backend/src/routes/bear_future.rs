
//! Bear Future — the governance and treasury layer.
//! These routes power the Bear Future zone in the frontend.
//! All reads are public. Writes (voting, submitting proposals)
//! require authentication — to be implemented in a future phase.

use axum::{extract::{Query, State}, Json};
use bearings_shared::models::{
    BearFutureProposal, OperationalLedger
};
use serde::{Deserialize, Serialize};
use crate::{db::SupabaseClient, error::AppError};

// ── TREASURY DISPLAY ──────────────────────────────────────────

/// Treasury summary read from platform_settings.
/// Displayed in the Bear Future "The Pot" section.
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

/// GET /api/treasury
/// Returns live treasury balances from platform_settings.
/// Updated weekly by the treasury agent via Blockfrost.
pub async fn treasury(
    State(db): State<SupabaseClient>,
) -> Result<Json<TreasurySummary>, AppError> {
    // Keys verified against platform_settings table 2026-06-06
    // treasury_wallet_ada / operational_wallet_ada store the Cardano addresses
    // treasury_balance_ada / operational_balance_ada store the ADA amounts
    let url = format!(
        "{}/rest/v1/platform_settings?select=key,value&key=in.(treasury_balance_ada,operational_balance_ada,governance_token_name,governance_dao_threshold,treasury_phase,treasury_wallet_ada,operational_wallet_ada,bear_future_active)",
        db.url
    );

    #[derive(Deserialize)]
    struct Setting { key: String, value: Option<String> }

    let settings: Vec<Setting> = db.get_json(&url).await?;

    let get = |key: &str| -> Option<String> {
        settings.iter().find(|s| s.key == key)?.value.clone()
    };

    Ok(Json(TreasurySummary {
        community_treasury_ada: get("treasury_balance_ada")
            .and_then(|v| v.parse().ok()).unwrap_or(0.0),
        operational_wallet_ada: get("operational_balance_ada")
            .and_then(|v| v.parse().ok()).unwrap_or(0.0),
        governance_token_total_minted: get("governance_token_total_minted")
            .and_then(|v| v.parse().ok()).unwrap_or(0),
        treasury_phase: get("treasury_phase")
            .and_then(|v| v.parse().ok()).unwrap_or(1),
        community_wallet_address: get("treasury_wallet_ada"),
        operational_wallet_address: get("operational_wallet_ada"),
        bear_future_active: get("bear_future_active")
            .map(|v| v == "true").unwrap_or(false),
    }))
}

// ── PROPOSALS ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ProposalsQuery {
    pub status: Option<String>,  // draft | open | passed | failed | funded
    pub limit: Option<u32>,
}

/// GET /api/bear-future
/// Returns proposals — privacy_mode proposals return anonymised applicant info.
pub async fn proposals(
    State(db): State<SupabaseClient>,
    Query(params): Query<ProposalsQuery>,
) -> Result<Json<Vec<BearFutureProposal>>, AppError> {
    let limit = params.limit.unwrap_or(50);
    let mut url = format!(
        "{}/rest/v1/bear_future_proposals?select=*&order=created_at.desc&limit={}",
        db.url, limit
    );
    if let Some(s) = params.status { url.push_str(&format!("&status=eq.{}", s)); }

    let proposals: Vec<BearFutureProposal> = db.get_json(&url).await?;

    // Redact sensitive fields on privacy_mode proposals
    let redacted = proposals.into_iter().map(|mut p| {
        if p.privacy_mode.unwrap_or(false) {
            p.receiving_wallet = Some("redacted — privacy mode".to_string());
        }
        p
    }).collect();

    Ok(Json(redacted))
}

/// GET /api/bear-future/funded
/// Returns funded proposals with on-chain proof (tx_hash).
pub async fn funded(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<BearFutureProposal>>, AppError> {
    let url = format!(
        "{}/rest/v1/bear_future_proposals?select=*&status=eq.funded&order=created_at.desc",
        db.url
    );
    Ok(Json(db.get_json(&url).await?))
}

// ── GOVERNANCE TOKEN HOLDERS ───────────────────────────────────

/// Public view of a NORTH token holder — wallet addresses stripped.
#[derive(Deserialize, Serialize)]
pub struct PublicTokenHolder {
    pub display_name: Option<String>,
    pub contributor_tier: Option<String>,
    pub verified_role_description: Option<String>,
    pub token_balance: Option<i32>,
    pub verified: Option<bool>,
}

/// GET /api/bear-future/token-holders
/// Returns public profile of verified NORTH token holders.
/// Wallet addresses are never exposed in the public endpoint.
pub async fn token_holders(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<PublicTokenHolder>>, AppError> {
    let url = format!(
        "{}/rest/v1/governance_token_holders?select=display_name,contributor_tier,verified_role_description,token_balance,verified&verified=eq.true&order=token_balance.desc",
        db.url
    );
    Ok(Json(db.get_json(&url).await?))
}

// ── OPERATIONAL LEDGER ─────────────────────────────────────────

/// GET /api/bear-future/ledger
/// Returns the public operational ledger — every ADA movement.
/// This is the transparency layer: the community can see exactly
/// what the platform spends money on.
pub async fn ledger(
    State(db): State<SupabaseClient>,
    Query(params): Query<LedgerQuery>,
) -> Result<Json<Vec<OperationalLedger>>, AppError> {
    let limit = params.limit.unwrap_or(50);
    let url = format!(
        "{}/rest/v1/operational_ledger?select=tx_date,direction,amount_ada,amount_usd,vendor,category,description,tx_hash&order=tx_date.desc&limit={}",
        db.url, limit
    );
    Ok(Json(db.get_json(&url).await?))
}

#[derive(Deserialize)]
pub struct LedgerQuery {
    pub limit: Option<u32>,
}
