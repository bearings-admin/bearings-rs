
//! x402 Protocol — autonomous agent payment execution.
//!
//! The x402 Protocol launched on Cardano mainnet in April 2026.
//! It enables AI agents to hold Cardano wallets and execute payments
//! independently — no human signature required for routine transactions.
//!
//! ## IMPORTANT: This module is Phase 4 ONLY.
//!
//! Current treasury_phase = 1 (steward manual).
//! This module MUST NOT execute any payments until:
//!   1. governance_vote passes to advance treasury_phase to 4
//!   2. The vote result is written to platform_settings
//!   3. The steward has confirmed the transition in writing
//!
//! The check_phase_before_payment() guard enforces this at runtime.
//!
//! ## What Phase 4 enables
//!
//! Routine operational payments only:
//!   - Anthropic API: monthly, ~$20-50 depending on research volume
//!   - Hostinger VPS: monthly, ~$5-10
//!   - Blockfrost API: monthly, free tier (0 ADA)
//!   - Domain renewal: annual
//!
//! Community treasury releases ALWAYS require governance vote + steward
//! co-signature regardless of phase. x402 never touches the community wallet.

use crate::{error::AgentError, supabase::SupabaseWriter};

/// Operational payment categories allowed in Phase 4 autonomous mode.
#[derive(Debug, Clone, serde::Serialize)]
pub enum PaymentCategory {
    AnthropicApi,
    VpsHosting,
    DomainRenewal,
    BlockfrostApi,
}

impl PaymentCategory {
    /// Maximum ADA amount this category can spend per transaction.
    /// Hard-coded ceiling — governance vote needed to raise these limits.
    pub fn max_ada(&self) -> f64 {
        match self {
            PaymentCategory::AnthropicApi  => 100.0,
            PaymentCategory::VpsHosting    => 50.0,
            PaymentCategory::DomainRenewal => 50.0,
            PaymentCategory::BlockfrostApi => 0.0,   // Free tier
        }
    }

    pub fn description(&self) -> &str {
        match self {
            PaymentCategory::AnthropicApi  => "Anthropic API — research agent compute",
            PaymentCategory::VpsHosting    => "Hostinger VPS — backend + agent hosting",
            PaymentCategory::DomainRenewal => "Domain renewal",
            PaymentCategory::BlockfrostApi => "Blockfrost — Cardano wallet monitoring",
        }
    }
}

/// Guard: check treasury_phase before any payment attempt.
/// Returns Err if phase < 4. Called at the top of every payment function.
pub async fn check_phase_before_payment(
    db: &SupabaseWriter,
) -> Result<(), AgentError> {
    let settings: Vec<serde_json::Value> = db
        .get("platform_settings?key=eq.treasury_phase&select=value")
        .await?;

    let phase: u8 = settings
        .first()
        .and_then(|s| s["value"].as_str())
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    if phase < 4 {
        return Err(AgentError::Supabase(format!(
            "Payment blocked: treasury_phase is {} — Phase 4 required for autonomous payments. \
             Current phase requires steward manual authorization.",
            phase
        )));
    }

    Ok(())
}

// ── PHASE 4 STUB: x402 payment execution ─────────────────────
//
// TODO (Phase 4 — after governance vote):
//
// pub struct X402Wallet {
//     address: String,
//     // Private key loaded from VPS secure keystore
//     // NEVER logged, NEVER serialized, NEVER transmitted
// }
//
// pub async fn execute_operational_payment(
//     wallet: &X402Wallet,
//     category: PaymentCategory,
//     amount_ada: f64,
//     recipient: &str,
//     db: &SupabaseWriter,
// ) -> Result<String, AgentError> {
//     // 1. Guard: check_phase_before_payment()
//     // 2. Guard: amount_ada <= category.max_ada()
//     // 3. Guard: category is NOT community treasury
//     // 4. Build Cardano transaction via x402 SDK
//     // 5. Sign with operational wallet key
//     // 6. Submit to Cardano mainnet
//     // 7. Log to operational_ledger with tx_hash
//     // 8. Update platform_settings last_payment_date
// }
//
// Resources:
//   x402 Protocol: https://x402.org
//   Cardano wallet signing: cardano-serialization-lib
//   Agent SDK: masumi-network/adk-rust

/// Log a payment to the operational ledger (used in Phases 1-3 for manual payments,
/// and in Phase 4 for autonomous payments).
pub async fn log_operational_payment(
    db: &SupabaseWriter,
    category: PaymentCategory,
    amount_ada: f64,
    tx_hash: Option<&str>,
    authorized_by: &str,
) -> Result<(), AgentError> {
    let entry = serde_json::json!({
        "direction": "out",
        "amount_ada": amount_ada,
        "vendor": category.description(),
        // Use explicit snake_case names — Debug repr has no separators
        "category": match &category {
            PaymentCategory::AnthropicApi  => "anthropic_api",
            PaymentCategory::VpsHosting    => "vps_hosting",
            PaymentCategory::DomainRenewal => "domain_renewal",
            PaymentCategory::BlockfrostApi => "blockfrost_api",
        },
        "description": format!("Operational payment: {}", category.description()),
        "tx_hash": tx_hash,
        "authorized_by": authorized_by,
        "tx_date": chrono::Utc::now().to_rfc3339(),
    });

    db.insert("operational_ledger", &entry).await?;
    Ok(())
}
