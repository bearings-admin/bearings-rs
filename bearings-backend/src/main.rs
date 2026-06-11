//! bearings-backend — binary entry point.
//!
//! Thin wrapper: load env, validate config, construct the Supabase client,
//! build the router (see `bearings_backend::build_app`), and serve.
//! All application logic lives in the library crate.
//!
//! Run: cargo run -p bearings-backend   (PORT env var overrides the default 3000)

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bearings_backend::{build_app, config::Config, db::SupabaseClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env — no-op in production (env vars set by systemd EnvironmentFile).
    dotenvy::dotenv().ok();

    // Structured logging — RUST_LOG=bearings_backend=debug for verbose output.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Validate all config at startup — fail fast with clear error messages.
    let mut config = Config::from_env()?;

    // Supabase client — shared across all handlers via Axum state.
    let db = SupabaseClient::from_env()?;

    // Load live feature flags from platform_settings.
    let temp_client = reqwest::Client::new();
    config.load_feature_flags(&temp_client, &db.anon_key).await;
    config.log_startup();

    let app = build_app(db);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("bearings-backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
