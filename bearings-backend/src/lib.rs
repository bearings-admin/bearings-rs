//! bearings-backend — library crate.
//!
//! Holds the entire application: configuration, the Supabase client, the typed
//! REST API handlers (`routes`), and the server-side rendered zones (`ssr`).
//! The binary (`main.rs`) is a thin wrapper that wires env -> config -> server.
//!
//! Layering (mirrors a route -> repository -> client flow):
//!   routes/ssr  ->  repositories  ->  db::SupabaseClient  ->  Supabase PostgREST
//!
//! Tests:
//!   cargo test -p bearings-backend --lib     # unit tests (no network)
//!   cargo test -p bearings-backend           # + tests/api_tests.rs (needs SUPABASE_URL)

pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod i18n;
pub mod llms;
pub mod mcp;
pub mod middleware;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod ssr;
pub mod ui;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Construct the Axum router with all routes and middleware wired in.
///
/// Extracted so integration tests can spin up the app without binding a port:
/// ```ignore
/// let db = bearings_backend::db::SupabaseClient::from_env()?;
/// let server = axum_test::TestServer::new(bearings_backend::build_app(db))?;
/// ```
pub fn build_app(db: db::SupabaseClient) -> Router {
    Router::new()
        // ── Primary zone dispatcher ─────────────────────────────
        // GET / with ?zone= query param — all zones served from one handler.
        .route("/", get(ssr::root))
        // ── Legacy named paths — backwards compatibility ────────
        // Delegate to the same zone functions as the dispatcher.
        .route("/coming-up", get(ssr::coming_up_page))
        .route("/history", get(ssr::history_page))
        .route("/bear-future", get(ssr::bear_future_page))
        .route("/events", get(ssr::events_page))
        .route("/places", get(ssr::places_page))
        .route("/clubs", get(ssr::clubs_page))
        .route("/titles", get(ssr::titles_page))
        .route("/creators", get(ssr::creators_page))
        .route("/campaigns", get(ssr::campaigns_page))
        .route("/digital-spaces", get(ssr::digital_spaces_page))
        // ── Events API ──────────────────────────────────────────
        .route("/api/events", get(routes::events::list))
        .route("/api/events/by-month", get(routes::events::by_month))
        .route("/api/events/:id", get(routes::events::get_one))
        // ── Places API ──────────────────────────────────────────
        .route("/api/places", get(routes::places::list))
        .route("/api/places/nearby", get(routes::places::nearby))
        .route("/api/places/:id", get(routes::places::get_one))
        // ── Clubs API ───────────────────────────────────────────
        .route("/api/clubs", get(routes::clubs::list))
        .route("/api/clubs/:id", get(routes::clubs::get_one))
        // ── Title holders API ───────────────────────────────────
        .route("/api/title-holders", get(routes::titles::list))
        .route("/api/title-holders/current", get(routes::titles::current))
        // ── Competitions API ────────────────────────────────────
        .route("/api/competitions", get(routes::competitions::list))
        // ── Bear history API ────────────────────────────────────
        .route("/api/bear-history", get(routes::history::list))
        // ── Campaigns API ───────────────────────────────────────
        .route("/api/campaigns", get(routes::campaigns::list))
        // ── Composite zone endpoints ────────────────────────────
        .route("/api/now", get(routes::now::feed))
        .route("/api/coming-up", get(routes::coming_up::feed))
        // ── Bear Future API ─────────────────────────────────────
        .route("/api/treasury", get(routes::bear_future::treasury))
        .route("/api/bear-future", get(routes::bear_future::proposals))
        .route("/api/bear-future/funded", get(routes::bear_future::funded))
        .route(
            "/api/bear-future/token-holders",
            get(routes::bear_future::token_holders),
        )
        .route("/api/bear-future/ledger", get(routes::bear_future::ledger))
        // ── Creators API ────────────────────────────────────────
        .route("/api/creators", get(routes::creators::list))
        .route("/api/creators/:id", get(routes::creators::get_one))
        // ── Digital spaces API ──────────────────────────────────
        .route("/api/digital-spaces", get(routes::digital_spaces::list))
        .route(
            "/api/digital-spaces/:id",
            get(routes::digital_spaces::get_one),
        )
        // ── Stories API ─────────────────────────────────────────
        .route("/api/stories", get(routes::stories::list))
        .route("/api/stories/:id", get(routes::stories::get_one))
        // ── Inclusion flags API ─────────────────────────────────
        .route("/api/inclusion-flags", get(routes::flags::list_flags))
        .route("/api/events/flagged", get(routes::flags::flagged_events))
        // ── Governance voting ───────────────────────────────────
        .route("/api/bear-future/vote", post(routes::votes::cast))
        // ── iCal export ─────────────────────────────────────────
        .route("/api/events/ical.ics", get(routes::ical::export))
        .route(
            "/api/future-ideas/:id/upvote",
            post(routes::future_ideas::upvote),
        )
        // ── Submissions ─────────────────────────────────────────
        .route("/api/submissions", post(routes::submissions::create))
        // ── AI crawlability ─────────────────────────────────────
        .route("/llms.txt", get(llms::llms_txt))
        .route("/llms-full.txt", get(llms::llms_full_txt))
        .route("/robots.txt", get(llms::robots_txt))
        .route("/mcp", post(mcp::mcp_handler).get(mcp::mcp_get))
        .route("/style.css", get(stylesheet_css))
        // ── Utility ─────────────────────────────────────────────
        .route("/health", get(health))
        // ── Middleware ──────────────────────────────────────────
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(db)
}

/// GET /health — used by systemd and uptime monitors.
async fn health() -> &'static str {
    "ok"
}

/// GET /style.css — the shared stylesheet, cached by the browser for an hour
/// (was previously re-sent inline in every page).
async fn stylesheet_css() -> impl axum::response::IntoResponse {
    (
        [
            (axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (axum::http::header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        crate::ui::stylesheet(),
    )
}
