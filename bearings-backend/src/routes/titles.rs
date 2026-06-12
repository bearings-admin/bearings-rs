//! GET /api/title-holders — title archive. Data access in `repositories::title_repo`.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::title_repo::{SupabaseTitleRepository, TitleFilter, TitleRepository};
use axum::{
    extract::{Query, State},
    Json,
};
use bearings_shared::models::TitleHolder;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TitlesQuery {
    pub title_name: Option<String>,
    pub year: Option<i32>,
    pub country: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/title-holders — full competition archive, filterable.
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<TitlesQuery>,
) -> Result<Json<Vec<TitleHolder>>, AppError> {
    let repo = SupabaseTitleRepository::new(db);
    let holders = repo
        .find(TitleFilter {
            title_name: params.title_name,
            year: params.year,
            country: params.country,
            limit: params.limit.unwrap_or(200).min(500),
        })
        .await?;
    Ok(Json(holders))
}

/// GET /api/title-holders/current — current holder per competition (SQL view dedup).
pub async fn current(State(db): State<SupabaseClient>) -> Result<Json<Vec<TitleHolder>>, AppError> {
    let repo = SupabaseTitleRepository::new(db);
    Ok(Json(repo.find_current().await?))
}
