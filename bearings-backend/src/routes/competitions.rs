
use axum::{extract::{Query, State}, Json};
use bearings_shared::models::Competition;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct CompetitionsQuery {
    pub scope: Option<String>,   // international | continental | national | regional | local
    pub country: Option<String>,
    pub include_archived: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/competitions
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<CompetitionsQuery>,
) -> Result<Json<Vec<Competition>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(200);
    let include_archived = params.include_archived.unwrap_or(false);

    let mut url = format!(
        "{}/rest/v1/competitions?select=*&order=scope.asc,country.asc,name.asc&limit={}",
        db.url, limit
    );

    // Default to active only unless archived is explicitly requested
    if !include_archived {
        url.push_str("&active=eq.true");
    }
    if let Some(s) = params.scope   { url.push_str(&format!("&scope=eq.{}", s)); }
    if let Some(c) = params.country { url.push_str(&format!("&country=eq.{}", c)); }

    Ok(Json(db.get_json::<Vec<Competition>>(&url).await?))
}
