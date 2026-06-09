
use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Event;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct EventsQuery {
    pub country: Option<String>,
    pub month: Option<String>,
    /// event_type maps to the database `type` column
    pub event_type: Option<String>,
    pub limit: Option<u32>,
    pub upcoming_only: Option<bool>,
}

/// GET /api/events
/// List active events. Default: all upcoming events ordered by start_date.
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<EventsQuery>,
) -> Result<Json<Vec<Event>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200); // cap at 200

    let mut url = format!(
        "{}/rest/v1/events?select=*&active=eq.true&order=start_date.asc&limit={}",
        db.url, limit
    );

    if let Some(c) = params.country     { url.push_str(&format!("&country=eq.{}", c)); }
    if let Some(m) = params.month       { url.push_str(&format!("&month=eq.{}", m)); }
    if let Some(t) = params.event_type  { url.push_str(&format!("&type=eq.{}", t)); }

    // upcoming_only: filter to events whose start_date >= today
    if params.upcoming_only.unwrap_or(false) {
        let today = chrono::Local::now().date_naive();
        url.push_str(&format!("&start_date=gte.{}", today));
    }

    Ok(Json(db.get_json::<Vec<Event>>(&url).await?))
}

/// GET /api/events/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Event>, AppError> {
    let url = format!(
        "{}/rest/v1/events?select=*&id=eq.{}&limit=1",
        db.url, id
    );
    let mut events: Vec<Event> = db.get_json(&url).await?;
    events.pop()
        .ok_or_else(|| AppError::NotFound(format!("Event {} not found", id)))
        .map(Json)
}

/// GET /api/events/by-month
/// Returns event counts grouped by month — used for the timeline bar chart.
/// Response: [{"month": "January", "count": 12}, ...]
pub async fn by_month(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<MonthCount>>, AppError> {
    let url = format!(
        "{}/rest/v1/events?select=month&active=eq.true&month=not.is.null",
        db.url
    );
    let events: Vec<MonthOnly> = db.get_json(&url).await?;

    // Count by month in Rust rather than relying on a group-by RPC
    let mut counts: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for e in events {
        if let Some(m) = e.month {
            *counts.entry(m).or_insert(0) += 1;
        }
    }

    let result = counts.into_iter()
        .map(|(month, count)| MonthCount { month, count })
        .collect();

    Ok(Json(result))
}

#[derive(Deserialize)]
struct MonthOnly { month: Option<String> }

#[derive(serde::Serialize)]
pub struct MonthCount {
    pub month: String,
    pub count: u32,
}
