//! Data access for the `clubs` table.

use async_trait::async_trait;
use bearings_shared::models::Club;
use crate::db::SupabaseClient;
use crate::error::AppError;
use super::clause;

#[derive(Debug, Default, Clone)]
pub struct ClubFilter {
    pub country: Option<String>,
    pub city:    Option<String>,
    pub limit:   u32,
}

#[async_trait]
pub trait ClubRepository: Send + Sync {
    async fn find(&self, filter: ClubFilter) -> Result<Vec<Club>, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Club>, AppError>;
}

pub struct SupabaseClubRepository {
    db: SupabaseClient,
}

impl SupabaseClubRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ClubRepository for SupabaseClubRepository {
    async fn find(&self, filter: ClubFilter) -> Result<Vec<Club>, AppError> {
        let mut url = format!(
            "{}/rest/v1/clubs?select=*&active=eq.true&order=country.asc,name.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(c) = filter.country { url.push_str(&clause("country", "eq", &c)); }
        if let Some(c) = filter.city    { url.push_str(&clause("city",    "eq", &c)); }
        self.db.get_json::<Vec<Club>>(&url).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Club>, AppError> {
        let url = format!("{}/rest/v1/clubs?select=*&id=eq.{}&limit=1", self.db.url, id);
        let mut clubs: Vec<Club> = self.db.get_json(&url).await?;
        Ok(clubs.pop())
    }
}
