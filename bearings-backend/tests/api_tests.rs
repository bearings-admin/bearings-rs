
//! Integration test scaffold for bearings-backend.
//!
//! Currently these are compile-check placeholders — the real tests
//! need the app builder extracted from main() into a pub fn.
//!
//! To enable: refactor main.rs to expose `pub fn build_app(db: SupabaseClient) -> Router`
//! then wire up axum::serve in main(), and use Router directly in tests.
//!
//! Run: cargo test -p bearings-backend
//! Note: --lib runs unit tests; integration tests here require --test api_tests

// No imports needed until TestServer is wired up

#[tokio::test]
async fn health_route_compiles() {
    // Placeholder — proves the test binary compiles and tokio runtime works
    assert_eq!(2 + 2, 4);
}

#[tokio::test]
async fn events_list_compiles() {
    // TODO: extract build_app() from main.rs, then:
    // let app = bearings_backend::build_app(test_db());
    // let client = TestClient::new(app);
    // let resp = client.get("/api/events").await;
    // assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn places_nearby_requires_lat_lng() {
    // TODO: assert /api/places/nearby without params returns 400/422
}

#[tokio::test]
async fn title_holders_current_deduplicates() {
    // TODO: assert /api/title-holders/current has at most one entry per title_name
}

#[tokio::test]
async fn treasury_returns_phase_1() {
    // TODO: assert /api/treasury returns { treasury_phase: 1 } in bootstrapping
}
