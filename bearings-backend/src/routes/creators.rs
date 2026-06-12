//! Creators — people behind bear community art, music, and film.
//! Data access in `repositories::creator_repo`.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::creator_repo::{
    CreatorFilter, CreatorRepository, SupabaseCreatorRepository,
};
use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Creator;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatorsQuery {
    pub creator_type: Option<String>,
    pub country: Option<String>,
    pub bear_community_member: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/creators
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<CreatorsQuery>,
) -> Result<Json<Vec<Creator>>, AppError> {
    let repo = SupabaseCreatorRepository::new(db);
    let creators = repo
        .find(CreatorFilter {
            creator_type: params.creator_type,
            country: params.country,
            bear_community_member: params.bear_community_member.unwrap_or(false),
            limit: params.limit.unwrap_or(100).min(200),
        })
        .await?;
    Ok(Json(creators))
}

/// GET /api/creators/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Creator>, AppError> {
    let repo = SupabaseCreatorRepository::new(db);
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Creator {id} not found")))
        .map(Json)
}
