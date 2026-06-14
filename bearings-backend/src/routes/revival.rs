//! POST /api/revival/:kind/:id — content-free "would you return?" signal on a
//! closed venue in the Archive "Gone but not forgotten" memorial. Atomic via the
//! increment_revival_votes(kind, id) Postgres function. Returns the replacement
//! button HTML (HTMX outerHTML swap). Not a governance vote — a soft demand signal.

use crate::db::SupabaseClient;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// POST /api/revival/:kind/:id  (kind = place | club)
pub async fn vote(
    State(db): State<SupabaseClient>,
    Path((kind, id)): Path<(String, i64)>,
) -> Response {
    if kind != "place" && kind != "club" {
        return (StatusCode::BAD_REQUEST, Html("bad kind".to_string())).into_response();
    }
    let body = serde_json::json!({ "p_kind": kind, "p_id": id });
    let new_count: Option<i64> = match db.post_rpc("increment_revival_votes", &body).await {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("error".to_string())).into_response()
        }
    };
    match new_count {
        Some(c) => Html(format!(
            "<button disabled title=\"Signal recorded\" \
               style=\"font-size:11px;color:#fff;background:#D2691E;border:1px solid #D2691E;\
                       border-radius:20px;padding:3px 11px;cursor:default\">\u{25B2} {c} would return</button>"
        ))
        .into_response(),
        None => (StatusCode::NOT_FOUND, Html("not found".to_string())).into_response(),
    }
}
