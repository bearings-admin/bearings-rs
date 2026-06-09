
//! NOW zone — the "here and now" composite endpoint.
//!
//! A single call returns everything the NOW zone needs:
//!   - Hot events (next 30 days, proximity-sorted if location provided)
//!   - Nearby venues (bear-popular first, within radius)
//!   - Active campaigns (always global)
//!   - Current title holders (always global)
//!
//! Without location: returns upcoming hot events sorted by date.
//! With location: returns events and venues sorted by proximity.
//!
//! This replaces the Lovable approach of 4 sequential API calls
//! which caused the "static" feeling — each section loaded independently.
//! One call, one response, everything renders together.
//!
//! Query params:
//!   lat, lng    — bear's current location (from browser geolocation or IP)
//!   radius_km   — how far to search (default 500km for events, 50km for venues)

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct NowQuery {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_km: Option<f64>,
}

/// GET /api/now
/// Returns the full NOW zone payload in a single database call.
/// Uses the `now_feed` Supabase function which handles proximity sorting in SQL.
pub async fn feed(
    State(db): State<SupabaseClient>,
    Query(params): Query<NowQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let body = serde_json::json!({
        "input_lat":  params.lat,
        "input_lng":  params.lng,
        "radius_km":  params.radius_km.unwrap_or(500.0),
    });

    let result: serde_json::Value = db.post_rpc("now_feed", &body).await?;
    Ok(Json(result))
}
