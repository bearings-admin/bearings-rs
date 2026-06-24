//! Data access for revival ("would you return?") signals on closed venues.
//!
//! The route depends on `RevivalRepository`, not on `SupabaseClient` directly,
//! mirroring the other resources (DIP).

use crate::db::SupabaseClient;
use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait RevivalRepository: Send + Sync {
    /// Record one revival signal for a closed place/club and return the new
    /// total, or `None` if the target does not exist. Atomic via the
    /// `increment_revival_votes(p_kind, p_id)` Postgres function.
    async fn record_vote(&self, kind: &str, id: i64) -> Result<Option<i64>, AppError>;
}

pub struct SupabaseRevivalRepository {
    db: SupabaseClient,
}

impl SupabaseRevivalRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RevivalRepository for SupabaseRevivalRepository {
    async fn record_vote(&self, kind: &str, id: i64) -> Result<Option<i64>, AppError> {
        let body = serde_json::json!({ "p_kind": kind, "p_id": id });
        self.db
            .post_rpc::<_, Option<i64>>("increment_revival_votes", &body)
            .await
    }
}
