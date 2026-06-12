//! Treasury monitoring — read-only Cardano wallet surveillance.
//! This module NEVER holds private keys.
//! It reads from Blockfrost and writes to Supabase.
//!
//! The key logic: for each new transaction, we calculate how much
//! ADA was NET RECEIVED by our wallet (outputs to us minus inputs from us).
//! This correctly handles change outputs and complex transactions.

use crate::AgentConfig;
use crate::{blockfrost::BlockfrostClient, error::AgentError, supabase::SupabaseWriter};
use bearings_shared::models::OperationalLedger;
use chrono::{TimeZone, Utc};

/// Check both wallets for new inbound transactions.
/// Returns the count of new transactions logged.
pub async fn check_inbound(
    cardano: &BlockfrostClient,
    db: &SupabaseWriter,
    config: &AgentConfig,
) -> Result<usize, AgentError> {
    let mut count = 0;

    for (wallet, wallet_name) in [
        (&config.treasury_wallet, "community_treasury"),
        (&config.operational_wallet, "operational_wallet"),
    ] {
        let txs = cardano.recent_transactions(wallet, 10).await?;

        for tx in &txs {
            if db.ledger_entry_exists(&tx.tx_hash).await? {
                continue; // Already logged
            }

            // Fetch UTXO detail to calculate the actual ADA amount
            let utxos = match cardano.tx_utxos(&tx.tx_hash).await {
                Ok(u) => u,
                Err(e) => {
                    tracing::warn!("Could not fetch UTXOs for {}: {}", tx.tx_hash, e);
                    continue;
                }
            };

            let net_ada = cardano.net_received(&utxos, wallet);
            if net_ada <= 0.0 {
                continue; // This was an outgoing transaction
            }

            let tx_time = Utc
                .timestamp_opt(tx.block_time, 0)
                .single()
                .unwrap_or_else(|| Utc::now()); // closure required — Utc::now is fn, not value

            let entry = OperationalLedger {
                id: 0, // Set by database
                tx_date: Some(tx_time.date_naive()),
                direction: "in".to_string(),
                amount_ada: Some(net_ada),
                amount_usd: None,
                vendor: Some("Community donation".to_string()),
                category: Some(format!("{}_inflow", wallet_name)),
                description: Some(format!("Inbound {} ADA to {} wallet", net_ada, wallet_name)),
                tx_hash: Some(tx.tx_hash.clone()),
                authorized_by: Some("cardano_network".to_string()),
                ..Default::default()
            };

            db.insert_ledger_entry(&entry).await?;
            count += 1;

            tracing::info!(
                "Logged inbound transaction: {} ADA to {} ({})",
                net_ada,
                wallet_name,
                &tx.tx_hash[..8]
            );
        }
    }

    Ok(count)
}

/// Update treasury_balance_ada and operational_balance_ada in platform_settings.
/// Called weekly on Monday mornings.
pub async fn update_balances(
    cardano: &BlockfrostClient,
    db: &SupabaseWriter,
    config: &AgentConfig,
) -> Result<(), AgentError> {
    let treasury_ada = cardano.wallet_balance_ada(&config.treasury_wallet).await?;
    let operational_ada = cardano
        .wallet_balance_ada(&config.operational_wallet)
        .await?;

    db.update_platform_setting("treasury_balance_ada", &treasury_ada.to_string())
        .await?;
    db.update_platform_setting("operational_balance_ada", &operational_ada.to_string())
        .await?;

    tracing::info!(
        "Balances updated — treasury: {:.6} ADA, operational: {:.6} ADA",
        treasury_ada,
        operational_ada
    );

    Ok(())
}
