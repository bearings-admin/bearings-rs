//! bearings-agent — research and Bluesky publishing scaffold.
//! Runs as a systemd service on the VPS. Never holds private keys.
//!
//! The treasury is a single Base/USDC wallet that the steward tops up and
//! spends manually, so there is no autonomous on-chain monitor here. Bluesky
//! posts always require steward review (CONST-10), so this binary has no
//! autonomous loop yet — it is the scaffold the research scheduler hangs off.

#![allow(dead_code)] // supabase/bluesky helpers are wired as the scheduler grows

mod bluesky; // Bluesky social publishing — CONST-10, steward review required
mod error;
mod supabase;

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

    tracing::info!("bearings-agent starting (manual-treasury mode; no autonomous loop)");
    Ok(())
}
