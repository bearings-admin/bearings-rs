//! Stories — oral histories, essays, archive pieces.
//! Data access in `repositories::story_repo`. Privacy-flagged rows are excluded there.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::story_repo::{StoryFilter, StoryRepository, SupabaseStoryRepository};
use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Story;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StoriesQuery {
    pub story_type: Option<String>,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
    pub language_code: Option<String>,
    pub featured: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/stories
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<StoriesQuery>,
) -> Result<Json<Vec<Story>>, AppError> {
    let repo = SupabaseStoryRepository::new(db);
    let stories = repo
        .find(StoryFilter {
            story_type: params.story_type,
            year_from: params.year_from,
            year_to: params.year_to,
            language_code: params.language_code,
            featured: params.featured.unwrap_or(false),
            limit: params.limit.unwrap_or(50).min(200),
        })
        .await?;
    Ok(Json(stories))
}

/// GET /api/stories/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Story>, AppError> {
    let repo = SupabaseStoryRepository::new(db);
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Story {id} not found")))
        .map(Json)
}
