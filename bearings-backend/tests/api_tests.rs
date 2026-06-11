//! HTTP integration tests for bearings-backend.
//! Standard tests/ integration layout — imports the library crate by name.
//!
//! Being inside src/ (not tests/) gives direct access to all crate internals
//! — no pub re-export machinery needed for a binary crate.
//!
//! Two test tiers:
//!   Unit tests (no env needed): src/ssr/mod.rs, src/ssr/query.rs
//!   HTTP tests (need env):      this file — skipped if SUPABASE_URL absent
//!
//! Run unit tests only:  cargo test -p bearings-backend --lib
//! Run everything:       SUPABASE_URL=... cargo test -p bearings-backend

use axum_test::TestServer;
use bearings_backend::build_app;

/// Returns a live TestServer when SUPABASE_URL is in the environment, or None.
/// Every HTTP test calls this first and returns early on None (graceful skip).
fn live_server() -> Option<TestServer> {
    dotenvy::dotenv().ok();
    if std::env::var("SUPABASE_URL").is_err() {
        eprintln!("SUPABASE_URL not set -- skipping HTTP integration test");
        return None;
    }
    let db = bearings_backend::db::SupabaseClient::from_env()
        .expect("env vars present but SupabaseClient::from_env() failed");
    Some(TestServer::new(build_app(db)).expect("TestServer::new failed"))
}

// ── Health ─────────────────────────────────────────────────────────────────────

/// GET /health must return 200 "ok". No Supabase needed.
/// Proves the app starts and routing is wired correctly.
#[tokio::test]
async fn health_returns_ok() {
    let Some(server) = live_server() else { return; };
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    resp.assert_text("ok");
}

// ── SSR zone smoke tests ───────────────────────────────────────────────────────

/// Every zone returns 200 HTML. Catches routing regressions and Supabase errors.
/// Gaspar's concern: a renamed zone string in Zone::parse breaks silently.
/// This test suite makes that a visible failure.
macro_rules! zone_smoke_test {
    ($name:ident, $zone:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(server) = live_server() else { return; };
            let resp = server.get(&format!("/?zone={}", $zone)).await;
            resp.assert_status_ok();
            let body = resp.text();
            assert!(
                body.contains("BEARINGS"),
                "?zone={} does not look like Bearings HTML -- first 200 chars:\n{}",
                $zone,
                &body[..200.min(body.len())]
            );
        }
    };
}

zone_smoke_test!(zone_now_returns_html,            "now");
zone_smoke_test!(zone_coming_up_returns_html,      "coming-up");
zone_smoke_test!(zone_archive_returns_html,        "archive");
zone_smoke_test!(zone_future_returns_html,         "future");
zone_smoke_test!(zone_places_returns_html,         "places");
zone_smoke_test!(zone_events_returns_html,         "events");
zone_smoke_test!(zone_clubs_returns_html,          "clubs");
zone_smoke_test!(zone_titles_returns_html,         "titles");
zone_smoke_test!(zone_creators_returns_html,       "creators");
zone_smoke_test!(zone_campaigns_returns_html,      "campaigns");
zone_smoke_test!(zone_digital_spaces_returns_html, "digital-spaces");

/// An unknown ?zone= value must not 500 -- falls through to Now.
/// Prevents future Zone::parse changes from creating a 500 on typo input.
#[tokio::test]
async fn zone_unknown_does_not_500() {
    let Some(server) = live_server() else { return; };
    let resp = server.get("/?zone=does_not_exist_at_all").await;
    resp.assert_status_ok();
}

// ── JSON API smoke tests ───────────────────────────────────────────────────────

/// GET /api/events returns a JSON array.
/// Gaspar's concern: serde field names must match the Supabase SELECT clause.
/// If EventRow.event_type doesn't match the JSON key "type", this 500s visibly.
#[tokio::test]
async fn api_events_returns_json_array() {
    let Some(server) = live_server() else { return; };
    let resp = server.get("/api/events").await;
    resp.assert_status_ok();
    let body = resp.text();
    assert!(
        body.starts_with('['),
        "/api/events should return a JSON array, got: {}",
        &body[..80.min(body.len())]
    );
}

/// /api/title-holders/current returns a non-empty JSON array.
/// Deduplication is owned by the current_title_holders SQL view;
/// this test verifies the endpoint is reachable and returns parseable data.
/// Unit test for the Rust-side dedup helper lives in src/ssr/query.rs.
#[tokio::test]
async fn api_title_holders_current_returns_json() {
    let Some(server) = live_server() else { return; };
    let resp = server.get("/api/title-holders/current").await;
    resp.assert_status_ok();
    let json: serde_json::Value = serde_json::from_str(&resp.text())
        .expect("/api/title-holders/current is not valid JSON");
    assert!(json.is_array(), "expected a JSON array");
    assert!(!json.as_array().unwrap().is_empty(), "current_title_holders view returned empty");
}

#[tokio::test]
async fn api_places_returns_json_array() {
    let Some(server) = live_server() else { return; };
    let resp = server.get("/api/places").await;
    resp.assert_status_ok();
    assert!(resp.text().starts_with('['));
}

// ── Legacy named paths ─────────────────────────────────────────────────────────

/// /coming-up must still work (legacy path kept for backwards compatibility).
#[tokio::test]
async fn legacy_coming_up_path_works() {
    let Some(server) = live_server() else { return; };
    let resp = server.get("/coming-up").await;
    let status = resp.status_code();
    assert!(
        status.is_success() || status.is_redirection(),
        "/coming-up returned {status} -- legacy path broken"
    );
}
