
use axum::{extract::{Path, Query, State}, Json};
use bearings_shared::models::Club;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct ClubsQuery {
    pub country: Option<String>,
    pub city: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/clubs
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<ClubsQuery>,
) -> Result<Json<Vec<Club>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(500);
    let mut url = format!(
        "{}/rest/v1/clubs?select=*&active=eq.true&order=country.asc,name.asc&limit={}",
        db.url, limit
    );
    if let Some(c) = params.country { url.push_str(&format!("&country=eq.{}", c)); }
    if let Some(c) = params.city    { url.push_str(&format!("&city=eq.{}", c)); }

    Ok(Json(db.get_json::<Vec<Club>>(&url).await?))
}

/// GET /api/clubs/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Club>, AppError> {
    let url = format!("{}/rest/v1/clubs?select=*&id=eq.{}&limit=1", db.url, id);
    let mut clubs: Vec<Club> = db.get_json(&url).await?;
    clubs.pop().ok_or_else(|| AppError::NotFound(format!("Club {} not found", id))).map(Json)
}
