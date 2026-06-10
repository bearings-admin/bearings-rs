//! Upvote endpoint for "What Could Be" ideas on the Bear Future page.
//!
//! POST /api/future-ideas/:id/upvote
//!
//! No authentication required — this is a lightweight community signal,
//! not a governance vote. The response is the replacement HTML for the
//! upvote button (HTMX outerHTML swap), showing the new count.
//!
//! Rate limiting: the DB constraint upvotes >= 0 is the only guard.
//! Client-side: the onclick immediately greys the button to discourage
//! repeat tapping, and localStorage tracks voted IDs.

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    http::StatusCode,
};
use crate::db::SupabaseClient;

const ORANGE: &str = "#D2691E";
const BROWN:  &str = "#5C4033";
const TAN:    &str = "#C8B89A";
const OFF_WHITE: &str = "#F9F5F0";

/// POST /api/future-ideas/:id/upvote
/// Increments the upvote count and returns replacement button HTML.
pub async fn upvote(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Response {
    // Fetch current count
    let fetch_url = format!(
        "{}/rest/v1/future_ideas?id=eq.{}&select=id,upvotes,title&active=eq.true&limit=1",
        db.url, id
    );
    let rows: Vec<serde_json::Value> = db.get_json(&fetch_url).await.unwrap_or_default();
    let row = match rows.into_iter().next() {
        Some(r) => r,
        None    => return (StatusCode::NOT_FOUND, Html("not found".to_string())).into_response(),
    };

    let current = row["upvotes"].as_i64().unwrap_or(0);
    let new_count = current + 1;

    // Increment via PATCH
    let patch_url = format!("{}/rest/v1/future_ideas?id=eq.{}", db.url, id);
    let body = serde_json::json!({ "upvotes": new_count });
    let _ = db.write_json(reqwest::Method::PATCH, &patch_url, &body).await;

    // Return the replacement button HTML (HTMX swaps this in)
    let html = format!(
        "<div hx-post=\"/api/future-ideas/{id}/upvote\"\
             hx-swap=\"outerHTML\"\
             hx-target=\"this\"\
             style=\"flex-shrink:0;display:flex;flex-direction:column;\
                     align-items:center;padding:6px 10px;\
                     border-radius:10px;border:1px solid {ORANGE};\
                     background:{ORANGE};color:#fff;\
                     user-select:none;min-width:44px;cursor:default\"\
             title=\"Voted!\">\
          <span style=\"font-size:16px;line-height:1\">▲</span>\
          <span style=\"font-size:12px;font-weight:700;margin-top:2px\">{new_count}</span>\
        </div>"
    );

    Html(html).into_response()
}
