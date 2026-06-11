//! GET /api/clubs — clubs REST endpoints. Data access lives in
//! `repositories::club_repo`.

use axum::{extract::{Path, Query, State}, Json};
use bearings_shared::models::Club;
use serde::Deserialize;
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::club_repo::{ClubFilter, ClubRepository, SupabaseClubRepository};

#[derive(Deserialize)]
pub struct ClubsQuery {
    pub country: Option<String>,
    pub city:    Option<String>,
    pub limit:   Option<u32>,
}

/// GET /api/clubs
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<ClubsQuery>,
) -> Result<Json<Vec<Club>>, AppError> {
    let repo = SupabaseClubRepository::new(db);
    let clubs = repo.find(ClubFilter {
        country: params.country,
        city:    params.city,
        limit:   params.limit.unwrap_or(100).min(500),
    }).await?;
    Ok(Json(clubs))
}

/// GET /api/clubs/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Club>, AppError> {
    let repo = SupabaseClubRepository::new(db);
    repo.find_by_id(id).await?
        .ok_or_else(|| AppError::NotFound(format!("Club {id} not found")))
        .map(Json)
}
