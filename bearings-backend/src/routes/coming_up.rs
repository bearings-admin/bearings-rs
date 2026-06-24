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
//!   "Bear runs specifically, next 3 months"
//!   GET /api/coming-up?event_type=bear-run
//!
//! The iCal export endpoint mirrors these filters:
//!   GET /api/events/ical.ics?country=Germany&month=September
//!
//! Season mapping (server-side):
//!   spring → Mar–May · summer → Jun–Aug · autumn → Sep–Nov · winter → Dec–Feb
//!   (no season) → next 6 months from today
//!
//! The response is the typed [`ComingUpFeed`] — a faithful mirror of the
//! `coming_up` Supabase function's JSON (no `serde_json::Value` pass-through).

use crate::{db::SupabaseClient, error::AppError};
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

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

/// Full COMING UP payload returned by the `coming_up` RPC.
#[derive(Debug, Serialize, Deserialize)]
pub struct ComingUpFeed {
    pub location_used: bool,
    pub season: Option<String>,
    pub window_from: String,
    pub window_to: String,
    pub events: Vec<UpcomingEvent>,
    pub venues: Vec<UpcomingVenue>,
    pub clubs: Vec<LocalClub>,
}

/// An upcoming event within the trip window.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpcomingEvent {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub country: String,
    pub description: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub event_mode: String,
    pub size: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub hot: bool,
    pub link: String,
    pub slug: String,
    pub inclusion_flag_codes: Option<Vec<String>>,
}

/// A venue near the destination, distance-sorted when a location is supplied.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpcomingVenue {
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
    pub bear_popular: bool,
    pub bear_night_schedule: Option<String>,
    pub booking_link: Option<String>,
    pub website: String,
    pub slug: String,
}

/// A local club at the destination.
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalClub {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub country: String,
    pub description: String,
    pub website: String,
    pub contact_email: Option<String>,
}

/// GET /api/coming-up — trip planner composite endpoint.
/// Uses the `coming_up` Supabase function (proximity + season logic in SQL).
pub async fn feed(
    State(db): State<SupabaseClient>,
    Query(params): Query<ComingUpQuery>,
) -> Result<Json<ComingUpFeed>, AppError> {
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

    let feed: ComingUpFeed = db.post_rpc("coming_up", &body).await?;
    Ok(Json(feed))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the `coming_up` contract: a representative payload (with nulls in
    /// every nullable field) must deserialise into `ComingUpFeed`. Catches a
    /// renamed field or a required/Option mismatch at `cargo test` time.
    #[test]
    fn coming_up_feed_deserialises_with_nulls() {
        let json = r#"{
          "location_used":false,"season":null,
          "window_from":"2026-06-24","window_to":"2026-12-24",
          "events":[{"id":1,"name":"E","city":"C","country":"X","description":"d",
            "type":"bear-run","event_mode":"in_person","size":"regional",
            "start_date":"2026-09-01","end_date":null,"hot":false,"link":"l",
            "slug":"s","inclusion_flag_codes":null}],
          "venues":[{"id":2,"name":"V","city":"C","country":"X","description":"d",
            "place_type":"leather-bar","lat":1.0,"lng":2.0,"distance_km":9.0,
            "has_bear_night":true,"men_only":false,"bear_popular":true,
            "bear_night_schedule":null,"booking_link":null,"website":"w","slug":"s"}],
          "clubs":[{"id":3,"name":"K","city":"C","country":"X","description":"d",
            "website":"w","contact_email":null}]
        }"#;
        let feed: ComingUpFeed =
            serde_json::from_str(json).expect("ComingUpFeed should deserialise");
        assert_eq!(feed.events[0].event_type, "bear-run");
        assert!(feed.events[0].end_date.is_none());
        assert!(feed.venues[0].bear_popular);
        assert!(feed.clubs[0].contact_email.is_none());
        assert!(feed.season.is_none());
    }
}
