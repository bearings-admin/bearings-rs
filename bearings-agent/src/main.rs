//! bearings-agent — treasury monitor and research scheduler
//! Runs as a systemd service on the VPS. Never holds private keys.
//!
//! Schedule:
//!   Every hour  — check for new inbound transactions on both wallets
//!   Monday 03:00 UTC — update balance snapshot in platform_settings

#![allow(dead_code)] // Path-B (Cardano/NORTH) scaffolding — wired as governance phases activate

mod blockfrost;
mod bluesky; // Bluesky social publishing — CONST-10, steward review required
mod error;
mod north_token; // NORTH token minting — Phase 1 manual, Phase 4 autonomous
mod supabase;
mod treasury;
mod wallet_onboarding; // Embedded Cardano wallet — email-only onboarding
mod x402; // x402 autonomous payments — Phase 4 ONLY

// Chrono traits must be explicitly imported to use .weekday() and .hour()
use chrono::{Datelike, Timelike, Utc};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("bearings-agent starting");

    let config = AgentConfig::from_env()?;
    let db = supabase::SupabaseWriter::from_env()?;
    let cardano = blockfrost::BlockfrostClient::from_env()?;

    tracing::info!(
        "Watching treasury wallet:     {}",
        &config.treasury_wallet[..20]
    );
    tracing::info!(
        "Watching operational wallet:  {}",
        &config.operational_wallet[..20]
    );

    loop {
        let now = Utc::now();

        // ── Hourly: check for new transactions ──────────────
        match treasury::check_inbound(&cardano, &db, &config).await {
            Ok(0) => tracing::debug!("Treasury check: no new transactions"),
            Ok(count) => tracing::info!("Logged {} new treasury transactions", count),
            Err(e) => tracing::error!("Treasury check failed: {}", e),
        }

        // ── Weekly Monday 03:xx UTC: update balance snapshot ─
        if now.weekday() == chrono::Weekday::Mon && now.hour() == 3 {
            match treasury::update_balances(&cardano, &db, &config).await {
                Ok(_) => tracing::info!("Weekly balance snapshot updated"),
                Err(e) => tracing::error!("Balance update failed: {}", e),
            }
        }

        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}

/// Runtime configuration — loaded once from environment at startup.
pub struct AgentConfig {
    pub treasury_wallet: String,
    pub operational_wallet: String,
}

impl AgentConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        use anyhow::Context;
        Ok(Self {
            treasury_wallet: std::env::var("TREASURY_WALLET_ADDRESS")
                .context("TREASURY_WALLET_ADDRESS not set")?,
            operational_wallet: std::env::var("OPERATIONAL_WALLET_ADDRESS")
                .context("OPERATIONAL_WALLET_ADDRESS not set")?,
        })
    }
}
