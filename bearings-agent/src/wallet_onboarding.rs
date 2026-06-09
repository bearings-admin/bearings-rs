
//! Embedded Cardano wallet onboarding.
//!
//! Design intent: a bear should be able to participate in governance
//! without knowing anything about cryptocurrency.
//!
//! Flow:
//!   1. Bear submits email + claimed role via the web form
//!   2. Custodial Cardano wallet is created server-side
//!   3. Steward verifies the role claim
//!   4. On approval: NORTH token minted to custodial wallet
//!   5. Bear receives email: "You have been verified. You can now vote."
//!   6. At any time: bear can connect own wallet (Eternl, Lace)
//!      and transfer their NORTH token to self-custody
//!
//! wallet_type field: "custodial" | "self-custody" | "both"
//!
//! Privacy: custodial_wallet and self_custody_wallet fields are
//! NEVER returned in the public API (see routes/bear_future.rs
//! PublicTokenHolder — wallet fields excluded).

use crate::{error::AgentError, supabase::SupabaseWriter};

/// A request to onboard a new governance participant.
#[derive(Debug, serde::Deserialize)]
pub struct OnboardingRequest {
    pub email: String,
    pub display_name: String,
    pub claimed_role: String,
    pub evidence_url: Option<String>,
}

/// Phase 1: Create a custodial wallet placeholder.
///
/// In Phase 1, actual Cardano wallet creation happens via the
/// cardano-wallet server running on the VPS. This function
/// records the intent and queues the wallet creation.
///
/// Full implementation requires:
///   - cardano-wallet HTTP API on VPS (port 8090)
///   - Wallet name: "bearings-contributor-{id}"
///   - Passphrase: generated, stored encrypted, not logged
///
/// TODO: integrate with cardano-wallet server when VPS is live
pub async fn create_custodial_wallet(
    db: &SupabaseWriter,
    contributor_id: i64,
) -> Result<String, AgentError> {
    // Phase 1: return a placeholder address
    // Phase 2: call cardano-wallet server to generate real wallet
    let placeholder_address = format!("custodial-pending-{}", contributor_id);

    let path = format!("governance_token_holders?id=eq.{}", contributor_id);
    // DB column is cardano_wallet (single field, not custodial/self_custody split)
    db.patch(&path, &serde_json::json!({
        "cardano_wallet": placeholder_address,
    })).await?;

    tracing::info!(
        "Custodial wallet placeholder created for contributor {}",
        contributor_id
    );

    Ok(placeholder_address)
}

/// Self-custody exit: bear connects their own Cardano wallet.
/// Updates their record from custodial to self-custody.
/// The NORTH token transfer happens on-chain separately.
pub async fn register_self_custody_wallet(
    db: &SupabaseWriter,
    contributor_id: i64,
    wallet_address: &str,
) -> Result<(), AgentError> {
    // Validate it looks like a Cardano address (starts with addr1)
    if !wallet_address.starts_with("addr1") {
        return Err(AgentError::Supabase(
            "Invalid Cardano address — must start with addr1".to_string()
        ));
    }

    let path = format!("governance_token_holders?id=eq.{}", contributor_id);
    // DB column is cardano_wallet — self-custody replaces custodial address
    db.patch(&path, &serde_json::json!({
        "cardano_wallet": wallet_address,
    })).await?;

    tracing::info!(
        "Self-custody wallet registered for contributor {} — NORTH transfer pending",
        contributor_id
    );

    Ok(())
}

// ── PHASE 4 STUB: Privy embedded wallet integration ──────────
//
// Privy provides a polished embedded wallet SDK that handles
// the custodial → self-custody flow with a clean UX.
// When treasury_phase = 4, replace the placeholder above with:
//
// TODO (Phase 4):
// pub async fn create_privy_wallet(
//     privy_client: &PrivyClient,
//     email: &str,
// ) -> Result<String, AgentError> {
//     // Create embedded wallet via Privy API
//     // Returns wallet address automatically
//     // User never sees seed phrase or keys
//     // Can export to Eternl/Lace later via Privy export flow
// }
