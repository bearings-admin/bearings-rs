
//! Digital spaces — the online bear community.
//! Apps, Discord servers, podcasts, Twitch channels, gaming communities.
//! 26 verified spaces as of June 2026.
//!
//! Note: DB column is `url` not `link`.

use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::DigitalSpace;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct DigitalSpacesQuery {
    pub space_type: Option<String>,   // app | discord | podcast | twitch | reddit | youtube
    pub country: Option<String>,
    pub bear_specific: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/digital-spaces
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<DigitalSpacesQuery>,
) -> Result<Json<Vec<DigitalSpace>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(200);

    let mut url = format!(
        "{}/rest/v1/digital_spaces?select=*&active=eq.true&order=space_type.asc,name.asc&limit={}",
        db.url, limit
    );

    if let Some(t) = params.space_type         { url.push_str(&format!("&space_type=eq.{}", t)); }
    if let Some(c) = params.country            { url.push_str(&format!("&country=eq.{}", c)); }
    if let Some(true) = params.bear_specific   { url.push_str("&bear_specific=eq.true"); }

    Ok(Json(db.get_json::<Vec<DigitalSpace>>(&url).await?))
}

/// GET /api/digital-spaces/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<DigitalSpace>, AppError> {
    let url = format!(
        "{}/rest/v1/digital_spaces?select=*&id=eq.{}&limit=1",
        db.url, id
    );
    let mut spaces: Vec<DigitalSpace> = db.get_json(&url).await?;
    spaces.pop()
        .ok_or_else(|| AppError::NotFound(format!("Digital space {} not found", id)))
        .map(Json)
}
