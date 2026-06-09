
//! Stories — oral histories, essays, and community archive pieces.
//! Powers the BEAR ARCHIVES oral history section.
//! Table exists and is ready; content collection is pending.
//!
//! Privacy note: stories with privacy_mode=true are excluded from public listing.
//! This covers stories from bears in criminalised countries.

use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Story;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct StoriesQuery {
    pub story_type: Option<String>,  // oral_history | essay | interview | archive
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
    let limit = params.limit.unwrap_or(50).min(200);

    let mut url = format!(
        "{}/rest/v1/stories?select=*&active=eq.true&privacy_mode=eq.false&order=timeline_year.desc,year.desc&limit={}",
        db.url, limit
    );

    if let Some(t) = params.story_type          { url.push_str(&format!("&story_type=eq.{}", t)); }
    if let Some(y) = params.year_from           { url.push_str(&format!("&year=gte.{}", y)); }
    if let Some(y) = params.year_to             { url.push_str(&format!("&year=lte.{}", y)); }
    if let Some(l) = params.language_code       { url.push_str(&format!("&language_code=eq.{}", l)); }
    if let Some(true) = params.featured         { url.push_str("&featured=eq.true"); }

    Ok(Json(db.get_json::<Vec<Story>>(&url).await?))
}

/// GET /api/stories/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Story>, AppError> {
    let url = format!(
        "{}/rest/v1/stories?select=*&id=eq.{}&privacy_mode=eq.false&limit=1",
        db.url, id
    );
    let mut stories: Vec<Story> = db.get_json(&url).await?;
    stories.pop()
        .ok_or_else(|| AppError::NotFound(format!("Story {} not found", id)))
        .map(Json)
}
