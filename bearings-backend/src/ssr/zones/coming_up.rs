//! Zone: coming_up

use super::super::query::*;
use crate::{db::SupabaseClient, ui::*};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Datelike, Months, NaiveDate, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_coming_up(
    db: SupabaseClient,
    months_ahead: Option<u32>,
    event_country: Option<String>,
    month_filter: Option<u32>,
    lang: &str,
) -> Response {
    let country = event_country.as_deref().unwrap_or("");

    // Compute the date window from the selected "when" value. Most options are a
    // cumulative "today -> today + N months" range, but two are fixed windows that
    // skip the near term, encoded as sentinels so the <select> still submits a
    // single numeric months_ahead:
    //   612 = 6 months to a year out (excludes the next 6 months)
    //   999 = the next calendar year (Jan 1 - Dec 31)
    let today = Utc::now().date_naive();
    let (from_date, to_date, sel_val) = match months_ahead.unwrap_or(6) {
        612 => (
            today.checked_add_months(Months::new(6)).unwrap_or(today),
            today.checked_add_months(Months::new(12)).unwrap_or(today),
            612u32,
        ),
        999 => {
            let ny = today.year() + 1;
            (
                NaiveDate::from_ymd_opt(ny, 1, 1).unwrap_or(today),
                NaiveDate::from_ymd_opt(ny, 12, 31).unwrap_or(today),
                999u32,
            )
        }
        n => {
            let m = n.clamp(1, 24);
            (
                today,
                today.checked_add_months(Months::new(m)).unwrap_or(today),
                m,
            )
        }
    };
    let from_str = from_date.format("%Y-%m-%d").to_string();
    let to_str = to_date.format("%Y-%m-%d").to_string();

    let country_val = if country.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(country.to_string())
    };

    let rpc_body = serde_json::json!({
        "input_lat":    serde_json::Value::Null,
        "input_lng":    serde_json::Value::Null,
        "radius_km":    serde_json::Value::Null,
        "season":       serde_json::Value::Null,
        "from_date":    from_str,
        "to_date":      to_str,
        "event_type":   serde_json::Value::Null,
        "country":      country_val,
        "max_rows":     60,
    });
    let data: serde_json::Value = match db.post_rpc("coming_up", &rpc_body).await {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "coming_up rpc failed").into_response()
        }
    };
    let events: Vec<EventRow> = data["events"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    let venues: Vec<PlaceRow> = data["venues"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    // ── Selectors ─────────────────────────────────────────────
    let sel_style = "width:100%;padding:10px 12px;border-radius:12px;\
                     border:1px solid {TAN};background:#fff;\
                     font-size:13px;color:{DARK};font-family:inherit";

    let months_opts: &[(u32, &str)] = &[
        (1, "Next month"),
        (2, "Next 2 months"),
        (3, "Next 3 months"),
        (6, "Next 6 months"),
        (612, "6 months to a year"),
        (999, "Next year"),
    ];
    let months_sel: String = months_opts
        .iter()
        .map(|(v, l)| {
            let sel = if *v == sel_val { " selected" } else { "" };
            format!("<option value=\"{v}\"{sel}>{l}</option>")
        })
        .collect();

    // Country groups
    let regions: &[(&str, &[&str])] = &[
        ("North America", &["Canada", "USA", "Mexico"]),
        (
            "Europe",
            &[
                "Belgium",
                "Czech Republic",
                "France",
                "Germany",
                "Iceland",
                "Ireland",
                "Italy",
                "Luxembourg",
                "Netherlands",
                "Poland",
                "Portugal",
                "Scotland",
                "Spain",
                "Sweden",
                "Switzerland",
                "UK",
            ],
        ),
        (
            "Asia Pacific",
            &["Australia", "Japan", "New Zealand", "Thailand"],
        ),
        (
            "Latin America",
            &["Brazil", "Argentina", "Chile", "Colombia", "Mexico"],
        ),
        (
            "Africa & Middle East",
            &["South Africa", "Egypt", "Morocco"],
        ),
    ];
    let world_sel = if country.is_empty() { " selected" } else { "" };
    let mut country_opts = format!("<option value=\"\"{world_sel}>🌍 Worldwide</option>");
    for (region, countries) in regions {
        country_opts.push_str(&format!("<optgroup label=\"{region}\">"));
        for c in *countries {
            let sel = if *c == country { " selected" } else { "" };
            country_opts.push_str(&format!("<option value=\"{c}\"{sel}>{c}</option>"));
        }
        country_opts.push_str("</optgroup>");
    }

    let where_label = if country.is_empty() {
        "Worldwide".to_string()
    } else {
        esc(country)
    };
    let month_label = months_opts
        .iter()
        .find(|(v, _)| *v == sel_val)
        .map(|(_, l)| *l)
        .unwrap_or("6 months");

    // ── Monthly bar chart + optional month filter ───────────────
    let country_enc = if country.is_empty() {
        String::new()
    } else {
        format!("&event_country={}", urlencoding::encode(country))
    };
    let bar_base = format!("/?zone=coming-up&months_ahead={sel_val}&lang={lang}{country_enc}");
    let bar = timeline_bar(
        &events
            .iter()
            .map(|e| e.start_date.clone())
            .collect::<Vec<_>>(),
        month_filter,
        &bar_base,
        "#upcoming-results",
    );

    // Filter displayed events by selected month
    let disp_events: Vec<EventRow> = if let Some(m) = month_filter {
        events
            .iter()
            .filter(|ev| {
                ev.start_date
                    .as_deref()
                    .and_then(|d| d.split('-').nth(1))
                    .and_then(|s| s.parse::<u32>().ok())
                    .map(|em| em == m)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    } else {
        events.clone()
    };

    // ── Event cards grouped by region ────────────────────────
    let make_cu_card = |ev: &EventRow| -> String {
        let name = esc(ev.name.as_str());
        let city = esc(ev.city.as_deref().unwrap_or(""));
        let ctry = esc(ev.country.as_deref().unwrap_or(""));
        let start = esc(ev.start_date.as_deref().unwrap_or(""));
        let end = esc(ev.end_date.as_deref().unwrap_or(""));
        let link = esc(ev.link.as_deref().unwrap_or(""));
        let etype = esc(ev.event_type.as_deref().unwrap_or(""));
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
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;\
                         align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">\
                  {city}{sep}{ctry} · {dates}</div>\
                <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                  <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                  {fhtml}\
                </div>\
              </div>\
              {link_html}\
            </div>",
            sep = if !city.is_empty() && !ctry.is_empty() {
                ", "
            } else {
                ""
            },
            fhtml = flags(&fs),
        ))
    };
    let worldwide = event_country
        .as_deref()
        .map(|c| c.is_empty())
        .unwrap_or(true);
    let ev_cards: String = if worldwide {
        let region_order = [
            "North America",
            "Europe",
            "Asia Pacific",
            "Latin America",
            "Africa & Middle East",
            "Other",
        ];
        region_order.iter().filter_map(|&region| {
            let group: Vec<_> = disp_events.iter()
                .filter(|ev| country_region(ev.country.as_deref().unwrap_or("")) == region)
                .collect();
            if group.is_empty() { return None; }
            let cards: String = group.iter().map(|ev| make_cu_card(ev)).collect();
            Some(format!(
                    "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.1em;color:{BROWN};padding:10px 4px 4px\">{region}</div>{cards}"
            ))
        }).collect()
    } else {
        disp_events.iter().map(make_cu_card).collect()
    };

    let empty_html = if disp_events.is_empty() {
        format!(
            "<div style=\"text-align:center;padding:32px 0;color:{MID}\">\
              <div style=\"font-size:32px;margin-bottom:8px\">🐻</div>\
              <div style=\"font-size:14px;font-weight:600\">No events found</div>\
              <div style=\"font-size:12px;margin-top:4px\">\
                Try a longer time window or a different region.</div>\
            </div>"
        )
    } else {
        String::new()
    };

    // iCal subscribe block
    let ical_block = format!(
        "<div style=\"border-radius:14px;border:1px solid {GOLD};\
             background:#FFFEF5;padding:14px 16px;margin-bottom:10px;\
             display:flex;justify-content:space-between;align-items:center;gap:12px\">\
          <div>\
            <div style=\"font-weight:600;font-size:12px;color:{BROWN};margin-bottom:2px\">\
              📅 Subscribe to bear events</div>\
            <div style=\"font-size:11px;color:{MID}\">Auto-updates in any calendar app</div>\
          </div>\
          <a href=\"/api/events/ical.ics\" class=\"btn-g\">iCal</a>\
        </div>"
    );

    // Venues section (compact)
    let vn_cards: String = venues
        .iter()
        .take(3)
        .map(|v| {
            let name = esc(v.name.as_str());
            let ptype = esc(v.place_type.as_deref().unwrap_or(""));
            let city = esc(v.city.as_deref().unwrap_or(""));
            let ctry = esc(v.country.as_deref().unwrap_or(""));
            let site = esc(v.website.as_deref().unwrap_or(""));
            let site_btn = if !site.is_empty() && site != "#" {
                format!("<a href=\"{site}\" target=\"_blank\" class=\"btn-t\">Visit</a>")
            } else {
                String::new()
            };
            card(&format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center\">\
              <div>\
                <div style=\"font-weight:600;font-size:13px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ptype}</span></div>\
                <div style=\"font-size:12px;color:{MID}\">{city}, {ctry}</div>\
              </div>\
              {site_btn}\
            </div>"
            ))
        })
        .collect();

    let body = format!(
        "<div style=\"text-align:center;padding:20px 0 12px\">\
          <h1 style=\"font-size:22px;font-weight:700;color:{BROWN};line-height:1.2;\
                      margin-bottom:6px\">When &amp; Where<br>\
            <span style=\"font-size:15px;font-weight:400;color:{MID}\">do you want to meet?</span>\
          </h1>\
        </div>\
        \
        <form id=\"upcoming-filters\"\
              hx-get=\"/\" action=\"/\" method=\"get\"\
              hx-target=\"#upcoming-results\"\
              hx-select=\"#upcoming-results\"\
              hx-swap=\"outerHTML\"\
              hx-trigger=\"change\"\
              hx-indicator=\"#cu-spin\"\
              style=\"margin-bottom:12px\">\
          <input type=\"hidden\" name=\"lang\" value=\"{lang}\">\
          <input type=\"hidden\" name=\"zone\" value=\"coming-up\">\
          <div style=\"display:grid;grid-template-columns:1fr 1fr;gap:8px\">\
            <select name=\"months_ahead\" style=\"{sel_style}\">{months_sel}</select>\
            <select name=\"event_country\" style=\"{sel_style}\">{country_opts}</select>\
          </div>\
        </form>\
        \
        <div id=\"cu-spin\" class=\"htmx-indicator\">Finding events…</div>\
        \
        {bar}\
        \
        <div id=\"upcoming-results\">\
          {h_ev}\
          {ev_cards}\
          {empty_html}\
          {ical_block}\
          {h_vn}\
          {vn_cards}\
        </div>",
        h_ev = sh(
            &format!("{month_label} · {where_label}"),
            Some(disp_events.len())
        ),
        h_vn = if venues.is_empty() {
            String::new()
        } else {
            sh(&format!("Venues in {where_label}"), Some(venues.len()))
        },
    );
    Html(shell(
        "Upcoming Events",
        "Find bear events near you.",
        "coming-up",
        &body,
        lang,
    ))
    .into_response()
}

// ── ZONE: BEAR ARCHIVES (decade tabs) ────────────────────────
