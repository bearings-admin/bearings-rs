//! Bluesky social publishing agent.
//!
//! The DB has agent_posts and agent_inbox tables — this module is the Rust
//! implementation. The directive (CONST-10) requires steward review during
//! bootstrapping. The DB enforces this via reviewed_by_steward boolean.
//!
//! ## What this module does:
//!
//!   1. generate_post() — reads a pending event/title/campaign and drafts
//!      post text via the Anthropic API. Writes to agent_posts with
//!      status = "draft" and reviewed_by_steward = false.
//!
//!   2. check_inbox() — polls agent_inbox for new inbound mentions,
//!      classifies intent (submission | question | feedback), and either
//!      auto-responds or escalates to steward.
//!
//!   3. publish_approved() — finds agent_posts where reviewed_by_steward = true
//!      and status = "scheduled" and publishes to Bluesky AT Protocol API.
//!
//! ## Circuit breakers (from directive):
//!   - 4-hour cooldown between posts (enforced by published_at check)
//!   - reviewed_by_steward MUST be true — never bypassed
//!   - Max 3 posts per day (enforced by count of today's published_at)
//!
//! ## NOT YET IMPLEMENTED:
//!   - Actual Bluesky AT Protocol API calls (needs atrium crate or reqwest)
//!   - Anthropic API post generation (needs API key from .env)
//!   - Scheduling logic
//!
//! This module is scaffolded — all DB interactions are ready.
//! Bluesky API integration is the next Rust work item.

use crate::{error::AgentError, supabase::SupabaseWriter};
use bearings_shared::models::AgentPost;
use chrono::Utc;

// BSKY_API base URL — uncomment when AT Protocol integration is implemented
// const BSKY_API: &str = "https://bsky.social/xrpc";

/// Check and enforce the 4-hour post cooldown.
/// Returns Ok(true) if posting is allowed, Ok(false) if in cooldown.
pub async fn cooldown_ok(db: &SupabaseWriter) -> Result<bool, AgentError> {
    let four_hours_ago = (Utc::now() - chrono::Duration::hours(4)).to_rfc3339();

    let url = format!(
        "agent_posts?select=id&status=eq.published&published_at=gte.{}&limit=1",
        urlencoding::encode(&four_hours_ago)
    );

    let recent: Vec<serde_json::Value> = db.get(&url).await?;
    Ok(recent.is_empty())
}

/// Check daily post limit (max 3 per day).
pub async fn daily_limit_ok(db: &SupabaseWriter) -> Result<bool, AgentError> {
    let today = chrono::Local::now().date_naive().to_string();
    let url = format!(
        "agent_posts?select=id&status=eq.published&published_at=gte.{}&limit=4",
        today
    );
    let today_posts: Vec<serde_json::Value> = db.get(&url).await?;
    Ok(today_posts.len() < 3)
}

/// Find approved posts ready to publish (reviewed_by_steward=true, status=scheduled).
pub async fn get_approved_posts(db: &SupabaseWriter) -> Result<Vec<AgentPost>, AgentError> {
    let now = Utc::now().to_rfc3339();
    let url = format!(
        "agent_posts?select=*&reviewed_by_steward=eq.true&status=eq.scheduled&scheduled_for=lte.{}&order=scheduled_for.asc&limit=1",
        urlencoding::encode(&now)
    );

    let posts: Vec<serde_json::Value> = db.get(&url).await?;
    let mut result = Vec::new();

    for p in posts {
        // Convert to AgentPost struct (partial — sufficient for publishing)
        let post = AgentPost {
            id: p["id"].as_i64().unwrap_or(0),
            platform: p["platform"].as_str().unwrap_or("bluesky").to_string(),
            post_type: p["post_type"].as_str().unwrap_or("").to_string(),
            post_text: p["post_text"].as_str().unwrap_or("").to_string(),
            status: p["status"].as_str().map(String::from),
            reviewed_by_steward: p["reviewed_by_steward"].as_bool(),
            ..Default::default()
        };
        result.push(post);
    }

    Ok(result)
}

/// Mark a post as published and record the post URI.
pub async fn mark_published(
    db: &SupabaseWriter,
    post_id: i64,
    post_uri: &str,
    post_cid: &str,
) -> Result<(), AgentError> {
    let path = format!("agent_posts?id=eq.{}", post_id);
    db.patch(
        &path,
        &serde_json::json!({
            "status": "published",
            "post_uri": post_uri,
            "post_cid": post_cid,
            "published_at": Utc::now().to_rfc3339(),
        }),
    )
    .await
}

/// Mark a post as failed with an error note.
pub async fn mark_failed(
    db: &SupabaseWriter,
    post_id: i64,
    reason: &str,
) -> Result<(), AgentError> {
    let path = format!("agent_posts?id=eq.{}", post_id);
    db.patch(
        &path,
        &serde_json::json!({
            "status": "failed",
            "notes": reason,
        }),
    )
    .await
}

// ── TODO: Bluesky AT Protocol integration ────────────────────
//
// The Bluesky API uses the AT Protocol (https://atproto.com).
// To publish a post:
//
// 1. Authenticate:
//    POST https://bsky.social/xrpc/com.atproto.server.createSession
//    Body: { identifier: "handle.bsky.social", password: "app-password" }
//    Returns: { accessJwt, refreshJwt, did }
//
// 2. Create post:
//    POST https://bsky.social/xrpc/com.atproto.repo.createRecord
//    Headers: Authorization: Bearer {accessJwt}
//    Body: {
//      repo: did,
//      collection: "app.bsky.feed.post",
//      record: {
//        text: post_text,
//        createdAt: timestamp,
//        $type: "app.bsky.feed.post"
//      }
//    }
//    Returns: { uri, cid }
//
// Rust crate options:
//   - atrium-api (https://github.com/sugyan/atrium) — most complete
//   - or raw reqwest calls (consistent with our no-heavy-crates approach)
//
// Add to bearings-agent/Cargo.toml when implementing:
//   atrium-api = "0.24"
//
// Env vars to add to .env.example:
//   BSKY_HANDLE=bearings.bsky.social
//   BSKY_APP_PASSWORD=your-bluesky-app-password

/// Check agent_inbox for new inbound mentions.
/// Returns count of new items found.
pub async fn check_inbox(db: &SupabaseWriter) -> Result<usize, AgentError> {
    let url = "agent_inbox?select=id&status=eq.pending&order=received_at.asc&limit=10";
    let items: Vec<serde_json::Value> = db.get(url).await?;

    if items.is_empty() {
        return Ok(0);
    }

    tracing::info!("Agent inbox: {} pending items", items.len());

    // TODO: implement intent classification via Anthropic API
    // For each item:
    //   1. Call Anthropic with the post_text
    //   2. Classify intent: submission | question | feedback | spam
    //   3. If submission: create entry in submissions table
    //   4. If question: draft response, set status = "escalated" for steward
    //   5. If spam: set status = "responded" with a note

    Ok(items.len())
}
