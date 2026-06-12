//! Zone: events

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_events(db: SupabaseClient, month: Option<u32>, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/events?active=eq.true&status=neq.past\
         &select=name,city,country,start_date,end_date,type,hot,link,inclusion_flag_codes\
         &order=start_date.asc&limit=100",
        db.url
    );
    let events: Vec<EventRow> = db.get_json::<Vec<EventRow>>(&url).await.or_log("events");
    let items: String = events.iter().filter(|e| {
        if let Some(mn) = month {
            e.start_date.as_deref().and_then(extract_month) == Some(mn)
        } else { true }
    }).map(|ev| {
        let name  = esc(ev.name.as_str());
        let city  = esc(ev.city.as_deref().unwrap_or(""));
        let ctry  = esc(ev.country.as_deref().unwrap_or(""));
        let start = esc(ev.start_date.as_deref().unwrap_or(""));
        let link  = esc(ev.link.as_deref().unwrap_or(""));
        let hot   = ev.hot.unwrap_or(false);
        let fs    = ev.inclusion_flag_codes.clone().unwrap_or_default();
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}{h}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}, {ctry} · {start}</div>\
                <div style=\"margin-top:5px\">{fhtml}</div>\
              </div>\
              {link_html}\
            </div>",
            h     = if hot { " 🔥" } else { "" },
            fhtml = flags(&fs),
        ))
    }).collect();
    let body = format!(
        "<div style=\"display:flex;justify-content:space-between;align-items:center;margin-bottom:16px\">\
          <h1 style=\"font-size:18px;font-weight:700;color:{BROWN}\">Bear Events</h1>\
          <a href=\"/api/events/ical.ics\" class=\"btn-g\">📅 Subscribe</a>\
        </div>{items}"
    );
    Html(shell(
        "Events",
        "Bear events worldwide.",
        "now",
        &body,
        lang,
    ))
    .into_response()
}
