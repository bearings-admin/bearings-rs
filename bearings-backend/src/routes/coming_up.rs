//! COMING UP — the trip planner endpoint.
//!
//! A bear planning a trip sets two things:
//!   HERE: their destination (lat/lng or country)
//!   WHEN: a season, month, or date range
//!
//! Returns events + nearby venues + local clubs in one response.
//!
//! Examples:
//!   "What's happening in Europe in September?"
//!   GET /api/coming-up?season=autumn&country=Germany
//!
//!   "What's on near Berlin in October?"
//!   GET /api/coming-up?lat=52.52&lng=13.40&radius_km=200&season=autumn
//!
//!   "Trips I could take this summer from Ottawa"
//!   GET /api/coming-up?lat=45.42&lng=-75.70&radius_km=2000&season=summer
//!
//!   "Bear runs specifically, next 3 months"
//!   GET /api/coming-up?event_type=bear-run
//!
//! The iCal export endpoint mirrors these filters:
//!   GET /api/events/ical.ics?country=Germany&month=September
//!
//! Season mapping (server-side, no client calculation needed):
//!   spring  → March–May
//!   summer  → June–August
//!   autumn  → September–November
//!   winter  → December–February
//!   (no season) → next 6 months from today

use crate::{db::SupabaseClient, error::AppError};
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ComingUpQuery {
    // Location (either lat/lng OR country, or neither for global)
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_km: Option<f64>,
    pub country: Option<String>,

    // Time window (season takes precedence over from/to if both provided)
    pub season: Option<String>,    // spring | summer | autumn | winter
    pub from_date: Option<String>, // ISO date YYYY-MM-DD
    pub to_date: Option<String>,   // ISO date YYYY-MM-DD

    // Content filter
    pub event_type: Option<String>, // bear-run | cruise | social | event | party

    pub limit: Option<i32>,
}

/// GET /api/coming-up
/// Trip planner composite endpoint.
/// Uses the `coming_up` Supabase function for proximity + season logic in SQL.
pub async fn feed(
    State(db): State<SupabaseClient>,
    Query(params): Query<ComingUpQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let body = serde_json::json!({
        "input_lat":  params.lat,
        "input_lng":  params.lng,
        "radius_km":  params.radius_km.unwrap_or(500.0),
        "season":     params.season,
        "from_date":  params.from_date,
        "to_date":    params.to_date,
        "event_type": params.event_type,
        "country":    params.country,
        "max_rows":   params.limit.unwrap_or(30).min(100),
    });

    let result: serde_json::Value = db.post_rpc("coming_up", &body).await?;
    Ok(Json(result))
}
