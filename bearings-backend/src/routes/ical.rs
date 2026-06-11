//! iCal export — GET /api/events/ical.ics
//!
//! A bear filters events and exports an iCalendar feed (RFC 5545) to subscribe
//! from Apple Calendar / Google Calendar / Outlook. Event data is fetched through
//! the shared `EventRepository`; this module only formats the feed.
//!
//!   ?country=Germany   ?month=September   ?type=bear-run

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use bearings_shared::models::Event;
use chrono::NaiveDate;
use serde::Deserialize;
use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::event_repo::{EventFilter, EventRepository, SupabaseEventRepository};

#[derive(Deserialize)]
pub struct ICalQuery {
    pub country: Option<String>,
    pub month: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

/// GET /api/events/ical.ics — iCalendar feed of upcoming bear events.
pub async fn export(
    State(db): State<SupabaseClient>,
    Query(params): Query<ICalQuery>,
) -> Result<Response, AppError> {
    let repo = SupabaseEventRepository::new(db);
    let events = repo.find(EventFilter {
        country:       params.country,
        month:         params.month,
        event_type:    params.event_type,
        upcoming_only: true,
        limit:         200,
    }).await?;

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

/// Build a valid iCalendar string (RFC 5545) from a list of events.
fn build_ical(events: &[Event]) -> String {
    let mut cal = String::new();
    cal.push_str("BEGIN:VCALENDAR\r\n");
    cal.push_str("VERSION:2.0\r\n");
    cal.push_str("PRODID:-//Bearings//Global Bear Community//EN\r\n");
    cal.push_str("CALSCALE:GREGORIAN\r\n");
    cal.push_str("METHOD:PUBLISH\r\n");
    cal.push_str("X-WR-CALNAME:Bearings — Bear Events\r\n");
    cal.push_str("X-WR-CALDESC:Global gay bear community events\r\n");
    cal.push_str("X-WR-TIMEZONE:UTC\r\n");

    for event in events {
        cal.push_str("BEGIN:VEVENT\r\n");
        cal.push_str(&format!("UID:bearings-event-{}@bearings.community\r\n", event.id));
        if let Some(start) = event.start_date {
            cal.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", format_ical_date(start)));
        }
        if let Some(end) = event.end_date {
            // iCal DTEND for all-day events is exclusive — add one day.
            let end_exclusive = end.checked_add_days(chrono::Days::new(1)).unwrap_or(end);
            cal.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", format_ical_date(end_exclusive)));
        }
        cal.push_str(&format!("SUMMARY:{}\r\n", ical_escape(&event.name)));
        if let (Some(city), Some(country)) = (&event.city, &event.country) {
            cal.push_str(&format!("LOCATION:{}\r\n", ical_escape(&format!("{city}, {country}"))));
        }
        if let Some(desc) = &event.description {
            let truncated: String = desc.chars().take(500).collect();
            cal.push_str(&format!("DESCRIPTION:{}\r\n", ical_escape(&truncated)));
        }
        if let Some(link) = &event.link {
            cal.push_str(&format!("URL:{}\r\n", link));
        }
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

/// RFC 5545 §3.3.11 — escape commas, semicolons, backslashes, newlines.
fn ical_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace(',', "\\,")
     .replace(';', "\\;")
     .replace('\n', "\\n")
}
