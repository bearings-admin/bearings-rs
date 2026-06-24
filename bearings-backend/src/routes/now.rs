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
//! One call, one response, everything renders together.
//!
//! Query params:
//!   lat, lng    — bear's current location (from browser geolocation or IP)
//!   radius_km   — how far to search (default 500km for events, 50km for venues)
//!
//! The response is the typed [`NowFeed`] — a faithful mirror of the `now_feed`
//! Supabase function's JSON (no `serde_json::Value` pass-through).

use crate::{db::SupabaseClient, error::AppError};
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct NowQuery {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_km: Option<f64>,
}

/// Full NOW payload returned by the `now_feed` RPC.
#[derive(Debug, Serialize, Deserialize)]
pub struct NowFeed {
    pub as_of: String,
    pub location_used: bool,
    pub hot_events: Vec<HotEvent>,
    pub nearby_venues: Vec<NearbyVenue>,
    pub active_campaigns: Vec<CampaignBrief>,
    pub current_titles: Vec<CurrentTitle>,
}

/// A hot event (next ~30 days).
#[derive(Debug, Serialize, Deserialize)]
pub struct HotEvent {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub country: String,
    pub description: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub hot: bool,
    pub link: String,
    pub slug: Option<String>,
    pub inclusion_flag_codes: Option<Vec<String>>,
}

/// A nearby venue, distance-sorted when a location is supplied.
#[derive(Debug, Serialize, Deserialize)]
pub struct NearbyVenue {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub country: String,
    pub description: String,
    pub place_type: String,
    pub lat: f64,
    pub lng: f64,
    pub distance_km: f64,
    pub has_bear_night: bool,
    pub men_only: bool,
    pub bear_night_schedule: Option<String>,
    pub booking_link: Option<String>,
    pub website: String,
    pub slug: String,
    pub inclusion_flag_codes: Option<Vec<String>>,
}

/// A funding campaign (brief projection for the NOW rail).
#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignBrief {
    pub id: i64,
    pub name: String,
    pub org: String,
    pub description: String,
    pub link: String,
    pub currency: String,
    pub goal: Option<i64>,
    pub raised: Option<i64>,
}

/// A current title holder (or gap record).
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentTitle {
    pub id: i64,
    pub holder_name: String,
    pub holder_status: String,
    pub display_name: String,
    pub title_name: String,
    pub year: i32,
    pub city: Option<String>,
    pub country: Option<String>,
    pub competition_name: Option<String>,
    pub competition_scope: Option<String>,
    pub display_status: Option<String>,
    pub holdover_reason: Option<String>,
}

/// GET /api/now — the full NOW zone payload in a single database call.
/// Uses the `now_feed` Supabase function (proximity sorting handled in SQL).
pub async fn feed(
    State(db): State<SupabaseClient>,
    Query(params): Query<NowQuery>,
) -> Result<Json<NowFeed>, AppError> {
    let body = serde_json::json!({
        "input_lat":  params.lat,
        "input_lng":  params.lng,
        "radius_km":  params.radius_km.unwrap_or(500.0),
    });

    let feed: NowFeed = db.post_rpc("now_feed", &body).await?;
    Ok(Json(feed))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the `now_feed` contract: a representative payload (with nulls in
    /// every nullable field) must deserialise into `NowFeed`. Catches a renamed
    /// field or a required/Option mismatch at `cargo test` time, no network.
    #[test]
    fn now_feed_deserialises_with_nulls() {
        let json = r#"{
          "as_of":"2026-06-24","location_used":false,
          "hot_events":[{"id":1,"name":"E","city":"C","country":"X","description":"d",
            "type":"party","start_date":"2026-07-01","end_date":null,"hot":true,
            "link":"l","slug":null,"inclusion_flag_codes":null}],
          "nearby_venues":[{"id":2,"name":"V","city":"C","country":"X","description":"d",
            "place_type":"sauna-bathhouse","lat":1.0,"lng":2.0,"distance_km":3.5,
            "has_bear_night":true,"men_only":false,"bear_night_schedule":null,
            "booking_link":null,"website":"w","slug":"s","inclusion_flag_codes":["x"]}],
          "active_campaigns":[{"id":3,"name":"C","org":"O","description":"d","link":"l",
            "currency":"EUR","goal":null,"raised":0}],
          "current_titles":[{"id":4,"holder_name":"H","holder_status":"active",
            "display_name":"D","title_name":"T","year":2026,"city":null,"country":null,
            "competition_name":null,"competition_scope":null,"display_status":null,
            "holdover_reason":null}]
        }"#;
        let feed: NowFeed = serde_json::from_str(json).expect("NowFeed should deserialise");
        assert_eq!(feed.hot_events[0].event_type, "party");
        assert!(feed.hot_events[0].end_date.is_none());
        assert_eq!(feed.active_campaigns[0].raised, Some(0));
        assert!(feed.active_campaigns[0].goal.is_none());
        assert!(feed.current_titles[0].holdover_reason.is_none());
    }
}
