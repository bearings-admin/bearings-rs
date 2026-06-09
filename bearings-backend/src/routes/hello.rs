
//! Gaspar''s original route — preserved as the foundation.
//! GET /hello/:name → "Hello, {name}! Welcome to Bearings."
//! This is where the server started. Everything else is built on this.

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

pub async fn handler(Path(name): Path<String>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!("Hello, {}! Welcome to Bearings.", name),
    )
}
