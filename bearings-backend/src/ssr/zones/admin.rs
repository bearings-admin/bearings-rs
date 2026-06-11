//! Zone: admin

use axum::response::{Html, IntoResponse, Response};
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;
use super::super::query::*;

pub(crate) async fn zone_admin(db: SupabaseClient, token: Option<String>, lang: &str) -> Response {
    let expected = std::env::var("ADMIN_TOKEN")
        .unwrap_or_else(|_| "bearings-admin".to_string());
    // Timing-safe token check: compare all bytes regardless of early mismatch
    // to mitigate timing oracle attacks on the token.
    fn token_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() { return false; }
        a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
    }
    if !token_eq(token.as_deref().unwrap_or(""), &expected) {
        return Html("<html><body style=\"font-family:sans-serif;padding:40px\"><h2>Bearings Admin</h2><p>Pass <code>?zone=admin&amp;token=YOUR_TOKEN</code></p></body></html>".to_string()).into_response();
    }

    let candidates_url = format!(
        "{}/rest/v1/candidate_events?status=eq.pending&select=id,raw_title,raw_description,raw_date,parsed_country,source_url,created_at&order=created_at.desc&limit=50",
        db.url
    );
    let feeds_url = format!(
        "{}/rest/v1/watched_feeds?active=eq.true&select=id,org_name,feed_type,last_fetched,fetch_errors&order=id.asc",
        db.url
    );

    let (cands_res, feeds_res) = tokio::join!(
        db.get_json::<Vec<CandidateEventRow>>(&candidates_url),
        db.get_json::<Vec<WatchedFeedRow>>(&feeds_url),
    );

    let candidates = cands_res.or_log("admin:cands_res");
    let feeds      = feeds_res.or_log("admin:feeds_res");

    let feed_rows: String = feeds.iter().map(|f| {
        let name    = esc(f.org_name.as_deref().unwrap_or(""));
        let ftype   = esc(f.feed_type.as_deref().unwrap_or(""));
        let fetched = esc(f.last_fetched.as_deref().unwrap_or("never"));
        let errors  = f.fetch_errors.unwrap_or(0);
        let err_col = if errors > 0 { "#C0392B" } else { MID };
        format!(
            "<tr><td style=\"padding:4px 8px;font-size:12px\">{name}</td><td style=\"padding:4px 8px;font-size:11px;color:{MID}\">{ftype}</td><td style=\"padding:4px 8px;font-size:11px;color:{MID}\">{fetched}</td><td style=\"padding:4px 8px;font-size:11px;color:{err_col}\">{errors}</td></tr>"
        )
    }).collect();

    let cand_cards: String = if candidates.is_empty() {
        format!("<div style=\"padding:24px;text-align:center;color:{MID};font-size:13px\">No pending candidates — queue is clear.</div>")
    } else {
        candidates.iter().map(|c| {
            let id    = c.id.unwrap_or(0);
            let title = esc(c.raw_title.as_deref().unwrap_or(""));
            let desc  = esc(c.raw_description.as_deref().unwrap_or(""));
            let date  = esc(c.raw_date.as_deref().unwrap_or(""));
            let ctry  = esc(c.parsed_country.as_deref().unwrap_or(""));
            let url   = esc(c.source_url.as_deref().unwrap_or("#"));
            let snip  = desc.chars().take(220).collect::<String>();
            let tok   = &expected;
            card(&format!(
                "<div><div style=\"font-weight:600;font-size:14px;margin-bottom:2px\">{title}</div><div style=\"font-size:11px;color:{MID};margin-bottom:6px\">{date} · {ctry}</div><div style=\"font-size:12px;line-height:1.5;margin-bottom:8px\">{snip}</div><div style=\"display:flex;gap:8px\"><a href=\"{url}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\" style=\"font-size:11px\">Source \u{2197}</a><a href=\"/?zone=admin&token={tok}&action=approve&id={id}\" class=\"btn-g\" style=\"font-size:11px\">\u{2713} Approve</a><a href=\"/?zone=admin&token={tok}&action=reject&id={id}\" style=\"font-size:11px;color:{MID};padding:6px 0\">Reject</a></div></div>"
            ))
        }).collect()
    };

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Admin â Feed Review</h1><p style=\"font-size:12px;color:{MID};margin-bottom:16px\">Candidates from the nightly feed reader.</p>{h_feeds}<div style=\"overflow-x:auto;margin-bottom:16px\"><table style=\"width:100%;border-collapse:collapse\"><thead><tr style=\"border-bottom:1px solid {TAN}\"><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Feed</th><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Type</th><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Last fetched</th><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Errors</th></tr></thead><tbody>{feed_rows}</tbody></table></div>{h_cands}{cand_cards}",
        h_feeds = sh("Watched Feeds", Some(feeds.len())),
        h_cands = sh("Pending Candidates", Some(candidates.len())),
    );

    Html(shell("Admin", "Feed review.", "archive", &body, lang)).into_response()
}
