//! Digital spaces — apps, Discord servers, podcasts, Twitch, Reddit.
//! Data access in `repositories::digital_space_repo`. (DB column is `url` not `link`.)

use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::DigitalSpace;
use serde::Deserialize;
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::digital_space_repo::{DigitalSpaceFilter, DigitalSpaceRepository, SupabaseDigitalSpaceRepository};

#[derive(Deserialize)]
pub struct DigitalSpacesQuery {
    pub space_type:    Option<String>,
    pub country:       Option<String>,
    pub bear_specific: Option<bool>,
    pub limit:         Option<u32>,
}

/// GET /api/digital-spaces
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<DigitalSpacesQuery>,
) -> Result<Json<Vec<DigitalSpace>>, AppError> {
    let repo = SupabaseDigitalSpaceRepository::new(db);
    let spaces = repo.find(DigitalSpaceFilter {
        space_type:    params.space_type,
        country:       params.country,
        bear_specific: params.bear_specific.unwrap_or(false),
        limit:         params.limit.unwrap_or(100).min(200),
    }).await?;
    Ok(Json(spaces))
}

/// GET /api/digital-spaces/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<DigitalSpace>, AppError> {
    let repo = SupabaseDigitalSpaceRepository::new(db);
    repo.find_by_id(id).await?
        .ok_or_else(|| AppError::NotFound(format!("Digital space {id} not found")))
        .map(Json)
}
