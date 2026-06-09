
use axum::{extract::{Query, State}, Json};
use bearings_shared::models::Campaign;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct CampaignsQuery {
    pub include_archived: Option<bool>,
}

/// GET /api/campaigns
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<CampaignsQuery>,
) -> Result<Json<Vec<Campaign>>, AppError> {
    let include_archived = params.include_archived.unwrap_or(false);
    let mut url = format!(
        "{}/rest/v1/campaigns?select=*&order=active.desc,name.asc",
        db.url
    );
    if !include_archived {
        url.push_str("&active=eq.true");
    }
    // Never expose campaigns with privacy_mode in the public list
    url.push_str("&privacy_mode=eq.false");

    Ok(Json(db.get_json::<Vec<Campaign>>(&url).await?))
}
