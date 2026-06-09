
use axum::{extract::{Query, State}, Json};
use bearings_shared::models::BearHistory;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
    pub category: Option<String>,
    pub featured: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/bear-history
/// Returns community milestones in reverse chronological order.
/// The Archives zone uses this to populate the timeline spine.
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<BearHistory>>, AppError> {
    let limit = params.limit.unwrap_or(200).min(500);
    let mut url = format!(
        "{}/rest/v1/bear_history?select=*&active=eq.true&order=year.desc,month.desc&limit={}",
        db.url, limit
    );
    if let Some(y) = params.year_from { url.push_str(&format!("&year=gte.{}", y)); }
    if let Some(y) = params.year_to   { url.push_str(&format!("&year=lte.{}", y)); }
    if let Some(c) = params.category  { url.push_str(&format!("&category=eq.{}", c)); }
    if let Some(true) = params.featured { url.push_str("&featured=eq.true"); }

    Ok(Json(db.get_json::<Vec<BearHistory>>(&url).await?))
}
