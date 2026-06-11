//! Data access for the `bear_history` table.

use async_trait::async_trait;
use bearings_shared::models::BearHistory;
use crate::db::SupabaseClient;
use crate::error::AppError;
use super::clause;

#[derive(Debug, Default, Clone)]
pub struct HistoryFilter {
    pub year_from: Option<i32>,
    pub year_to:   Option<i32>,
    pub category:  Option<String>,
    pub featured:  bool,
    pub limit:     u32,
}

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    async fn find(&self, filter: HistoryFilter) -> Result<Vec<BearHistory>, AppError>;
}

pub struct SupabaseHistoryRepository { db: SupabaseClient }
impl SupabaseHistoryRepository { pub fn new(db: SupabaseClient) -> Self { Self { db } } }

#[async_trait]
impl HistoryRepository for SupabaseHistoryRepository {
    async fn find(&self, filter: HistoryFilter) -> Result<Vec<BearHistory>, AppError> {
        let mut url = format!(
            "{}/rest/v1/bear_history?select=*&active=eq.true&order=year.desc,month.desc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(y) = filter.year_from { url.push_str(&clause("year", "gte", &y.to_string())); }
        if let Some(y) = filter.year_to   { url.push_str(&clause("year", "lte", &y.to_string())); }
        if let Some(c) = filter.category  { url.push_str(&clause("category", "eq", &c)); }
        if filter.featured                { url.push_str(&clause("featured", "eq", "true")); }
        self.db.get_json::<Vec<BearHistory>>(&url).await
    }
}
