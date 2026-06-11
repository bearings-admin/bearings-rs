//! Data access for the `stories` table.
//!
//! Privacy: rows flagged `privacy_mode` are always excluded from public reads
//! (CONST-6) — this filter is not user-controllable.

use async_trait::async_trait;
use bearings_shared::models::Story;
use crate::db::SupabaseClient;
use crate::error::AppError;
use super::clause;

#[derive(Debug, Default, Clone)]
pub struct StoryFilter {
    pub story_type:    Option<String>,
    pub year_from:     Option<i32>,
    pub year_to:       Option<i32>,
    pub language_code: Option<String>,
    pub featured:      bool,
    pub limit:         u32,
}

#[async_trait]
pub trait StoryRepository: Send + Sync {
    async fn find(&self, filter: StoryFilter) -> Result<Vec<Story>, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Story>, AppError>;
}

pub struct SupabaseStoryRepository { db: SupabaseClient }
impl SupabaseStoryRepository { pub fn new(db: SupabaseClient) -> Self { Self { db } } }

#[async_trait]
impl StoryRepository for SupabaseStoryRepository {
    async fn find(&self, filter: StoryFilter) -> Result<Vec<Story>, AppError> {
        let mut url = format!(
            "{}/rest/v1/stories?select=*&active=eq.true&privacy_mode=eq.false&order=timeline_year.desc,year.desc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(t) = filter.story_type    { url.push_str(&clause("story_type", "eq", &t)); }
        if let Some(y) = filter.year_from     { url.push_str(&clause("year", "gte", &y.to_string())); }
        if let Some(y) = filter.year_to       { url.push_str(&clause("year", "lte", &y.to_string())); }
        if let Some(l) = filter.language_code { url.push_str(&clause("language_code", "eq", &l)); }
        if filter.featured                    { url.push_str(&clause("featured", "eq", "true")); }
        self.db.get_json::<Vec<Story>>(&url).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Story>, AppError> {
        let url = format!(
            "{}/rest/v1/stories?select=*&id=eq.{}&privacy_mode=eq.false&limit=1",
            self.db.url, id
        );
        let mut stories: Vec<Story> = self.db.get_json(&url).await?;
        Ok(stories.pop())
    }
}
