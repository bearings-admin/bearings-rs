//! Inclusion flags — CONST-10: inclusion is shown, not decided.
//! Data access in `repositories::flag_repo`.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::flag_repo::{FlagRepository, FlaggedEventsFilter, SupabaseFlagRepository};
use axum::extract::{Query, State};
use axum::Json;
use bearings_shared::models::{Event, InclusionFlag};
use serde::Deserialize;

/// GET /api/inclusion-flags — reference table of all flag codes.
pub async fn list_flags(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<InclusionFlag>>, AppError> {
    let repo = SupabaseFlagRepository::new(db);
    Ok(Json(repo.list_flags().await?))
}

#[derive(Deserialize)]
pub struct FlaggedEventsQuery {
    pub flag_code: Option<String>,
    pub country: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/events/flagged — events carrying inclusion flags, with context.
pub async fn flagged_events(
    State(db): State<SupabaseClient>,
    Query(params): Query<FlaggedEventsQuery>,
) -> Result<Json<Vec<Event>>, AppError> {
    let repo = SupabaseFlagRepository::new(db);
    let events = repo
        .flagged_events(FlaggedEventsFilter {
            flag_code: params.flag_code,
            country: params.country,
            limit: params.limit.unwrap_or(50).min(200),
        })
        .await?;
    Ok(Json(events))
}
