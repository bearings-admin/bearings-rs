//! Data access for the `title_holders` table and `current_title_holders` view.

use super::clause;
use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use bearings_shared::models::TitleHolder;

#[derive(Debug, Default, Clone)]
pub struct TitleFilter {
    pub title_name: Option<String>,
    pub year: Option<i32>,
    pub country: Option<String>,
    pub limit: u32,
}

#[async_trait]
pub trait TitleRepository: Send + Sync {
    /// Full competition archive, filterable by title, year, country.
    async fn find(&self, filter: TitleFilter) -> Result<Vec<TitleHolder>, AppError>;
    /// Current holder per competition, from the deduplicating SQL view.
    async fn find_current(&self) -> Result<Vec<TitleHolder>, AppError>;
}

pub struct SupabaseTitleRepository {
    db: SupabaseClient,
}
impl SupabaseTitleRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TitleRepository for SupabaseTitleRepository {
    async fn find(&self, filter: TitleFilter) -> Result<Vec<TitleHolder>, AppError> {
        let mut url = format!(
            "{}/rest/v1/title_holders?select=*&order=year.desc,title_name.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(t) = filter.title_name {
            url.push_str(&clause("title_name", "eq", &t));
        }
        if let Some(y) = filter.year {
            url.push_str(&clause("year", "eq", &y.to_string()));
        }
        if let Some(c) = filter.country {
            url.push_str(&clause("country", "eq", &c));
        }
        self.db.get_json::<Vec<TitleHolder>>(&url).await
    }

    async fn find_current(&self) -> Result<Vec<TitleHolder>, AppError> {
        let url = format!(
            "{}/rest/v1/current_title_holders?select=*&order=title_name.asc",
            self.db.url
        );
        self.db.get_json::<Vec<TitleHolder>>(&url).await
    }
}
