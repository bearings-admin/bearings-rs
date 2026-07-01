//! bearings-backend — binary entry point.
//!
//! Thin wrapper: load env, validate config, construct the Supabase client,
//! build the router (see `bearings_backend::build_app`), and serve.
//! All application logic lives in the library crate.
//!
//! Run: cargo run -p bearings-backend   (PORT env var overrides the default 3000)

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
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

    // ── Rate limiting (DoS) ─────────────────────────────────────
    // Per-IP token bucket over the whole surface (public API, /mcp, SSR). Applied
    // here in main (not build_app) so the api_tests keep exercising handlers without
    // a peer IP. Generous for a human + HTMX (10 req/s sustained, burst 60); a single
    // IP flooding the box is capped. Over-limit requests get 429 + Retry-After.
    //
    // SmartIpKeyExtractor reads the real client IP from `X-Forwarded-For` — required
    // because behind the Caddy reverse proxy the TCP peer is always 127.0.0.1, which
    // would otherwise lump every visitor into one shared bucket. (Anti-spoof note: for
    // full robustness Caddy should overwrite X-Forwarded-For; tune buckets via these
    // two numbers if shared-NAT clients trip the limit.)
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(100) // replenish 1 cell / 100ms ≈ 10 req/s sustained
            .burst_size(60)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("valid governor config"),
    );

    // Evict idle per-IP buckets so memory doesn't grow unbounded.
    let limiter = governor_conf.limiter().clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(60));
        limiter.retain_recent();
    });

    // Warm + periodically refresh the content-translation cache (Layer-2 i18n).
    bearings_backend::spawn_content_refresh(db.clone());

    let app = build_app(db).layer(GovernorLayer {
        config: governor_conf,
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("bearings-backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    // `into_make_service_with_connect_info` so the limiter can fall back to the peer
    // IP if no forwarded header is present.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
