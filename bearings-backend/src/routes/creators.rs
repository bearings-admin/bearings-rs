
//! Creators — the people behind bear community art, music, and film.
//! Powers the BEAR ARCHIVES creator zone.
//! 35 verified creators as of June 2026.

use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Creator;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct CreatorsQuery {
    pub creator_type: Option<String>,  // musician | filmmaker | illustrator | historian
    pub country: Option<String>,
    pub bear_community_member: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/creators
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<CreatorsQuery>,
) -> Result<Json<Vec<Creator>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(200);

    let mut url = format!(
        "{}/rest/v1/creators?select=*&active=eq.true&order=name.asc&limit={}",
        db.url, limit
    );

    if let Some(t) = params.creator_type        { url.push_str(&format!("&creator_type=eq.{}", t)); }
    if let Some(c) = params.country             { url.push_str(&format!("&country=eq.{}", c)); }
    if let Some(true) = params.bear_community_member { url.push_str("&bear_community_member=eq.true"); }

    Ok(Json(db.get_json::<Vec<Creator>>(&url).await?))
}

/// GET /api/creators/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Creator>, AppError> {
    let url = format!(
        "{}/rest/v1/creators?select=*&id=eq.{}&limit=1",
        db.url, id
    );
    let mut creators: Vec<Creator> = db.get_json(&url).await?;
    creators.pop()
        .ok_or_else(|| AppError::NotFound(format!("Creator {} not found", id)))
        .map(Json)
}
