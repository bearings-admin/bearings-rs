//! GET /api/campaigns. Data access in `repositories::campaign_repo`.

use axum::{extract::{Query, State}, Json};
use bearings_shared::models::Campaign;
use serde::Deserialize;
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::campaign_repo::{CampaignFilter, CampaignRepository, SupabaseCampaignRepository};

#[derive(Deserialize)]
pub struct CampaignsQuery {
    pub include_archived: Option<bool>,
}

/// GET /api/campaigns
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<CampaignsQuery>,
) -> Result<Json<Vec<Campaign>>, AppError> {
    let repo = SupabaseCampaignRepository::new(db);
    let campaigns = repo.find(CampaignFilter {
        include_archived: params.include_archived.unwrap_or(false),
    }).await?;
    Ok(Json(campaigns))
}
