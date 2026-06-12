//! Data access for `inclusion_flags` and the `events_with_flags` view.
//!
//! CONST-10: inclusion is shown, not decided. These reads power the "show the
//! reality" context in the UI; they never hide listings.

use super::clause;
use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;
use bearings_shared::models::{Event, InclusionFlag};

#[derive(Debug, Default, Clone)]
pub struct FlaggedEventsFilter {
    pub flag_code: Option<String>,
    pub country: Option<String>,
    pub limit: u32,
}

#[async_trait]
pub trait FlagRepository: Send + Sync {
    /// The reference table of all flag codes with labels and descriptions.
    async fn list_flags(&self) -> Result<Vec<InclusionFlag>, AppError>;
    /// Events carrying at least one inclusion flag (events_with_flags view).
    async fn flagged_events(&self, filter: FlaggedEventsFilter) -> Result<Vec<Event>, AppError>;
}

pub struct SupabaseFlagRepository {
    db: SupabaseClient,
}
impl SupabaseFlagRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FlagRepository for SupabaseFlagRepository {
    async fn list_flags(&self) -> Result<Vec<InclusionFlag>, AppError> {
        let url = format!(
            "{}/rest/v1/inclusion_flags?select=*&active=eq.true&order=severity.asc,code.asc",
            self.db.url
        );
        self.db.get_json::<Vec<InclusionFlag>>(&url).await
    }

    async fn flagged_events(&self, filter: FlaggedEventsFilter) -> Result<Vec<Event>, AppError> {
        let mut url = format!(
            "{}/rest/v1/events_with_flags?select=*&has_flags=eq.true&order=start_date.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(c) = filter.country {
            url.push_str(&clause("country", "eq", &c));
        }
        // PostgREST array-contains: cs.{"VALUE"} -> %7B%22VALUE%22%7D
        if let Some(flag) = filter.flag_code {
            url.push_str(&format!(
                "&inclusion_flag_codes=cs.%7B%22{}%22%7D",
                urlencoding::encode(&flag)
            ));
        }
        self.db.get_json::<Vec<Event>>(&url).await
    }
}
