//! Data access for the `campaigns` table.

use async_trait::async_trait;
use bearings_shared::models::Campaign;
use crate::db::SupabaseClient;
use crate::error::AppError;

#[derive(Debug, Default, Clone)]
pub struct CampaignFilter {
    pub include_archived: bool,
}

#[async_trait]
pub trait CampaignRepository: Send + Sync {
    async fn find(&self, filter: CampaignFilter) -> Result<Vec<Campaign>, AppError>;
}

pub struct SupabaseCampaignRepository { db: SupabaseClient }
impl SupabaseCampaignRepository { pub fn new(db: SupabaseClient) -> Self { Self { db } } }

#[async_trait]
impl CampaignRepository for SupabaseCampaignRepository {
    async fn find(&self, filter: CampaignFilter) -> Result<Vec<Campaign>, AppError> {
        let mut url = format!(
            "{}/rest/v1/campaigns?select=*&order=active.desc,name.asc",
            self.db.url
        );
        if !filter.include_archived { url.push_str("&active=eq.true"); }
        // Never expose campaigns flagged privacy_mode in the public list (CONST-6).
        url.push_str("&privacy_mode=eq.false");
        self.db.get_json::<Vec<Campaign>>(&url).await
    }
}
