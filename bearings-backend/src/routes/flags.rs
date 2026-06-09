
//! Inclusion flags — CONST-10: inclusion is shown, not decided.
//!
//! Every event and place can carry inclusion_flag_codes.
//! This route exposes the flag reference table and the events_with_flags view.
//!
//! Flag examples: MASC_ONLY, AGE_RESTRICTION, NO_BEARS, NO_FEMS
//!
//! The frontend uses these to show context, never to hide listings.

use axum::extract::{Query, State};
use axum::Json;
use bearings_shared::models::{Event, InclusionFlag};
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

/// GET /api/inclusion-flags
/// Returns the reference table of all flag codes with labels and descriptions.
pub async fn list_flags(
    State(db): State<SupabaseClient>,
) -> Result<Json<Vec<InclusionFlag>>, AppError> {
    let url = format!(
        "{}/rest/v1/inclusion_flags?select=*&active=eq.true&order=severity.asc,code.asc",
        db.url
    );
    Ok(Json(db.get_json::<Vec<InclusionFlag>>(&url).await?))
}

#[derive(Deserialize)]
pub struct FlaggedEventsQuery {
    pub flag_code: Option<String>,
    pub country: Option<String>,
    pub limit: Option<u32>,
}

/// GET /api/events/flagged
/// Returns events that carry at least one inclusion flag, with flag context.
/// Powers the CONST-10 "show the reality" view in the frontend.
/// Uses the events_with_flags view — pre-built in Supabase.
pub async fn flagged_events(
    State(db): State<SupabaseClient>,
    Query(params): Query<FlaggedEventsQuery>,
) -> Result<Json<Vec<Event>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200);

    // Query the events_with_flags view — has has_flags and flag_count computed columns
    let mut url = format!(
        "{}/rest/v1/events_with_flags?select=*&has_flags=eq.true&order=start_date.asc&limit={}",
        db.url, limit
    );

    if let Some(c) = params.country { url.push_str(&format!("&country=eq.{}", c)); }

    // PostgREST array contains: cs.{"VALUE"} not cs.["VALUE"]
    // URL-encoded curly braces: %7B and %7D
    if let Some(flag) = params.flag_code {
        url.push_str(&format!(
            "&inclusion_flag_codes=cs.%7B%22{}%22%7D",
            urlencoding::encode(&flag)
        ));
    }

    Ok(Json(db.get_json::<Vec<Event>>(&url).await?))
}
