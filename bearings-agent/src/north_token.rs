//! NORTH token governance.
//!
//! The NORTH token is a Cardano native asset. More NORTH = more sway
//! over community direction. Follow the NORTH.
//!
//! Token name: NORTH
//! Chain: Cardano mainnet
//! Distribution: 1 NORTH per verified role
//! Non-transferable during bootstrapping
//! DAO threshold: 100 verified holders
//!
//! ## Phase 1 (current): Manual minting
//! Steward verifies a role claim, manually mints NORTH to the
//! contributor's custodial wallet. This module logs the request
//! and updates the governance_token_holders table.
//!
//! ## Phase 4 (planned): Autonomous minting via x402
//! Once governance approves, the agent mints NORTH autonomously
//! when a verification request passes all criteria.

use crate::{error::AgentError, supabase::SupabaseWriter};
use serde::{Deserialize, Serialize};

/// A NORTH token minting request — created when a bear submits
/// their role for verification.
#[derive(Debug, Serialize, Deserialize)]
pub struct MintRequest {
    pub contributor_id: i64,
    pub display_name: String,
    pub claimed_role: String,
    pub evidence_url: Option<String>, // Link to public record (BWM article, competition page)
    pub wallet_address: Option<String>, // Custodial wallet created by onboarding flow
    pub requested_at: chrono::DateTime<chrono::Utc>,
}

/// Result of a Phase 1 manual mint operation.
#[derive(Debug, Serialize)]
pub struct MintResult {
    pub contributor_id: i64,
    pub north_balance: i32,
    pub tx_hash: Option<String>, // Cardano tx hash — set after manual mint
    pub phase: u8,
}

/// Phase 1: Log a mint request to the governance_token_holders table.
/// The steward reviews the request and executes the mint manually.
/// Updates token_balance and marks verified = true after steward action.
pub async fn log_mint_request(
    db: &SupabaseWriter,
    request: &MintRequest,
) -> Result<(), AgentError> {
    // In Phase 1, this creates a governance_token_holders row with
    // verified = false. The steward sets verified = true manually
    // after checking the claimed role against public records.
    // Schema: governance_token_holders uses cardano_wallet (single field)
    // Not custodial_wallet/self_custody_wallet — those were from an older design
    let body = serde_json::json!({
        "display_name": request.display_name,
        "contributor_tier": "community",
        "verified_role_description": request.claimed_role,
        "token_balance": 0,
        "verified": false,
    });

    db.insert("governance_token_holders", &body).await?;

    tracing::info!(
        "Mint request logged for {} — awaiting steward verification",
        request.display_name
    );

    Ok(())
}

/// Phase 1: Steward confirms a mint — called after manual Cardano transaction.
/// Updates the holder record with verified = true and token_balance = 1.
pub async fn confirm_mint(
    db: &SupabaseWriter,
    contributor_id: i64,
    tx_hash: &str,
) -> Result<MintResult, AgentError> {
    let path = format!("governance_token_holders?id=eq.{}", contributor_id);
    let body = serde_json::json!({
        "verified": true,
        "token_balance": 1,
        "contributor_tier": "verified_contributor",
    });

    db.patch(&path, &body).await?;

    tracing::info!(
        "NORTH token minted — contributor {} — tx: {}",
        contributor_id,
        tx_hash.get(..12).unwrap_or(tx_hash) // safe: no panic on short hashes
    );

    Ok(MintResult {
        contributor_id,
        north_balance: 1,
        tx_hash: Some(tx_hash.to_string()),
        phase: 1,
    })
}

/// Check whether the DAO threshold (100 verified holders) has been reached.
/// When reached, logs a milestone to bear_history and notifies the steward.
pub async fn check_dao_threshold(db: &SupabaseWriter) -> Result<bool, AgentError> {
    let count: Vec<serde_json::Value> = db
        .get("governance_token_holders?select=id&verified=eq.true")
        .await?;

    let verified_count = count.len();
    let threshold = 100; // From platform_settings.governance_dao_threshold

    if verified_count >= threshold {
        tracing::info!(
            "DAO THRESHOLD REACHED: {} verified NORTH holders — full governance unlocked",
            verified_count
        );
        // TODO: notify steward, update platform_settings, log bear_history milestone
        return Ok(true);
    }

    tracing::debug!(
        "NORTH holders: {}/{} — {} remaining until full governance",
        verified_count,
        threshold,
        threshold.saturating_sub(verified_count) // safe: no panic if somehow over threshold
    );

    Ok(false)
}

// ── PHASE 2 STUB: Multi-signature ────────────────────────────
//
// When treasury_phase advances to 2:
// - Two of three designated keyholders must co-sign mints
// - Keyholders are elected by NORTH vote
// - This module submits a co-sign request and awaits threshold
//
// TODO (Phase 2):
// pub async fn request_multisig_mint(...) -> Result<(), AgentError> {
//     // Submit to co-sign queue
//     // Wait for 2-of-3 signatures
//     // Execute Cardano transaction
// }

// ── PHASE 4 STUB: Autonomous minting via x402 ────────────────
//
// When treasury_phase advances to 4 and governance has voted:
// - Agent autonomously mints NORTH after verification passes all criteria
// - Uses x402 Protocol on Cardano mainnet
// - Every mint is logged on-chain and mirrored to operational_ledger
//
// TODO (Phase 4):
// pub async fn autonomous_mint(
//     cardano: &BlockfrostClient,
//     wallet: &X402Wallet,
//     request: &MintRequest,
// ) -> Result<MintResult, AgentError> {
//     // Verify claim against public records (BWM, competition DB)
//     // Build Cardano transaction
//     // Sign with x402 wallet
//     // Submit to chain
//     // Log to operational_ledger
// }
