//! Data access for the `events` table.

use super::clause;
use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use bearings_shared::models::Event;

/// List filter for events. Built from public query params in the handler, then
/// handed to the repository — handlers never assemble a query string themselves.
#[derive(Debug, Default, Clone)]
pub struct EventFilter {
    pub country: Option<String>,
    pub month: Option<String>,
    /// Maps to the database `type` column.
    pub event_type: Option<String>,
    pub upcoming_only: bool,
    pub limit: u32,
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Active events matching `filter`, ordered by start date.
    async fn find(&self, filter: EventFilter) -> Result<Vec<Event>, AppError>;
    /// A single event by id, or `None` if not found.
    async fn find_by_id(&self, id: i64) -> Result<Option<Event>, AppError>;
    /// The `month` value of every active event (for the timeline histogram).
    async fn list_months(&self) -> Result<Vec<String>, AppError>;
}

/// Supabase PostgREST implementation.
pub struct SupabaseEventRepository {
    db: SupabaseClient,
}

impl SupabaseEventRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventRepository for SupabaseEventRepository {
    async fn find(&self, filter: EventFilter) -> Result<Vec<Event>, AppError> {
        let mut url = format!(
            "{}/rest/v1/events?select=*&active=eq.true&order=start_date.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(c) = filter.country {
            url.push_str(&clause("country", "eq", &c));
        }
        if let Some(m) = filter.month {
            url.push_str(&clause("month", "eq", &m));
        }
        if let Some(t) = filter.event_type {
            url.push_str(&clause("type", "eq", &t));
        }
        if filter.upcoming_only {
            let today = chrono::Local::now().date_naive().to_string();
            url.push_str(&clause("start_date", "gte", &today));
        }
        self.db.get_json::<Vec<Event>>(&url).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Event>, AppError> {
        let url = format!(
            "{}/rest/v1/events?select=*&id=eq.{}&limit=1",
            self.db.url, id
        );
        let mut events: Vec<Event> = self.db.get_json(&url).await?;
        Ok(events.pop())
    }

    async fn list_months(&self) -> Result<Vec<String>, AppError> {
        let url = format!(
            "{}/rest/v1/events?select=month&active=eq.true&month=not.is.null",
            self.db.url
        );
        #[derive(serde::Deserialize)]
        struct MonthOnly {
            month: Option<String>,
        }
        let rows: Vec<MonthOnly> = self.db.get_json(&url).await?;
        Ok(rows.into_iter().filter_map(|r| r.month).collect())
    }
}
