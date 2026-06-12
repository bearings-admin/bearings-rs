//! GET /api/events — events REST endpoints. Data access lives in
//! `repositories::event_repo`; this layer only maps HTTP <-> repository.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::event_repo::{EventFilter, EventRepository, SupabaseEventRepository};
use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Event;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct EventsQuery {
    pub country: Option<String>,
    pub month: Option<String>,
    /// event_type maps to the database `type` column.
    pub event_type: Option<String>,
    pub limit: Option<u32>,
    pub upcoming_only: Option<bool>,
}

/// GET /api/events — list active events, ordered by start date.
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<EventsQuery>,
) -> Result<Json<Vec<Event>>, AppError> {
    let repo = SupabaseEventRepository::new(db);
    let events = repo
        .find(EventFilter {
            country: params.country,
            month: params.month,
            event_type: params.event_type,
            upcoming_only: params.upcoming_only.unwrap_or(false),
            limit: params.limit.unwrap_or(50).min(200),
        })
        .await?;
    Ok(Json(events))
}

/// GET /api/events/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Event>, AppError> {
    let repo = SupabaseEventRepository::new(db);
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Event {id} not found")))
        .map(Json)
}

/// GET /api/events/by-month — event counts grouped by month, for the timeline bar.
/// Response: `[{"month": "January", "count": 12}, ...]`
pub async fn by_month(State(db): State<SupabaseClient>) -> Result<Json<Vec<MonthCount>>, AppError> {
    let repo = SupabaseEventRepository::new(db);
    let months = repo.list_months().await?;

    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for m in months {
        *counts.entry(m).or_insert(0) += 1;
    }
    let result = counts
        .into_iter()
        .map(|(month, count)| MonthCount { month, count })
        .collect();
    Ok(Json(result))
}

#[derive(serde::Serialize)]
pub struct MonthCount {
    pub month: String,
    pub count: u32,
}
