//! Upvote endpoint for "What Could Be" ideas on the Bear Future page.
//!
//! POST /api/future-ideas/:id/upvote — lightweight community signal (not a
//! governance vote). Returns the replacement HTML for the upvote button
//! (HTMX outerHTML swap). Data access in `repositories::future_idea_repo`.
//!
//! Note: the read-then-write upvote is not concurrency-safe; a future SQL
//! `increment_upvote(id)` RPC would make it atomic. Acceptable for a soft signal.

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    http::StatusCode,
};
use crate::db::SupabaseClient;
use crate::repositories::future_idea_repo::{FutureIdeaRepository, SupabaseFutureIdeaRepository};

const ORANGE: &str = "#D2691E";

/// POST /api/future-ideas/:id/upvote
pub async fn upvote(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Response {
    let repo = SupabaseFutureIdeaRepository::new(db);

    let new_count = match repo.increment_upvotes(id).await {
        Ok(Some(c)) => c,
        Ok(None)    => return (StatusCode::NOT_FOUND, Html("not found".to_string())).into_response(),
        Err(_)      => return (StatusCode::INTERNAL_SERVER_ERROR, Html("error".to_string())).into_response(),
    };

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
