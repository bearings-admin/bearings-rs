
//! AI crawlability layer.
//! GET /llms.txt      — brief summary for AI context windows
//! GET /llms-full.txt — rich multi-resource index using DB ai_* views
//!
//! The ai_* views in Supabase are pre-built plain-text optimised summaries:
//!   ai_event_summary, ai_place_summary, ai_title_summary,
//!   ai_campaign_summary, ai_history_summary, ai_creator_summary
//!
//! These replace the previous approach of fetching raw events and
//! formatting them in Rust — the DB does it better.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use crate::{db::SupabaseClient, error::AppError};

/// GET /llms.txt — brief platform summary, fits in most AI context windows
pub async fn llms_txt() -> impl IntoResponse {
    let content = r#"# Bearings — Global Bear Community Infrastructure

Bearings is a verified directory of the global gay bear community.

## What we contain
- Events: bear runs, festivals, cruises, socials (88 active events, 25+ countries)
- Places: bars, leather bars, saunas, campgrounds (173 venues)
- Clubs: community clubs and organising associations (49 clubs)
- Title holders: competition winners 1992–present (148 records, IBR complete)
- Competitions: 27 active bear title competitions worldwide
- Creators: musicians, DJs, authors, illustrators, filmmakers, historians (46 verified)
- Digital spaces: apps, Discord, podcasts, Twitch (35 spaces)
- Bear history: community milestones 1987–present (58 records)
- Campaigns: active charity fundraising (12 campaigns)
- Shops: bear-owned shops, gear, and books by bear authors (24 shops)

## API endpoints
GET /api/events            — all upcoming events (?country=X &month=September)
GET /api/events/ical.ics   — iCal feed, subscribable by URL
GET /api/events/by-month   — event density by month (powers timeline chart)
GET /api/places            — all active venues (?country=X &place_type=bar)
GET /api/clubs             — community clubs (?country=X)
GET /api/title-holders     — competition archive (?title_name=X &year=X)
GET /api/title-holders/current — current holders per competition
GET /api/creators          — community creators (?creator_type=musician)
GET /api/digital-spaces    — apps, Discord, podcasts (?space_type=discord)
GET /api/stories           — oral histories and community essays
GET /api/bear-history      — community milestones 1987–present
GET /api/campaigns         — active charity campaigns
GET /api/treasury          — community treasury status (planned, not yet funded)
GET /api/bear-future       — community governance proposals
GET /api/inclusion-flags   — CONST-10 inclusion flag reference

## MCP server (for AI agents)
A read-only Model Context Protocol server is live at POST /mcp (JSON-RPC over HTTP).
Point any MCP client at https://srv1744879.hstgr.cloud/mcp to query the directory.
Tools: search_events, list_places, current_title_holders, list_clubs, list_creators,
list_campaigns, list_digital_spaces.

## Governance
Community-governed. Token: NORTH (Cardano). 100 verified holders = full DAO.
Legal: unincorporated association. Contact: ursasteward@pm.me
"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        content,
    )
}

/// GET /llms-full.txt — rich content index using Supabase ai_* views
/// These views are optimised for AI consumption — plain text, pre-formatted,
/// with human-readable dates and bearings_url links.
pub async fn llms_full_txt(
    State(db): State<SupabaseClient>,
) -> Result<Response, AppError> {
    // Fetch from all ai_* views concurrently
    let url_events    = format!("{}/rest/v1/ai_event_summary?select=*&limit=50", db.url);
    let url_places    = format!("{}/rest/v1/ai_place_summary?select=*&limit=50", db.url);
    let url_titles    = format!("{}/rest/v1/ai_title_summary?select=*", db.url);
    let url_campaigns = format!("{}/rest/v1/ai_campaign_summary?select=*", db.url);
    let (events, places, titles, campaigns) = tokio::try_join!(
        db.get_json::<Vec<serde_json::Value>>(&url_events),
        db.get_json::<Vec<serde_json::Value>>(&url_places),
        db.get_json::<Vec<serde_json::Value>>(&url_titles),
        db.get_json::<Vec<serde_json::Value>>(&url_campaigns),
    )?;

    // Format events
    let event_lines: String = events.iter().map(|e| {
        format!(
            "- {} | {} | {} to {} | {}\n",
            e["name"].as_str().unwrap_or(""),
            e["location"].as_str().unwrap_or(""),
            e["starts"].as_str().unwrap_or(""),
            e["ends"].as_str().unwrap_or(""),
            e["bearings_url"].as_str().unwrap_or(""),
        )
    }).collect();

    // Format places (first 30 — bear_popular first)
    let place_lines: String = places.iter().take(30).map(|p| {
        let note = p["access_note"].as_str().unwrap_or("");
        let note_str = if note.is_empty() { "".to_string() } else { format!(" [{}]", note) };
        format!(
            "- {} ({}) | {}{} | {}\n",
            p["name"].as_str().unwrap_or(""),
            p["place_type"].as_str().unwrap_or(""),
            p["location"].as_str().unwrap_or(""),
            note_str,
            p["bearings_url"].as_str().unwrap_or(""),
        )
    }).collect();

    // Format current title holders
    let title_lines: String = titles.iter().map(|t| {
        format!(
            "- {} {} ({}) | {} | {}\n",
            t["title_name"].as_str().unwrap_or(""),
            t["year"].as_i64().unwrap_or(0),
            t["scope"].as_str().unwrap_or(""),
            t["holder_name"].as_str().unwrap_or(""),
            t["holder_location"].as_str().unwrap_or(""),
        )
    }).collect();

    // Format campaigns
    let campaign_lines: String = campaigns.iter().map(|c| {
        format!(
            "- {} | {} | {}\n",
            c["name"].as_str().unwrap_or(""),
            c["org"].as_str().unwrap_or(""),
            c["donate_link"].as_str().unwrap_or(""),
        )
    }).collect();

    let content = format!(
        "# Bearings — Full Content Index\n\n\
         ## Upcoming Events\n{}\n\
         ## Bear Venues (bear-popular first)\n{}\n\
         ## Current Title Holders\n{}\n\
         ## Active Campaigns\n{}\n\
         ## Full Platform\nhttps://bearings.lovable.app\n",
        event_lines, place_lines, title_lines, campaign_lines
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        content,
    ).into_response())
}

/// GET /robots.txt — points AI crawlers to /llms.txt
pub async fn robots_txt() -> impl IntoResponse {
    let content = "User-agent: *\nAllow: /\n\nUser-agent: *\nSitemap: https://www.bearings.community/llms.txt\n";
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        content,
    )
}
