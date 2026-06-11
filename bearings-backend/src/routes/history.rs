//! GET /api/bear-history. Data access in `repositories::history_repo`.

use axum::{extract::{Query, State}, Json};
use bearings_shared::models::BearHistory;
use serde::Deserialize;
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::history_repo::{HistoryFilter, HistoryRepository, SupabaseHistoryRepository};

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub year_from: Option<i32>,
    pub year_to:   Option<i32>,
    pub category:  Option<String>,
    pub featured:  Option<bool>,
    pub limit:     Option<u32>,
}

/// GET /api/bear-history — community milestones, reverse chronological.
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<BearHistory>>, AppError> {
    let repo = SupabaseHistoryRepository::new(db);
    let entries = repo.find(HistoryFilter {
        year_from: params.year_from,
        year_to:   params.year_to,
        category:  params.category,
        featured:  params.featured.unwrap_or(false),
        limit:     params.limit.unwrap_or(200).min(500),
    }).await?;
    Ok(Json(entries))
}
