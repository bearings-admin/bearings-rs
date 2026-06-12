//! Data access for the `future_ideas` table (Bear Future "What Could Be" upvotes).

use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait FutureIdeaRepository: Send + Sync {
    /// Atomically increment an active idea's upvotes and return the new total,
    /// or `None` if the idea does not exist / is inactive.
    async fn increment_upvotes(&self, id: i64) -> Result<Option<i64>, AppError>;
}

pub struct SupabaseFutureIdeaRepository {
    db: SupabaseClient,
}
impl SupabaseFutureIdeaRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FutureIdeaRepository for SupabaseFutureIdeaRepository {
    async fn increment_upvotes(&self, id: i64) -> Result<Option<i64>, AppError> {
        // One atomic `UPDATE ... SET upvotes = upvotes + 1 ... RETURNING upvotes`,
        // via the increment_future_idea_upvotes(idea_id) Postgres function.
        // Replaces a read-then-write that dropped upvotes under concurrent taps.
        let body = serde_json::json!({ "idea_id": id });
        self.db
            .post_rpc::<_, Option<i64>>("increment_future_idea_upvotes", &body)
            .await
    }
}
