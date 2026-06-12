//! Data access for the `competitions` table.

use super::clause;
use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use bearings_shared::models::Competition;

#[derive(Debug, Default, Clone)]
pub struct CompetitionFilter {
    pub scope: Option<String>,
    pub country: Option<String>,
    pub include_archived: bool,
    pub limit: u32,
}

#[async_trait]
pub trait CompetitionRepository: Send + Sync {
    async fn find(&self, filter: CompetitionFilter) -> Result<Vec<Competition>, AppError>;
}

pub struct SupabaseCompetitionRepository {
    db: SupabaseClient,
}
impl SupabaseCompetitionRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CompetitionRepository for SupabaseCompetitionRepository {
    async fn find(&self, filter: CompetitionFilter) -> Result<Vec<Competition>, AppError> {
        let mut url = format!(
            "{}/rest/v1/competitions?select=*&order=scope.asc,country.asc,name.asc&limit={}",
            self.db.url, filter.limit
        );
        if !filter.include_archived {
            url.push_str("&active=eq.true");
        }
        if let Some(s) = filter.scope {
            url.push_str(&clause("scope", "eq", &s));
        }
        if let Some(c) = filter.country {
            url.push_str(&clause("country", "eq", &c));
        }
        self.db.get_json::<Vec<Competition>>(&url).await
    }
}
