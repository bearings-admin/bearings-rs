
use axum::{extract::{Query, State}, Json};
use bearings_shared::models::TitleHolder;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct TitlesQuery {
    pub title_name: Option<String>,
    pub year: Option<i32>,
    pub country: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/title-holders
/// Returns the full competition archive — filterable by title, year, country.
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<TitlesQuery>,
) -> Result<Json<Vec<TitleHolder>>, AppError> {
    let limit = params.limit.unwrap_or(200).min(500);
    let mut url = format!(
        "{}/rest/v1/title_holders?select=*&order=year.desc,title_name.asc&limit={}",
        db.url, limit
    );
    if let Some(t) = params.title_name { url.push_str(&format!("&title_name=eq.{}", t)); }
    if let Some(y) = params.year       { url.push_str(&format!("&year=eq.{}", y)); }
    if let Some(c) = params.country    { url.push_str(&format!("&country=eq.{}", c)); }

    Ok(Json(db.get_json::<Vec<TitleHolder>>(&url).await?))
}

/// GET /api/title-holders/current
/// Returns current holders per competition using the DB view.
/// The `current_title_holders` view handles deduplication correctly in SQL,
/// joining competitions, clubs, and events for enriched data.
pub async fn current(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<TitleHolder>>, AppError> {
    // Use the current_title_holders view — pre-built in Supabase.
    // This replaces the previous Rust-side HashSet dedup approach.
    // The view joins competition, club, and event data automatically.
    let url = format!(
        "{}/rest/v1/current_title_holders?select=*&order=title_name.asc",
        db.url
    );
    Ok(Json(db.get_json::<Vec<TitleHolder>>(&url).await?))
}
