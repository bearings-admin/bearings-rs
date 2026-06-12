//! Data access for the `creators` table.

use super::clause;
use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use bearings_shared::models::Creator;

#[derive(Debug, Default, Clone)]
pub struct CreatorFilter {
    pub creator_type: Option<String>,
    pub country: Option<String>,
    pub bear_community_member: bool,
    pub limit: u32,
}

#[async_trait]
pub trait CreatorRepository: Send + Sync {
    async fn find(&self, filter: CreatorFilter) -> Result<Vec<Creator>, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Creator>, AppError>;
}

pub struct SupabaseCreatorRepository {
    db: SupabaseClient,
}
impl SupabaseCreatorRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CreatorRepository for SupabaseCreatorRepository {
    async fn find(&self, filter: CreatorFilter) -> Result<Vec<Creator>, AppError> {
        let mut url = format!(
            "{}/rest/v1/creators?select=*&active=eq.true&order=name.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(t) = filter.creator_type {
            url.push_str(&clause("creator_type", "eq", &t));
        }
        if let Some(c) = filter.country {
            url.push_str(&clause("country", "eq", &c));
        }
        if filter.bear_community_member {
            url.push_str(&clause("bear_community_member", "eq", "true"));
        }
        self.db.get_json::<Vec<Creator>>(&url).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Creator>, AppError> {
        let url = format!(
            "{}/rest/v1/creators?select=*&id=eq.{}&limit=1",
            self.db.url, id
        );
        let mut creators: Vec<Creator> = self.db.get_json(&url).await?;
        Ok(creators.pop())
    }
}
