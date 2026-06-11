//! Zone: ical

use axum::response::{Html, IntoResponse, Response};
use crate::ui::*;
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_ical(lang: &str) -> Response {
    let body = format!(
        "<div style=\"text-align:center;padding:24px 0 16px\">\
          <div style=\"font-size:48px;margin-bottom:8px\">📅</div>\
          <h1 style=\"font-size:20px;font-weight:700;color:{BROWN};margin-bottom:6px\">\
            Subscribe to Bear Events</h1>\
          <p style=\"font-size:13px;color:{MID};max-width:360px;margin:0 auto 20px;\
             line-height:1.6\">\
            Add the full bear events calendar to Apple Calendar, \
            Google Calendar, or Outlook — it auto-updates as new events are added.</p>\
        </div>\
        \
        <div class=\"card\" style=\"margin-bottom:12px\">\
          <div style=\"font-size:11px;font-weight:600;color:{MID};text-transform:uppercase;\
            letter-spacing:.08em;margin-bottom:10px\">All upcoming events</div>\
          <div style=\"display:flex;gap:8px;align-items:center;flex-wrap:wrap\">\
            <code style=\"flex:1;font-size:11px;background:{OFF_WHITE};border:1px solid {TAN};\
              border-radius:8px;padding:8px 10px;word-break:break-all;color:{DARK}\">\
              https://bearings.app/api/events/ical.ics</code>\
            <a href=\"/api/events/ical.ics\" class=\"btn-o\" download=\"bearings-events.ics\">\
              Download</a>\
          </div>\
          <div style=\"font-size:11px;color:{MID};margin-top:8px\">\
            In Apple Calendar: File → New Calendar Subscription → paste the URL above.<br>\
            In Google Calendar: Other calendars (+) → From URL → paste and add.</div>\
        </div>\
        \
        <div class=\"card\" style=\"margin-bottom:12px\">\
          <div style=\"font-size:11px;font-weight:600;color:{MID};text-transform:uppercase;\
            letter-spacing:.08em;margin-bottom:10px\">Filter by country</div>\
          <div style=\"display:flex;flex-direction:column;gap:8px\">\
            {country_rows}\
          </div>\
        </div>\
        \
        <div class=\"card\" style=\"text-align:center\">\
          <div style=\"font-size:12px;color:{MID};margin-bottom:6px\">\
            Calendar feeds update automatically. No account needed.</div>\
          <a href=\"/?zone=coming-up&lang={lang}\" class=\"btn-t\">← Back to events</a>\
        </div>",
        country_rows = [
            ("Worldwide", ""),
            ("Canada",    "Canada"),
            ("USA",       "USA"),
            ("Germany",   "Germany"),
            ("Netherlands","Netherlands"),
            ("UK",        "UK"),
            ("Australia", "Australia"),
            ("Brazil",    "Brazil"),
            ("Spain",     "Spain"),
        ].iter().map(|(label, c)| {
            let qs = if c.is_empty() { String::new() } else { format!("?country={c}") };
            format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center;\
                  padding:4px 0;border-bottom:1px solid {OFF_WHITE}\">\
                  <span style=\"font-size:12px;color:{DARK}\">{label}</span>\
                  <a href=\"/api/events/ical.ics{qs}\" class=\"btn-t\" style=\"font-size:11px\"\
                    download>ics</a>\
                </div>",
                label = label,
                qs    = qs,
            )
        }).collect::<String>(),
    );
    Html(shell("iCal Export", "Subscribe to bear events in your calendar.", "ical", &body, lang)).into_response()
}


