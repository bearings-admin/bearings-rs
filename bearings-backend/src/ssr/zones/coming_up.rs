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
    let tl = |k: &str| crate::i18n::t(crate::i18n::translations(), lang, k);

    // Compute the date window from the selected "when" value. Most options are a
    // cumulative "today -> today + N months" range, but two are fixed windows that
    // skip the near term, encoded as sentinels so the <select> still submits a
    // single numeric months_ahead:
    //   612 = 6 months to a year out (excludes the next 6 months)
    //   999 = the next calendar year (Jan 1 - Dec 31)
    let today = Utc::now().date_naive();
    let (from_date, to_date, sel_val) = match months_ahead.unwrap_or(12) {
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
        "country":      country_val.clone(),
        "max_rows":     60,
    });
    // The list respects the selected window; the bar always shows a rolling
    // 12 months so the forward year is always visible. Fetch both concurrently.
    let bar_to = today.checked_add_months(Months::new(12)).unwrap_or(today);
    let rpc_body_bar = serde_json::json!({
        "input_lat":    serde_json::Value::Null,
        "input_lng":    serde_json::Value::Null,
        "radius_km":    serde_json::Value::Null,
        "season":       serde_json::Value::Null,
        "from_date":    today.format("%Y-%m-%d").to_string(),
        "to_date":      bar_to.format("%Y-%m-%d").to_string(),
        "event_type":   serde_json::Value::Null,
        "country":      country_val.clone(),
        "max_rows":     200,
    });
    let pred_country = if country.is_empty() {
        String::new()
    } else {
        format!("&country=eq.{}", urlencoding::encode(country))
    };
    let predictions_url = format!(
        "{}/rest/v1/event_predictions?select=sample_name,city,country,predicted_date,confidence{pred_country}",
        db.url
    );
    let (data_res, bar_res, pred_res) = tokio::join!(
        db.post_rpc("coming_up", &rpc_body),
        db.post_rpc("coming_up", &rpc_body_bar),
        db.get_json::<Vec<PredictionRow>>(&predictions_url),
    );
    let data: serde_json::Value = match data_res {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "coming_up rpc failed").into_response()
        }
    };
    let bar_dates: Vec<Option<String>> = bar_res
        .ok()
        .and_then(|v: serde_json::Value| v["events"].as_array().cloned())
        .map(|arr| {
            arr.iter()
                .map(|e| {
                    e.get("start_date")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();
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
        (1, "comingup.when.1"),
        (2, "comingup.when.2"),
        (3, "comingup.when.3"),
        (6, "comingup.when.6"),
        (612, "comingup.when.6to12"),
        (999, "comingup.when.year"),
    ];
    let sel12 = if sel_val == 12 { " selected" } else { "" };
    let mut months_sel = format!("<option value=\"12\"{sel12}>Next 12 months</option>");
    months_sel.push_str(
        &months_opts
            .iter()
            .map(|(v, l)| {
                let sel = if *v == sel_val { " selected" } else { "" };
                format!("<option value=\"{v}\"{sel}>{}</option>", tl(l))
            })
            .collect::<String>(),
    );

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
    let month_label = if sel_val == 12 {
        "Next 12 months".to_string()
    } else {
        months_opts
            .iter()
            .find(|(v, _)| *v == sel_val)
            .map(|(_, l)| tl(l))
            .unwrap_or_else(|| tl("comingup.when.6"))
    };

    // ── Monthly bar chart + optional month filter ───────────────
    let country_enc = if country.is_empty() {
        String::new()
    } else {
        format!("&event_country={}", urlencoding::encode(country))
    };
    let pred_rows: Vec<PredictionRow> = pred_res.unwrap_or_default();
    let predictions: Vec<(Option<String>, f64)> = pred_rows
        .iter()
        .map(|p| {
            let op = match p.confidence.as_deref() {
                Some("high") => 0.42,
                Some("medium") => 0.26,
                _ => 0.16,
            };
            (p.predicted_date.clone(), op)
        })
        .collect();
    let nav_base = format!("/?zone=coming-up&lang={lang}{country_enc}");
    let bar = timeline_bar(
        &bar_dates,
        month_filter,
        &nav_base,
        "#upcoming-results",
        today.year(),
        today.month(),
        &predictions,
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
                    "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.1em;color:{BROWN};padding:10px 4px 4px\" id=\"cu-rg-{slug}\">{region}</div>{cards}",
                    slug = region_slug(region)
            ))
        }).collect()
    } else {
        disp_events.iter().map(make_cu_card).collect()
    };

    // Tentative (forecasted) events for the list: predictions inside the selected
    // window/month, shown distinctly with an approximate week and a warning.
    let tentative_rows: Vec<&PredictionRow> = pred_rows
        .iter()
        .filter(|p| {
            let d = match p.predicted_date.as_deref() {
                Some(d) => d,
                None => return false,
            };
            if d < from_str.as_str() || d > to_str.as_str() {
                return false;
            }
            if let Some(m) = month_filter {
                let pm = d.split('-').nth(1).and_then(|s| s.parse::<u32>().ok());
                if pm != Some(m) {
                    return false;
                }
            }
            true
        })
        .collect();
    let tentative = if tentative_rows.is_empty() {
        String::new()
    } else {
        let cards: String = tentative_rows
            .iter()
            .map(|p| {
                let name = esc(p.sample_name.as_deref().unwrap_or("(likely event)"));
                let city = esc(p.city.as_deref().unwrap_or(""));
                let ctry = esc(p.country.as_deref().unwrap_or(""));
                let loc = match (city.is_empty(), ctry.is_empty()) {
                    (false, false) => format!("{city}, {ctry}"),
                    (false, true) => city.to_string(),
                    (true, false) => ctry.to_string(),
                    _ => String::new(),
                };
                let approx = approx_week_label(p.predicted_date.as_deref().unwrap_or(""));
                let conf = esc(p.confidence.as_deref().unwrap_or(""));
                format!(
                    "<div style=\"border:1px dashed {GOLD};background:#FFFDF6;border-radius:14px;\
                       padding:12px 14px;margin-bottom:8px\">\
                      <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
                        <div style=\"flex:1;min-width:0\">\
                          <div style=\"font-weight:600;font-size:14px;line-height:1.3;color:{BROWN}\">{name}</div>\
                          <div style=\"font-size:12px;color:{MID};margin-top:2px\">{loc} \u{00b7} ~ {approx}</div>\
                          <div style=\"font-size:11px;color:#9a7b1f;margin-top:5px\">\u{26a0} Specific dates not yet announced \u{2014} projected from past years ({conf} confidence)</div>\
                        </div>\
                        <span class=\"badge\" style=\"background:{GOLD};color:{DARK};white-space:nowrap\">tentative</span>\
                      </div>\
                    </div>"
                )
            })
            .collect();
        format!(
            "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;\
               letter-spacing:.1em;color:{BROWN};padding:14px 4px 4px\">\
               Likely to repeat \u{00b7} awaiting confirmation</div>{cards}"
        )
    };

    let empty_html = if disp_events.is_empty() && tentative_rows.is_empty() {
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

    let where_map = build_where_map(&disp_events, worldwide);
    let meet_title = tl("comingup.meet_title");
    let meet_sub = tl("comingup.meet_sub");
    let body = format!(
        "<div style=\"text-align:center;padding:20px 0 12px\">\
          <h1 style=\"font-size:22px;font-weight:700;color:{BROWN};line-height:1.2;\
                      margin-bottom:6px\">{meet_title}<br>\
            <span style=\"font-size:15px;font-weight:400;color:{MID}\">{meet_sub}</span>\
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
        <div id=\"upcoming-results\">\
          <div style=\"font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.1em;color:{MID};margin:2px 0 6px\">Where</div>\
          {where_map}\
          {bar}\
          {h_ev}\
          {ev_cards}\
          {tentative}\
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

/// Human "approximate week" label from an ISO date, e.g. "mid May 2027".
fn approx_week_label(d: &str) -> String {
    let mut it = d.splitn(3, '-');
    let y = it.next().unwrap_or("");
    let m = it.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let day = it
        .next()
        .and_then(|s| s.get(0..2))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    const MONTHS: [&str; 13] = [
        "", "January", "February", "March", "April", "May", "June", "July", "August",
        "September", "October", "November", "December",
    ];
    let mon = MONTHS.get(m).copied().unwrap_or("");
    let part = if day == 0 {
        ""
    } else if day <= 10 {
        "early "
    } else if day <= 20 {
        "mid "
    } else {
        "late "
    };
    format!("{part}{mon} {y}")
}

fn region_slug(r: &str) -> String {
    r.to_lowercase().replace(" & ", "-").replace(' ', "-")
}

/// Continental "Where" heat map for Coming Up, rendered on a real world map
/// (country shapes coloured by region). Geometry is a static asset; colours are
/// computed from the currently-shown events. Map source: flekschas/simple-world-map
/// (CC BY-SA 3.0, Al MacDonald / Fritz Lekschas).
fn build_where_map(events: &[EventRow], worldwide: bool) -> String {
    const INNER: &str = include_str!("world_inner.svg");
    const VIEWBOX: &str = "30.767 241.591 784.077 458.627";
    let regions: [(&str, &str, &[&str]); 5] = [
        ("North America", "210,105,30", &["us", "ca", "mx", "pr"]),
        (
            "Europe",
            "212,160,23",
            &[
                "be", "cz", "ee", "fr", "de", "is", "ie", "it", "lu", "nl", "no", "pl", "pt", "gb",
                "es", "se", "ch", "at", "dk", "fi", "gr", "hu", "ro", "rs", "hr", "sk", "si", "ba",
                "bg", "al", "lt", "lv", "ua", "by", "md", "mk",
            ],
        ),
        (
            "Asia Pacific",
            "29,158,117",
            &[
                "au", "jp", "my", "nz", "ph", "sg", "kr", "tw", "th", "id", "vn", "kh", "la", "mm",
                "np", "lk", "kp", "mn", "bd", "bt", "bn",
            ],
        ),
        (
            "Latin America",
            "194,96,122",
            &[
                "ar", "br", "cl", "co", "uy", "pe", "ve", "ec", "bo", "py", "gy", "sr", "gf", "pa",
                "cr", "ni", "hn", "gt", "sv", "bz", "cu", "do", "ht",
            ],
        ),
        (
            "Africa & Middle East",
            "154,123,181",
            &[
                "eg", "il", "ma", "za", "ae", "dz", "ly", "sd", "et", "ke", "ng", "tz", "cd", "ao",
                "mz", "na", "sa", "iq", "ir", "tr", "sy", "jo", "ye", "om", "tn", "gh", "ml", "ne",
                "td", "cm", "so", "zm", "zw", "bw", "mg", "ug", "sn", "cf", "cg", "mr", "bf",
            ],
        ),
    ];
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for e in events {
        *counts
            .entry(country_region(e.country.as_deref().unwrap_or("")))
            .or_default() += 1;
    }
    let max = counts.values().copied().max().unwrap_or(1).max(1) as f64;

    let mut css = String::from("#cuwm path{fill:#d8ccb5;stroke:none}");
    for (name, rgb, isos) in &regions {
        let c = *counts.get(name).unwrap_or(&0);
        let alpha = if c == 0 {
            0.14
        } else {
            0.25 + 0.75 * (c as f64 / max)
        };
        let sels: Vec<String> = isos
            .iter()
            .flat_map(|i| [format!("#cuwm #{i}"), format!("#cuwm #{i} path")])
            .collect();
        css.push_str(&format!(
            "{}{{fill:rgba({rgb},{alpha:.2})}}",
            sels.join(",")
        ));
    }

    let legend: String = regions
        .iter()
        .map(|(name, rgb, _)| {
            let c = *counts.get(name).unwrap_or(&0);
            let sw = format!(
                "<span style=\"display:inline-block;width:11px;height:11px;border-radius:2px;\
                 background:rgb({rgb});vertical-align:-1px;margin-right:5px\"></span>"
            );
            let label = if c == 0 {
                format!("{name} &middot; soon")
            } else {
                format!("{name} {c}")
            };
            if worldwide && c > 0 {
                format!(
                    "<a href=\"#cu-rg-{slug}\" style=\"font-size:12px;color:{MID};text-decoration:none\">{sw}{label}</a>",
                    slug = region_slug(name)
                )
            } else {
                format!("<span style=\"font-size:12px;color:{MID}\">{sw}{label}</span>")
            }
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<style>{css}</style>\
         <svg id=\"cuwm\" viewBox=\"{VIEWBOX}\" preserveAspectRatio=\"xMidYMid meet\" \
           style=\"width:100%;height:auto;background:#e7ddcb;border-radius:10px;display:block;margin-bottom:8px\" \
           role=\"img\"><title>Upcoming events by region</title>{INNER}</svg>\
         <div style=\"display:flex;gap:12px;flex-wrap:wrap;margin-bottom:14px\">{legend}</div>"
    )
}
