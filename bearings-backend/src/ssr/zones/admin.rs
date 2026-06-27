//! Zone: admin

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_admin(
    db: SupabaseClient,
    token: Option<String>,
    action: Option<String>,
    id: Option<i64>,
    id2: Option<i64>,
    lang: &str,
) -> Response {
    let expected = std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "bearings-admin".to_string());
    // Timing-safe token check: compare all bytes regardless of early mismatch
    // to mitigate timing oracle attacks on the token.
    fn token_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
    }
    if !token_eq(token.as_deref().unwrap_or(""), &expected) {
        return Html("<html><body style=\"font-family:sans-serif;padding:40px\"><h2>Bearings Admin</h2><p>Pass <code>?zone=admin&amp;token=YOUR_TOKEN</code></p></body></html>".to_string()).into_response();
    }

    // Process an approve/reject action, then redirect to the clean queue URL.
    if let (Some(act), Some(cid)) = (action.as_deref(), id) {
        match act {
            "reject" => {
                let _ = db
                    .write_json(
                        reqwest::Method::PATCH,
                        &format!("{}/rest/v1/candidate_events?id=eq.{cid}", db.url),
                        &serde_json::json!({ "status": "rejected" }),
                    )
                    .await;
            }
            "approve" => approve_candidate(&db, cid).await,
            "dup_archive" => {
                let _ = db
                    .write_json(
                        reqwest::Method::PATCH,
                        &format!("{}/rest/v1/events?id=eq.{cid}", db.url),
                        &serde_json::json!({
                            "active": false,
                            "archive_notes": "Archived via admin duplicate review."
                        }),
                    )
                    .await;
            }
            "dup_ignore" => {
                if let Some(other) = id2 {
                    let (lo, hi) = if cid < other {
                        (cid, other)
                    } else {
                        (other, cid)
                    };
                    let _ = db
                        .write_json(
                            reqwest::Method::POST,
                            &format!("{}/rest/v1/event_dupe_ignores", db.url),
                            &serde_json::json!({ "lo_id": lo, "hi_id": hi }),
                        )
                        .await;
                }
            }
            // Reverse a keeper auto-apply: archive the live event (id2) — never
            // delete (Archive Principle) — and revert the candidate so the agent's
            // autonomous write is fully undone and auditable.
            "undo_auto" => {
                if let Some(eid) = id2 {
                    let _ = db
                        .write_json(
                            reqwest::Method::PATCH,
                            &format!("{}/rest/v1/events?id=eq.{eid}", db.url),
                            &serde_json::json!({
                                "active": false,
                                "archive_notes": "Keeper auto-apply undone by steward."
                            }),
                        )
                        .await;
                }
                let _ = db
                    .write_json(
                        reqwest::Method::PATCH,
                        &format!("{}/rest/v1/candidate_events?id=eq.{cid}", db.url),
                        &serde_json::json!({
                            "status": "rejected",
                            "steward_notes": "auto-apply undone by steward"
                        }),
                    )
                    .await;
            }
            // Keeper lineage-harvest proposal (titleholders): approve → insert into
            // title_holders (idempotent claim), or reject.
            "approve_th" => approve_title_holder(&db, cid).await,
            "reject_th" => {
                let _ = db
                    .write_json(
                        reqwest::Method::PATCH,
                        &format!("{}/rest/v1/candidate_title_holders?id=eq.{cid}", db.url),
                        &serde_json::json!({ "status": "rejected" }),
                    )
                    .await;
            }
            _ => {}
        }
        let t = urlencoding::encode(token.as_deref().unwrap_or(""));
        return axum::response::Redirect::to(&format!("/?zone=admin&token={t}")).into_response();
    }

    let candidates_url = format!(
        "{}/rest/v1/candidate_events?status=eq.pending&select=id,raw_title,raw_description,raw_date,parsed_country,parsed_city,parsed_start,parsed_end,parsed_type,source_url,created_at&order=created_at.desc&limit=50",
        db.url
    );
    let feeds_url = format!(
        "{}/rest/v1/watched_feeds?active=eq.true&select=id,org_name,feed_type,last_fetched,fetch_errors&order=id.asc",
        db.url
    );
    let dupes_url = format!(
        "{}/rest/v1/event_dupe_candidates?select=*&limit=100",
        db.url
    );
    let preds_url = format!(
        "{}/rest/v1/event_predictions?select=sample_name,city,country,predicted_date,confidence,website&order=predicted_date&limit=60",
        db.url
    );
    let auto_url = format!(
        "{}/rest/v1/candidate_events?status=eq.auto_applied&select=id,raw_title,raw_description,parsed_country,parsed_city,parsed_start,source_url,event_id,reviewed_at&order=reviewed_at.desc&limit=25",
        db.url
    );
    // Titleholder proposals are read with the SERVICE key — the table holds identity
    // data and has no public-read policy (unlike candidate_events).
    let th_url = format!(
        "{}/rest/v1/candidate_title_holders?status=eq.pending&select=id,title_name,holder_name,year,city,country,competition_id,evidence&order=title_name.asc,year.asc&limit=80",
        db.url
    );

    let (cands_res, feeds_res, dupes_res, preds_res, auto_res, th_res) = tokio::join!(
        db.get_json::<Vec<CandidateEventRow>>(&candidates_url),
        db.get_json::<Vec<WatchedFeedRow>>(&feeds_url),
        db.get_json::<Vec<DupePairRow>>(&dupes_url),
        db.get_json::<Vec<PredictionRow>>(&preds_url),
        db.get_json::<Vec<CandidateEventRow>>(&auto_url),
        db.get_json_service::<Vec<CandidateTitleHolderRow>>(&th_url),
    );

    let candidates = cands_res.or_log("admin:cands_res");
    let feeds = feeds_res.or_log("admin:feeds_res");
    let dupes = dupes_res.or_log("admin:dupes_res");
    let preds = preds_res.or_log("admin:preds_res");
    let auto_applied = auto_res.or_log("admin:auto_res");
    let th_props = th_res.or_log("admin:th_res");

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
            let date  = esc(c.parsed_start.as_deref().or(c.raw_date.as_deref()).unwrap_or(""));
            let ctry  = esc(c.parsed_country.as_deref().unwrap_or(""));
            let url   = esc(c.source_url.as_deref().unwrap_or("#"));
            let snip  = desc.chars().take(220).collect::<String>();
            let tok   = &expected;
            card(&format!(
                "<div><div style=\"font-weight:600;font-size:14px;margin-bottom:2px\">{title}</div><div style=\"font-size:11px;color:{MID};margin-bottom:6px\">{date} · {ctry}</div><div style=\"font-size:12px;line-height:1.5;margin-bottom:8px\">{snip}</div><div style=\"display:flex;gap:8px\"><a href=\"{url}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\" style=\"font-size:11px\">Source \u{2197}</a><a href=\"/?zone=admin&token={tok}&action=approve&id={id}\" class=\"btn-g\" style=\"font-size:11px\">\u{2713} Approve</a><a href=\"/?zone=admin&token={tok}&action=reject&id={id}\" style=\"font-size:11px;color:{MID};padding:6px 0\">Reject</a></div></div>"
            ))
        }).collect()
    };

    let dupe_cards: String = if dupes.is_empty() {
        format!("<div style=\"padding:16px;text-align:center;color:{MID};font-size:13px\">No likely duplicates right now.</div>")
    } else {
        dupes.iter().map(|d| {
            let tok = &expected;
            let side = |id: i64, name: &str, city: Option<&str>, date: Option<&str>| -> String {
                format!(
                    "<div style=\"flex:1;min-width:0\"><div style=\"font-size:13px;font-weight:600\">{n}</div>\
                     <div style=\"font-size:11px;color:{MID}\">#{id} \u{00b7} {c} \u{00b7} {dt}</div></div>",
                    n = esc(name), c = esc(city.unwrap_or("\u{2014}")), dt = esc(date.unwrap_or("no date")),
                )
            };
            let a = side(d.id_a, &d.name_a, d.city_a.as_deref(), d.date_a.as_deref());
            let b = side(d.id_b, &d.name_b, d.city_b.as_deref(), d.date_b.as_deref());
            let sim = esc(d.sim.as_deref().unwrap_or(""));
            card(&format!(
                "<div><div style=\"display:flex;gap:10px;align-items:flex-start;margin-bottom:8px\">\
                   {a}<span style=\"font-size:10px;color:{MID};white-space:nowrap;padding-top:2px\">sim {sim}</span>{b}</div>\
                 <div style=\"display:flex;gap:8px;flex-wrap:wrap\">\
                   <a href=\"/?zone=admin&token={tok}&action=dup_archive&id={ib}\" class=\"btn-g\" style=\"font-size:11px\">Keep #{ia} \u{2014} archive #{ib}</a>\
                   <a href=\"/?zone=admin&token={tok}&action=dup_archive&id={ia}\" class=\"btn-g\" style=\"font-size:11px\">Keep #{ib} \u{2014} archive #{ia}</a>\
                   <a href=\"/?zone=admin&token={tok}&action=dup_ignore&id={ia}&id2={ib}\" style=\"font-size:11px;color:{MID};padding:6px 8px\">Not a duplicate</a>\
                 </div></div>",
                ia = d.id_a, ib = d.id_b,
            ))
        }).collect()
    };

    let pred_cards: String = if preds.is_empty() {
        format!("<div style=\"padding:16px;text-align:center;color:{MID};font-size:13px\">No predicted repeats yet \u{2014} needs multi-year history.</div>")
    } else {
        preds.iter().map(|p| {
            let name = esc(p.sample_name.as_deref().unwrap_or("(unknown series)"));
            let city = esc(p.city.as_deref().unwrap_or(""));
            let date = esc(p.predicted_date.as_deref().unwrap_or(""));
            let conf = esc(p.confidence.as_deref().unwrap_or(""));
            let badge = match p.confidence.as_deref() { Some("high") => ORANGE, _ => GOLD };
            let site = esc(p.website.as_deref().unwrap_or(""));
            let verify = if !site.is_empty() && site != "#" {
                format!("<div style=\"margin-top:3px\"><a href=\"{site}\" target=\"_blank\" rel=\"noopener\" style=\"font-size:10px;color:{ORANGE}\">verify next date \u{2197}</a></div>")
            } else {
                String::new()
            };
            card(&format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center;gap:8px\">\
                   <div><div style=\"font-size:14px;font-weight:600\">{name}</div>\
                     <div style=\"font-size:11px;color:{MID}\">{city} \u{00b7} likely ~ {date} \u{00b7} no confirmed edition yet</div>{verify}</div>\
                   <span style=\"font-size:10px;color:#fff;background:{badge};border-radius:6px;padding:2px 7px;white-space:nowrap\">{conf}</span>\
                 </div>"
            ))
        }).collect()
    };

    // Oversight surface: events the keeper auto-applied autonomously, each with a
    // one-click undo (archive the event + revert the candidate).
    let auto_cards: String = if auto_applied.is_empty() {
        format!("<div style=\"padding:16px;text-align:center;color:{MID};font-size:13px\">Nothing auto-applied by the keeper yet.</div>")
    } else {
        auto_applied.iter().map(|c| {
            let cid   = c.id.unwrap_or(0);
            let eid   = c.event_id.unwrap_or(0);
            let title = esc(c.raw_title.as_deref().unwrap_or(""));
            let date  = esc(c.parsed_start.as_deref().unwrap_or(""));
            let city  = esc(c.parsed_city.as_deref().unwrap_or(""));
            let ctry  = esc(c.parsed_country.as_deref().unwrap_or(""));
            let when  = esc(c.reviewed_at.as_deref().unwrap_or("").get(..10).unwrap_or(""));
            let url   = esc(c.source_url.as_deref().unwrap_or("#"));
            let snip  = esc(c.raw_description.as_deref().unwrap_or("")).chars().take(200).collect::<String>();
            let tok   = &expected;
            card(&format!(
                "<div><div style=\"display:flex;justify-content:space-between;gap:8px;align-items:baseline\">\
                   <div style=\"font-weight:600;font-size:14px\">{title}</div>\
                   <span style=\"font-size:10px;color:#fff;background:{GOLD};border-radius:6px;padding:2px 7px;white-space:nowrap\">auto-applied {when}</span></div>\
                 <div style=\"font-size:11px;color:{MID};margin:2px 0 6px\">{date} \u{00b7} {city} {ctry} \u{00b7} live event #{eid}</div>\
                 <div style=\"font-size:12px;line-height:1.5;margin-bottom:8px\">{snip}</div>\
                 <div style=\"display:flex;gap:8px\">\
                   <a href=\"{url}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\" style=\"font-size:11px\">Source \u{2197}</a>\
                   <a href=\"/?zone=admin&token={tok}&action=undo_auto&id={cid}&id2={eid}\" style=\"font-size:11px;color:#C0392B;padding:6px 8px\">\u{21a9} Undo (archive event)</a>\
                 </div></div>"
            ))
        }).collect()
    };

    // Keeper lineage-harvest proposals (titleholders) awaiting review.
    let th_cards: String = if th_props.is_empty() {
        format!("<div style=\"padding:16px;text-align:center;color:{MID};font-size:13px\">No pending titleholder proposals.</div>")
    } else {
        th_props.iter().map(|c| {
            let id    = c.id.unwrap_or(0);
            let title = esc(c.title_name.as_deref().unwrap_or(""));
            let who   = esc(c.holder_name.as_deref().unwrap_or(""));
            let yr    = c.year.map(|y| y.to_string()).unwrap_or_default();
            let mut loc_parts: Vec<String> = Vec::new();
            if let Some(s) = c.city.as_deref().filter(|s| !s.is_empty()) { loc_parts.push(esc(s)); }
            if let Some(s) = c.country.as_deref().filter(|s| !s.is_empty()) { loc_parts.push(esc(s)); }
            let loc = loc_parts.join(", ");
            let meta = if loc.is_empty() { who.clone() } else { format!("{who} \u{00b7} {loc}") };
            let ev    = esc(c.evidence.as_deref().unwrap_or(""));
            let tok   = &expected;
            card(&format!(
                "<div><div style=\"font-weight:600;font-size:14px\">{title} {yr}</div>\
                 <div style=\"font-size:12px;color:{DARK};margin:2px 0\">{meta}</div>\
                 <div style=\"font-size:11px;color:{MID};line-height:1.5;margin-bottom:8px\">{ev}</div>\
                 <div style=\"display:flex;gap:8px\">\
                   <a href=\"/?zone=admin&token={tok}&action=approve_th&id={id}\" class=\"btn-g\" style=\"font-size:11px\">\u{2713} Approve</a>\
                   <a href=\"/?zone=admin&token={tok}&action=reject_th&id={id}\" style=\"font-size:11px;color:{MID};padding:6px 0\">Reject</a>\
                 </div></div>"
            ))
        }).collect()
    };

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Admin â Feed Review</h1><p style=\"font-size:12px;color:{MID};margin-bottom:16px\">Candidates from the nightly feed reader.</p>{h_feeds}<div style=\"overflow-x:auto;margin-bottom:16px\"><table style=\"width:100%;border-collapse:collapse\"><thead><tr style=\"border-bottom:1px solid {TAN}\"><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Feed</th><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Type</th><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Last fetched</th><th style=\"text-align:left;padding:4px 8px;font-size:11px;color:{MID}\">Errors</th></tr></thead><tbody>{feed_rows}</tbody></table></div>{h_auto}{auto_cards}{h_th}{th_cards}{h_preds}{pred_cards}{h_dupes}{dupe_cards}{h_cands}{cand_cards}",
        h_feeds = sh("Watched Feeds", Some(feeds.len())),
        h_auto = sh("Auto-applied by the keeper", Some(auto_applied.len())),
        h_th = sh("Pending Titleholder Proposals", Some(th_props.len())),
        h_dupes = sh("Possible Duplicates", Some(dupes.len())),
        h_preds = sh("Likely Repeats \u{2014} confirm/chase", Some(preds.len())),
        h_cands = sh("Pending Candidates", Some(candidates.len())),
    );

    Html(shell("Admin", "Feed review.", "archive", &body, lang)).into_response()
}

/// Promote a pending candidate event into a live `events` row, then mark it
/// approved. Writes use the service key (RLS-bypassing) via `db.write_json`.
///
/// Idempotent under double-clicks: the candidate is *atomically claimed* by
/// flipping `pending -> approved` with a `status=eq.pending` filter and
/// `return=representation`. Only the first of two concurrent (or lagged) clicks
/// gets the row back; the rest find nothing pending and no-op — so a candidate
/// can only ever produce one event.
async fn approve_candidate(db: &SupabaseClient, id: i64) {
    let claim_url = format!(
        "{}/rest/v1/candidate_events?id=eq.{id}&status=eq.pending&select=raw_title,raw_description,raw_date,parsed_country,parsed_city,parsed_start,parsed_end,parsed_type,source_url",
        db.url
    );
    let claimed: Vec<CandidateEventRow> = db
        .write_json_returning(
            reqwest::Method::PATCH,
            &claim_url,
            &serde_json::json!({ "status": "approved" }),
        )
        .await
        .unwrap_or_default();
    let Some(c) = claimed.into_iter().next() else {
        return; // already approved/claimed by an earlier click — nothing to do
    };
    // Prefer a clean parsed date; fall back to the raw YYYYMMDD form.
    let start = c
        .parsed_start
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            c.raw_date
                .as_deref()
                .filter(|d| d.len() >= 8)
                .map(|d| format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..8]))
        });
    let etype = c
        .parsed_type
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "bear-run".to_string());
    let link = c
        .source_url
        .as_deref()
        .map(|s| s.split('#').next().unwrap_or(s).to_string());
    let event = serde_json::json!({
        "name": c.raw_title.unwrap_or_default(),
        "description": c.raw_description,
        "country": c.parsed_country,
        "city": c.parsed_city,
        "start_date": start,
        "end_date": c.parsed_end,
        "link": link,
        "type": etype,
        "active": true,
        "source": "admin-approved",
    });
    let created: Vec<serde_json::Value> = db
        .write_json_returning(
            reqwest::Method::POST,
            &format!("{}/rest/v1/events", db.url),
            &event,
        )
        .await
        .unwrap_or_default();
    // Record the link candidate -> event (also powers the auto-applied/undo view).
    if let Some(eid) = created
        .into_iter()
        .next()
        .and_then(|v| v.get("id").and_then(|x| x.as_i64()))
    {
        let _ = db
            .write_json(
                reqwest::Method::PATCH,
                &format!("{}/rest/v1/candidate_events?id=eq.{id}", db.url),
                &serde_json::json!({ "event_id": eid }),
            )
            .await;
    }
}

/// Promote a pending titleholder proposal into a live `title_holders` row, then
/// link it back. Idempotent under double-clicks via an atomic `pending -> approved`
/// claim (only the first click gets the row).
async fn approve_title_holder(db: &SupabaseClient, id: i64) {
    let claim_url = format!(
        "{}/rest/v1/candidate_title_holders?id=eq.{id}&status=eq.pending&select=title_name,holder_name,year,city,country,competition_id,evidence",
        db.url
    );
    let claimed: Vec<CandidateTitleHolderRow> = db
        .write_json_returning(
            reqwest::Method::PATCH,
            &claim_url,
            &serde_json::json!({ "status": "approved" }),
        )
        .await
        .unwrap_or_default();
    let Some(c) = claimed.into_iter().next() else {
        return; // already approved/claimed by an earlier click
    };
    let bio = c
        .evidence
        .as_deref()
        .map(|e| format!("Source (keeper lineage harvest): {e}"));
    let holder = serde_json::json!({
        "title_name": c.title_name,
        "holder_name": c.holder_name,
        "year": c.year,
        "city": c.city,
        "country": c.country,
        "competition_id": c.competition_id,
        "holder_status": "active",
        "bio": bio,
        "active": true,
    });
    let created: Vec<serde_json::Value> = db
        .write_json_returning(
            reqwest::Method::POST,
            &format!("{}/rest/v1/title_holders", db.url),
            &holder,
        )
        .await
        .unwrap_or_default();
    if let Some(thid) = created
        .into_iter()
        .next()
        .and_then(|v| v.get("id").and_then(|x| x.as_i64()))
    {
        let _ = db
            .write_json(
                reqwest::Method::PATCH,
                &format!("{}/rest/v1/candidate_title_holders?id=eq.{id}", db.url),
                &serde_json::json!({ "title_holder_id": thid }),
            )
            .await;
    }
}
