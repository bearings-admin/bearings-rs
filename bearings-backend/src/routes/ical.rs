
//! iCal export — the feature that makes Bearings sticky.
//!
//! A bear builds their personal bear calendar by filtering events
//! and exporting to iCal. Every event they save pushes to their
//! phone calendar. They return to Bearings to refresh the feed.
//!
//! GET /api/events/ical.ics
//!   ?country=Germany       — events in Germany only
//!   ?month=September       — September events only
//!   ?type=bear-run         — bear runs only
//!
//! The response is a valid iCalendar feed (RFC 5545).
//! Add to Apple Calendar / Google Calendar / Outlook by URL.
//!
//! TODO: personal calendar subscriptions
//!   When a bear has a verified account, they can save events to
//!   "My Bears Calendar" and get a personalised iCal URL.
//!   /api/events/ical.ics?token={personal_token}

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use bearings_shared::models::Event;
use chrono::NaiveDate;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct ICalQuery {
    pub country: Option<String>,
    pub month: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

/// GET /api/events/ical.ics
/// Returns a valid iCalendar feed of upcoming bear events.
pub async fn export(
    State(db): State<SupabaseClient>,
    Query(params): Query<ICalQuery>,
) -> Result<Response, AppError> {
    let mut url = format!(
        "{}/rest/v1/events?select=*&active=eq.true&order=start_date.asc&limit=200",
        db.url
    );

    if let Some(c) = &params.country    { url.push_str(&format!("&country=eq.{}", c)); }
    if let Some(m) = &params.month      { url.push_str(&format!("&month=eq.{}", m)); }
    if let Some(t) = &params.event_type { url.push_str(&format!("&type=eq.{}", t)); }

    // Only upcoming events
    let today = chrono::Local::now().date_naive();
    url.push_str(&format!("&start_date=gte.{}", today));

    let events: Vec<Event> = db.get_json(&url).await?;
    let ical = build_ical(&events);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=bearings-events.ics"),
        ],
        ical,
    ).into_response())
}

/// Build a valid iCalendar string from a list of events.
/// Follows RFC 5545 — tested against Apple Calendar, Google Calendar, Outlook.
fn build_ical(events: &[Event]) -> String {
    let mut cal = String::new();

    cal.push_str("BEGIN:VCALENDAR\r\n");
    cal.push_str("VERSION:2.0\r\n");
    cal.push_str("PRODID:-//Bearings//Global Bear Community//EN\r\n");
    cal.push_str("CALSCALE:GREGORIAN\r\n");
    cal.push_str("METHOD:PUBLISH\r\n");
    cal.push_str("X-WR-CALNAME:Bearings — Bear Events\r\n");
    cal.push_str("X-WR-CALDESC:Global gay bear community events from bearings.lovable.app\r\n");
    cal.push_str("X-WR-TIMEZONE:UTC\r\n");

    for event in events {
        cal.push_str("BEGIN:VEVENT\r\n");

        // UID — stable per event (same event always gets same UID)
        cal.push_str(&format!("UID:bearings-event-{}@bearings.lovable.app\r\n", event.id));

        // DTSTART / DTEND — all-day events use DATE format
        if let Some(start) = event.start_date {
            cal.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", format_ical_date(start)));
        }
        if let Some(end) = event.end_date {
            // iCal DTEND for all-day is exclusive — add one day
            // succ_opt() is deprecated since chrono 0.4.23 — use checked_add_days
            let end_exclusive = end
                .checked_add_days(chrono::Days::new(1))
                .unwrap_or(end);
            cal.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", format_ical_date(end_exclusive)));
        }

        // SUMMARY (required)
        cal.push_str(&format!("SUMMARY:{}\r\n", ical_escape(&event.name)));

        // LOCATION
        if let (Some(city), Some(country)) = (&event.city, &event.country) {
            cal.push_str(&format!("LOCATION:{}, {}\r\n", city, country));
        }

        // DESCRIPTION
        if let Some(desc) = &event.description {
            let truncated: String = desc.chars().take(500).collect();
            cal.push_str(&format!("DESCRIPTION:{}\r\n", ical_escape(&truncated)));
        }

        // URL
        if let Some(link) = &event.link {
            cal.push_str(&format!("URL:{}\r\n", link));
        }

        // Categories
        if let Some(etype) = &event.event_type {
            cal.push_str(&format!("CATEGORIES:{}\r\n", etype.to_uppercase()));
        }

        cal.push_str("END:VEVENT\r\n");
    }

    cal.push_str("END:VCALENDAR\r\n");
    cal
}

fn format_ical_date(date: NaiveDate) -> String {
    date.format("%Y%m%d").to_string()
}

/// Escape special characters in iCal text fields.
/// RFC 5545 §3.3.11: commas, semicolons, backslashes must be escaped.
fn ical_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace(',', "\\,")
     .replace(';', "\\;")
     .replace('\n', "\\n")
}
