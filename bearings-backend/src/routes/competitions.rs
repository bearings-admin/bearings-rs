//! GET /api/competitions. Data access in `repositories::competition_repo`.

use axum::{extract::{Query, State}, Json};
use bearings_shared::models::Competition;
use serde::Deserialize;
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::competition_repo::{CompetitionFilter, CompetitionRepository, SupabaseCompetitionRepository};

#[derive(Deserialize)]
pub struct CompetitionsQuery {
    pub scope:            Option<String>,
    pub country:          Option<String>,
    pub include_archived: Option<bool>,
    pub limit:            Option<u32>,
}

/// GET /api/competitions
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<CompetitionsQuery>,
) -> Result<Json<Vec<Competition>>, AppError> {
    let repo = SupabaseCompetitionRepository::new(db);
    let comps = repo.find(CompetitionFilter {
        scope:            params.scope,
        country:          params.country,
        include_archived: params.include_archived.unwrap_or(false),
        limit:            params.limit.unwrap_or(100).min(200),
    }).await?;
    Ok(Json(comps))
}
