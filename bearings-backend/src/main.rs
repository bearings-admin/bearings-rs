
//! bearings-backend — Axum HTTP server
//! Mirrors Gaspar's architectural approach: minimal setup, explicit routes, no magic.
//! Run: cargo run -p bearings-backend
//! Port: 3000 (set PORT env var to override)
//!
//! Route map (v2 — ?zone= dispatcher):
//!   ── Primary zone dispatcher (ssr.rs v2) ──────────────────────
//!   GET /                         → ssr::root            (?zone=now default)
//!   GET /?zone=now                → NOW zone (hot events, venues, campaigns, titles)
//!   GET /?zone=coming-up          → COMING UP (trip planner + iCal)
//!   GET /?zone=archive            → BEAR ARCHIVES (decade tabs)
//!   GET /?zone=archive&decade=X   → decade-filtered archive
//!   GET /?zone=future             → BEAR FUTURE (treasury + NORTH governance)
//!   GET /?zone=places             → Places directory
//!   GET /?zone=events             → Events list
//!   GET /?zone=clubs              → Clubs directory
//!   GET /?zone=titles             → Title holders
//!   GET /?zone=creators           → Creators
//!   GET /?zone=campaigns          → Campaigns
//!   GET /?zone=digital-spaces     → Digital spaces
//!
//!   ── Legacy named paths (kept for Gaspar review; delegate to root) ──
//!   GET /coming-up                → ssr::coming_up_page
//!   GET /history                  → ssr::history_page
//!   GET /bear-future              → ssr::bear_future_page
//!   GET /events                   → ssr::events_page
//!   GET /places                   → ssr::places_page
//!   GET /clubs                    → ssr::clubs_page
//!   GET /titles                   → ssr::titles_page
//!   GET /creators                 → ssr::creators_page
//!   GET /campaigns                → ssr::campaigns_page
//!   GET /digital-spaces           → ssr::digital_spaces_page
//!
//!   ── REST API (JSON) ──────────────────────────────────────────
//!   GET /api/events               → events::list
//!   GET /api/events/by-month      → events::by_month   (static before param)
//!   GET /api/events/:id           → events::get_one
//!   GET /api/places               → places::list
//!   GET /api/places/nearby        → places::nearby     (static before param)
//!   GET /api/places/:id           → places::get_one
//!   GET /api/clubs                → clubs::list
//!   GET /api/clubs/:id            → clubs::get_one
//!   GET /api/title-holders        → titles::list
//!   GET /api/title-holders/current → titles::current
//!   GET /api/competitions         → competitions::list
//!   GET /api/bear-history         → history::list
//!   GET /api/campaigns            → campaigns::list
//!   GET /api/now                  → now::feed          (composite JSON)
//!   GET /api/coming-up            → coming_up::feed    (composite JSON)
//!   GET /api/treasury             → bear_future::treasury
//!   GET /api/bear-future          → bear_future::proposals
//!   GET /api/bear-future/funded   → bear_future::funded
//!   GET /api/bear-future/token-holders → bear_future::token_holders
//!   GET /api/bear-future/ledger   → bear_future::ledger
//!   GET /api/creators             → creators::list
//!   GET /api/creators/:id         → creators::get_one
//!   GET /api/digital-spaces       → digital_spaces::list
//!   GET /api/digital-spaces/:id   → digital_spaces::get_one
//!   GET /api/stories              → stories::list
//!   GET /api/stories/:id          → stories::get_one
//!   GET /api/inclusion-flags      → flags::list_flags
//!   GET /api/events/flagged       → flags::flagged_events
//!   POST /api/bear-future/vote    → votes::cast
//!   GET /api/events/ical.ics      → ical::export
//!   POST /api/submissions         → submissions::create
//!
//!   ── AI discoverability ────────────────────────────────────────
//!   GET /llms.txt                 → llms::llms_txt
//!   GET /llms-full.txt            → llms::llms_full_txt
//!
//!   ── Utility ───────────────────────────────────────────────────
//!   GET /health                   → health check
//!   GET /hello/:name              → hello::handler (Gaspar's original — preserved)

mod config;
mod db;
mod error;
mod middleware;
mod routes;
mod i18n;
mod ssr;
mod llms;

use axum::{Router, routing::{get, post}};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env — no-op in production (env vars set directly by systemd EnvironmentFile)
    dotenvy::dotenv().ok();

    // Structured logging — RUST_LOG=bearings_backend=debug for verbose output
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Validate all config at startup — fail fast with clear error messages
    let mut config = config::Config::from_env()?;

    // Supabase client — shared across all route handlers via Axum state
    let db = db::SupabaseClient::from_env()?;

    // Load live feature flags from platform_settings
    let temp_client = reqwest::Client::new();
    config.load_feature_flags(&temp_client, &db.anon_key).await;
    config.log_startup();

    let app = Router::new()
        // ── Primary zone dispatcher (v2) ────────────────────────
        // GET / with ?zone= query param — all zones served from one handler.
        // Every internal nav link, "View all" link, and decade tab uses /?zone=X.
        .route("/",              get(ssr::root))
        // ── Legacy named paths (kept while Gaspar confirms ?zone= routing) ──
        // These delegate to the same zone functions via legacy wrappers in ssr.rs.
        // Safe to remove once the VPS is running v2 and nav is confirmed working.
        .route("/coming-up",     get(ssr::coming_up_page))
        .route("/history",       get(ssr::history_page))
        .route("/bear-future",   get(ssr::bear_future_page))
        .route("/events",        get(ssr::events_page))
        .route("/places",        get(ssr::places_page))
        .route("/clubs",         get(ssr::clubs_page))
        .route("/titles",        get(ssr::titles_page))
        .route("/creators",      get(ssr::creators_page))
        .route("/campaigns",     get(ssr::campaigns_page))
        .route("/digital-spaces",get(ssr::digital_spaces_page))
        // ── Events API ──────────────────────────────────────────
        .route("/api/events",                    get(routes::events::list))
        .route("/api/events/by-month",           get(routes::events::by_month))
        .route("/api/events/:id",                get(routes::events::get_one))
        // ── Places API ──────────────────────────────────────────
        .route("/api/places",                    get(routes::places::list))
        .route("/api/places/nearby",             get(routes::places::nearby))
        .route("/api/places/:id",                get(routes::places::get_one))
        // ── Clubs API ───────────────────────────────────────────
        .route("/api/clubs",                     get(routes::clubs::list))
        .route("/api/clubs/:id",                 get(routes::clubs::get_one))
        // ── Title holders API ───────────────────────────────────
        .route("/api/title-holders",             get(routes::titles::list))
        .route("/api/title-holders/current",     get(routes::titles::current))
        // ── Competitions API ─────────────────────────────────────
        .route("/api/competitions",              get(routes::competitions::list))
        // ── Bear history API ─────────────────────────────────────
        .route("/api/bear-history",              get(routes::history::list))
        // ── Campaigns API ───────────────────────────────────────
        .route("/api/campaigns",                 get(routes::campaigns::list))
        // ── Composite zone endpoints ─────────────────────────────
        .route("/api/now",                       get(routes::now::feed))
        .route("/api/coming-up",                 get(routes::coming_up::feed))
        // ── Bear Future API ──────────────────────────────────────
        .route("/api/treasury",                  get(routes::bear_future::treasury))
        .route("/api/bear-future",               get(routes::bear_future::proposals))
        .route("/api/bear-future/funded",        get(routes::bear_future::funded))
        .route("/api/bear-future/token-holders", get(routes::bear_future::token_holders))
        .route("/api/bear-future/ledger",        get(routes::bear_future::ledger))
        // ── Creators API ─────────────────────────────────────────
        .route("/api/creators",                  get(routes::creators::list))
        .route("/api/creators/:id",              get(routes::creators::get_one))
        // ── Digital spaces API ───────────────────────────────────
        .route("/api/digital-spaces",            get(routes::digital_spaces::list))
        .route("/api/digital-spaces/:id",        get(routes::digital_spaces::get_one))
        // ── Stories API ──────────────────────────────────────────
        .route("/api/stories",                   get(routes::stories::list))
        .route("/api/stories/:id",               get(routes::stories::get_one))
        // ── Inclusion flags API ───────────────────────────────────
        .route("/api/inclusion-flags",           get(routes::flags::list_flags))
        .route("/api/events/flagged",            get(routes::flags::flagged_events))
        // ── Governance voting ────────────────────────────────────
        .route("/api/bear-future/vote",          post(routes::votes::cast))
        // ── iCal export ─────────────────────────────────────────
        .route("/api/events/ical.ics",           get(routes::ical::export))
        .route("/api/future-ideas/:id/upvote",   post(routes::future_ideas::upvote))
        // ── Submissions ──────────────────────────────────────────
        .route("/api/submissions",               post(routes::submissions::create))
        // ── AI crawlability ─────────────────────────────────────
        .route("/llms.txt",                      get(llms::llms_txt))
        .route("/llms-full.txt",                 get(llms::llms_full_txt))
        // ── Utility ─────────────────────────────────────────────
        .route("/health",                        get(health))
        .route("/hello/:name",                   get(routes::hello::handler))
        // ── Middleware ──────────────────────────────────────────
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(db);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("bearings-backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// GET /health — used by systemd and uptime monitors
async fn health() -> &'static str {
    "ok"
}
