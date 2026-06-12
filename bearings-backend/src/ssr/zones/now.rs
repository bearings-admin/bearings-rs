//! Zone: now

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
use chrono::Utc;

pub(crate) async fn zone_now(db: SupabaseClient, lang: &str) -> Response {
    // Worldwide events starting within the next 30 days
    let today = Utc::now().date_naive();
    let in_30 = today
        .checked_add_days(chrono::Days::new(30))
        .unwrap_or(today);
    let from_s = today.format("%Y-%m-%d").to_string();
    let to_s = in_30.format("%Y-%m-%d").to_string();

    let events_url = format!(
        "{}/rest/v1/events\
         ?active=eq.true\
         &start_date=gte.{from_s}\
         &start_date=lte.{to_s}\
         &select=id,name,city,country,start_date,end_date,type,hot,link,description,\
                 event_mode,inclusion_flag_codes\
         &order=start_date.asc\
         &limit=40",
        db.url
    );

    // Active campaigns
    let camps_url = format!(
        "{}/rest/v1/campaigns?active=eq.true&select=name,org,link,raised,goal,currency,urgent,ends_at&order=urgent.desc,ends_at.asc.nullslast&limit=6",
        db.url
    );
    // current_title_holders view — should return one row per title, but Rust-side dedup
    // guards against the view returning multiple years (confirmed by test: current_holder_dedup_keeps_latest_year)
    let titles_url = format!(
        "{}/rest/v1/current_title_holders?select=title_name,holder_name,holder_status,display_status,year,city,country,competition_scope,competition_name&order=competition_scope.asc,title_name.asc",
        db.url
    );

    let (events_res, camps_res, titles_res) = tokio::join!(
        db.get_json::<Vec<EventRow>>(&events_url),
        db.get_json::<Vec<CampaignRow>>(&camps_url),
        db.get_json::<Vec<CurrentHolder>>(&titles_url),
    );

    let events = events_res.or_log("now:events_res");
    let cmpg = camps_res.or_log("now:camps_res");
    // Deduplicate: view may return multiple years per title_name; keep the most recent.
    let ttls: Vec<CurrentHolder> = {
        let raw = titles_res.or_log("now:titles_res");
        let mut seen: std::collections::HashMap<String, CurrentHolder> =
            std::collections::HashMap::new();
        for t in raw {
            let year = t.year.unwrap_or(0);
            let existing = seen.get(&t.title_name).and_then(|v| v.year).unwrap_or(0);
            if year >= existing {
                seen.insert(t.title_name.clone(), t);
            }
        }
        let mut v: Vec<_> = seen.into_values().collect();
        v.sort_by(|a, b| {
            a.competition_scope
                .cmp(&b.competition_scope)
                .then(a.title_name.cmp(&b.title_name))
        });
        v
    };

    // ── Event cards grouped by region ────────────────────────
    let make_event_card = |ev: &EventRow| -> String {
        let name = esc(ev.name.as_str());
        let city = esc(ev.city.as_deref().unwrap_or(""));
        let ctry = esc(ev.country.as_deref().unwrap_or(""));
        let start = esc(ev.start_date.as_deref().unwrap_or(""));
        let end = esc(ev.end_date.as_deref().unwrap_or(""));
        let link = esc(ev.link.as_deref().unwrap_or(""));
        let etype = esc(ev.event_type.as_deref().unwrap_or(""));
        let hot = ev.hot.unwrap_or(false);
        let fs = ev.inclusion_flag_codes.clone().unwrap_or_default();
        let dates = if !end.is_empty() && end != start {
            format!("{start} → {end}")
        } else {
            start.to_string()
        };
        let link_html = if !link.is_empty() && link != "#" {
            format!(
                "<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>"
            )
        } else {
            String::new()
        };
        let hot_badge = if hot {
            format!(
                "<span style=\"font-size:9px;background:{ORANGE};color:#fff;\
                      border-radius:6px;padding:1px 5px;margin-right:4px\">🔥 hot</span>"
            )
        } else {
            String::new()
        };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}{sep}{ctry}</div>\
                <div style=\"font-size:12px;color:{MID}\">{dates}</div>\
                <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                  {hot_badge}\
                  <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                  {fhtml}\
                </div>\
              </div>\
              {link_html}\
            </div>",
            sep   = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            fhtml = flags(&fs),
        ))
    };
    let region_order = [
        "North America",
        "Europe",
        "Asia Pacific",
        "Latin America",
        "Africa & Middle East",
        "Other",
    ];
    let event_cards: String = region_order.iter().filter_map(|&region| {
        let group: Vec<_> = events.iter()
            .filter(|ev| country_region(ev.country.as_deref().unwrap_or("")) == region)
            .collect();
        if group.is_empty() { return None; }
        let cards: String = group.iter().map(|ev| make_event_card(ev)).collect();
        Some(format!(
            "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.1em;color:{BROWN};padding:10px 4px 4px\">{region}</div>{cards}"
        ))
    }).collect();

    let empty_events = if events.is_empty() {
        format!(
            "<div style=\"text-align:center;padding:24px 0;color:{MID}\">\
              <div style=\"font-size:32px;margin-bottom:8px\">🐻</div>\
              <div style=\"font-size:13px;font-weight:600\">Nothing in the next 30 days</div>\
              <div style=\"font-size:12px;margin-top:4px\">\
                <a href=\"/?zone=coming-up&lang={lang}\" style=\"color:{ORANGE}\">Browse upcoming events →</a>\
              </div>\
            </div>"
        )
    } else {
        String::new()
    };

    // ── Campaign cards ────────────────────────────────────────
    let camp_cards: String = cmpg.iter().take(4).map(|c| {
        let name   = esc(c.name.as_str());
        let org    = esc(c.org.as_deref().unwrap_or(""));
        let link   = esc(c.link.as_deref().unwrap_or(""));
        let urgent = c.urgent.unwrap_or(false);
        let raised = c.raised;
        let goal   = c.goal;
        let _curr  = esc(c.currency.as_deref().unwrap_or("USD"));
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-g\">Donate</a>")
        } else { String::new() };
        let urgent_badge = if urgent {
            "<span style=\"font-size:9px;background:#C0392B;color:#fff;\
                      border-radius:6px;padding:1px 5px;margin-right:4px\">URGENT</span>".to_string()
        } else { String::new() };
        let progress = match (raised, goal) {
            (Some(r), Some(g)) if g > 0.0 => {
                let pct = (r / g * 100.0).min(100.0);
                format!(
                    "<div style=\"margin-top:6px;background:{TAN};border-radius:3px;height:4px\">\
                      <div style=\"background:{ORANGE};height:100%;width:{pct:.0}%\"></div>\
                    </div>"
                )
            },
            _ => String::new(),
        };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{urgent_badge}{name}</div>\
                <div style=\"font-size:11px;color:{MID};margin-top:2px\">{org}</div>\
                {progress}\
              </div>\
              <div style=\"flex-shrink:0\">{link_html}</div>\
            </div>"
        ))
    }).collect();

    // ── Title holder cards (from current_title_holders view) ─────
    // TBD cards: upcoming title contests without a recorded winner yet.
    // ttls has been deduped above — one entry per title_name
    let tbd_cards: Vec<(String, String)> = events
        .iter()
        .filter(|ev| ev.event_type.as_deref().unwrap_or("") == "title")
        .filter_map(|ev| {
            let name = esc(&ev.name);
            let start = ev.start_date.clone().unwrap_or_default();
            let already_held = ttls.iter().any(|t| {
                let comp_name = esc(t.competition_name.as_deref().unwrap_or(""));
                !comp_name.is_empty() && name.to_lowercase().contains(&comp_name.to_lowercase())
            });
            if already_held {
                return None;
            }
            Some((name, start))
        })
        .collect();

    let scope_order = [
        "international",
        "continental",
        "national",
        "regional",
        "local",
    ];
    let title_cards: String = scope_order.iter().filter_map(|&sc| {
        let group: Vec<&CurrentHolder> = ttls.iter().filter(|t| {
            t.competition_scope.as_deref().unwrap_or("") == sc
        }).collect();
        let is_last = sc == "local";
        if group.is_empty() && (tbd_cards.is_empty() || !is_last) { return None; }
        let scope_label = match sc {
            "international" => "International",
            "continental"   => "Continental",
            "national"      => "National",
            "regional"      => "Regional",
            _               => "Local",
        };
        let mut rows: String = group.iter().map(|t| {
            let title   = esc(t.title_name.as_str());
            let holder  = esc(t.holder_name.as_str());
            let year    = t.year.unwrap_or(0) as i64;
            let city    = esc(t.city.as_deref().unwrap_or(""));
            let ctry    = esc(t.country.as_deref().unwrap_or(""));
            let status  = t.holder_status.as_deref().unwrap_or("");
            let badge   = match status {
                "holdover" => format!(
                    "<span style=\"font-size:9px;font-weight:700;text-transform:uppercase;                                    letter-spacing:.05em;color:{ORANGE};border:1px solid {ORANGE};                                    border-radius:10px;padding:1px 6px;margin-left:6px\">holdover</span>"
                ),
                _ => String::new(),
            };
            card(&format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center\">                  <div>                    <div style=\"font-weight:600;font-size:14px\">{title}{badge}</div>                    <div style=\"font-size:12px;color:{MID};margin-top:2px\">{holder}</div>                    <div style=\"font-size:11px;color:{MID}\">{city}{sep}{ctry}</div>                  </div>                  <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">{year}</div>                </div>",
                sep   = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            ))
        }).collect();
        if is_last {
            for (name, start) in &tbd_cards {
                rows.push_str(&card(&format!(
                    "<div style=\"display:flex;justify-content:space-between;align-items:center\">                      <div>                        <div style=\"font-weight:600;font-size:14px\">{name}</div>                        <div style=\"font-size:12px;color:{MID};margin-top:2px\">Title to be decided</div>                        <div style=\"font-size:11px;color:{MID}\">{start}</div>                      </div>                      <div style=\"font-size:11px;font-weight:700;text-transform:uppercase;                                  letter-spacing:.05em;color:{ORANGE};border:1px solid {ORANGE};                                  border-radius:10px;padding:3px 8px\">TBD</div>                    </div>"
                )));
            }
        }
        if rows.is_empty() { return None; }
        Some(format!(
            "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;                          letter-spacing:.1em;color:{BROWN};padding:8px 0 4px\">              {scope_label}</div>{rows}"
        ))
    }).collect();

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Now</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:12px\">\
          What the bear world is doing in the next 30 days.</p>\
        \
        {h_events}\
        {event_cards}\
        {empty_events}\
        <a href=\"/?zone=coming-up&lang={lang}\" \
           style=\"display:block;text-align:center;font-size:13px;\
                  color:{ORANGE};padding:8px 0 16px\">Browse all upcoming events →</a>\
        \
        {h_camps}\
        {camp_cards}\
        <a href=\"/?zone=future&lang={lang}\" \
           style=\"display:block;text-align:center;font-size:13px;\
                  color:{ORANGE};padding:4px 0 16px\">All campaigns →</a>\
        \
        {h_titles}\
        {title_cards}\
        <a href=\"/?zone=titles\" \
           style=\"display:block;text-align:center;font-size:13px;\
                  color:{ORANGE};padding:4px 0 24px\">Full competition archive →</a>",
        h_events = sh("Happening in the Next 30 Days", Some(events.len())),
        h_camps = sh("Community Campaigns", Some(cmpg.len())),
        h_titles = sh("Current Title Holders", Some(ttls.len() + tbd_cards.len())),
    );

    Html(shell(
        "Now",
        "Bear events in the next 30 days.",
        "now",
        &body,
        lang,
    ))
    .into_response()
}

// ── ZONE: COMING UP ───────────────────────────────────────────
