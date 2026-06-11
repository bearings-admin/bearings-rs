//! Data access for the `digital_spaces` table.

use async_trait::async_trait;
use bearings_shared::models::DigitalSpace;
use crate::db::SupabaseClient;
use crate::error::AppError;
use super::clause;

#[derive(Debug, Default, Clone)]
pub struct DigitalSpaceFilter {
    pub space_type:    Option<String>,
    pub country:       Option<String>,
    pub bear_specific: bool,
    pub limit:         u32,
}

#[async_trait]
pub trait DigitalSpaceRepository: Send + Sync {
    async fn find(&self, filter: DigitalSpaceFilter) -> Result<Vec<DigitalSpace>, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<DigitalSpace>, AppError>;
}

pub struct SupabaseDigitalSpaceRepository { db: SupabaseClient }
impl SupabaseDigitalSpaceRepository { pub fn new(db: SupabaseClient) -> Self { Self { db } } }

#[async_trait]
impl DigitalSpaceRepository for SupabaseDigitalSpaceRepository {
    async fn find(&self, filter: DigitalSpaceFilter) -> Result<Vec<DigitalSpace>, AppError> {
        let mut url = format!(
            "{}/rest/v1/digital_spaces?select=*&active=eq.true&order=space_type.asc,name.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(t) = filter.space_type { url.push_str(&clause("space_type", "eq", &t)); }
        if let Some(c) = filter.country    { url.push_str(&clause("country", "eq", &c)); }
        if filter.bear_specific            { url.push_str(&clause("bear_specific", "eq", "true")); }
        self.db.get_json::<Vec<DigitalSpace>>(&url).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<DigitalSpace>, AppError> {
        let url = format!("{}/rest/v1/digital_spaces?select=*&id=eq.{}&limit=1", self.db.url, id);
        let mut spaces: Vec<DigitalSpace> = self.db.get_json(&url).await?;
        Ok(spaces.pop())
    }
}
