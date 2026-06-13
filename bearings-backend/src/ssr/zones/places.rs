//! Zone: places

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_places(
    db: SupabaseClient,
    filter_type: Option<String>,
    filter_country: Option<String>,
    lang: &str,
) -> Response {
    let ft = esc(filter_type.as_deref().unwrap_or(""));
    let fc = esc(filter_country.as_deref().unwrap_or(""));

    // Fetch ALL places for country (for tab counts) — no type filter
    let cc = if !fc.is_empty() {
        format!("&country=eq.{fc}")
    } else {
        String::new()
    };
    let url_all = format!(
        "{}/rest/v1/places?active=eq.true\
         &select=name,place_type,country\
         {cc}\
         &limit=500",
        db.url
    );
    // Fetch filtered places for display
    let tc = if !ft.is_empty() {
        format!("&place_type=eq.{ft}")
    } else {
        String::new()
    };
    let url_filtered = format!(
        "{}/rest/v1/places?active=eq.true\
         &select=name,place_type,city,country,address,hours_open,website,\
         booking_link,bear_popular,bear_night_schedule,inclusion_flag_codes\
         {tc}{cc}\
         &order=bear_popular.desc.nullslast,country.asc,city.asc&limit=200",
        db.url
    );
    let (all_res, filtered_res) = tokio::join!(
        db.get_json::<Vec<PlaceRow>>(&url_all),
        db.get_json::<Vec<PlaceRow>>(&url_filtered),
    );
    let all_places = all_res.or_log("places:all_res");
    let places = filtered_res.or_log("places:filtered_res");

    // Count per type across all (country-filtered) places
    let mut type_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for p in &all_places {
        let pt = p.place_type.as_deref().unwrap_or("other");
        *type_counts.entry(pt).or_insert(0) += 1;
    }
    let total_all = all_places.len();

    // Unique countries from ALL places
    let mut countries: Vec<String> = all_places
        .iter()
        .filter_map(|p| p.country.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    countries.sort();

    // Tab definitions — (slug, label, icon)
    let type_tabs: &[(&str, &str, &str)] = &[
        ("", "All", "🗺"),
        ("bar", "Bar", "🍺"),
        ("leather-bar", "Leather", "🧤"),
        ("sauna-bathhouse", "Sauna", "♨"),
        ("campground", "Camp", "🏕"),
        ("party-venue", "Party", "🎉"),
        ("resort", "Resort", "🛖"),
        ("hotel", "Hotel", "🏨"),
        ("cruise-ship", "Cruise", "🚢"),
    ];

    let tabs_html: String = type_tabs
        .iter()
        .filter_map(|(slug, label, icon)| {
            let count = if slug.is_empty() {
                total_all
            } else {
                *type_counts.get(*slug).unwrap_or(&0)
            };
            if count == 0 && !slug.is_empty() {
                return None;
            } // hide empty tabs
            let on = *slug == ft;
            let bg = if on { ORANGE } else { OFF_WHITE };
            let fg = if on { "#fff" } else { BROWN };
            let border = if on { ORANGE } else { TAN };
            let fw = if on { "700" } else { "400" };
            let country_qs = if !fc.is_empty() {
                format!("&place_country={fc}")
            } else {
                String::new()
            };
            Some(format!(
                "<a href=\"/?zone=places&place_type={slug}&lang={lang}{cqs}\" \
               style=\"display:inline-flex;flex-direction:column;align-items:center;\
                       gap:1px;text-decoration:none;padding:7px 10px;\
                       border-radius:12px;border:1px solid {border};\
                       background:{bg};color:{fg};font-weight:{fw};\
                       white-space:nowrap;min-width:52px\">\
              <span style=\"font-size:14px;line-height:1\">{icon}</span>\
              <span style=\"font-size:10px\">{label}</span>\
              <span style=\"font-size:9px;opacity:.8\">{count}</span>\
            </a>",
                cqs = country_qs,
            ))
        })
        .collect();

    // Region grouping definitions (same optgroups as coming-up country selector)
    let place_regions: &[(&str, &[&str])] = &[
        ("North America", &["Canada", "USA", "Mexico", "Puerto Rico"]),
        (
            "Europe",
            &[
                "Belgium",
                "Czech Republic",
                "Estonia",
                "France",
                "Germany",
                "Iceland",
                "Ireland",
                "Italy",
                "Luxembourg",
                "Netherlands",
                "Norway",
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
            &[
                "Australia",
                "Japan",
                "New Zealand",
                "South Korea",
                "Taiwan",
                "Thailand",
            ],
        ),
        (
            "Latin America",
            &["Argentina", "Brazil", "Chile", "Colombia"],
        ),
        (
            "Africa & Middle East",
            &["Egypt", "Morocco", "South Africa"],
        ),
    ];

    // Country dropdown
    let present: std::collections::HashSet<&str> = countries.iter().map(|s| s.as_str()).collect();
    let sel_attr = |v: &str| if v == fc { " selected" } else { "" };
    let mut ctry_sel = format!(
        "<option value=\"\"{}>\u{1f30d} All regions</option>",
        sel_attr("")
    );
    let mut placed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (region, region_countries) in place_regions {
        let opts: String = region_countries
            .iter()
            .copied()
            .filter(|c| present.contains(c))
            .map(|c| {
                placed.insert(c);
                format!("<option value=\"{c}\"{}>{c}</option>", sel_attr(c))
            })
            .collect();
        if !opts.is_empty() {
            ctry_sel.push_str(&format!("<optgroup label=\"{region}\">{opts}</optgroup>"));
        }
    }
    let other: String = countries
        .iter()
        .map(|s| s.as_str())
        .filter(|c| !placed.contains(c))
        .map(|c| format!("<option value=\"{c}\"{}>{c}</option>", sel_attr(c)))
        .collect();
    if !other.is_empty() {
        ctry_sel.push_str(&format!("<optgroup label=\"Other\">{other}</optgroup>"));
    }
    let sel_style = format!(
        "font-size:12px;padding:6px 12px;border-radius:20px;\
         border:1px solid {TAN};background:{OFF_WHITE};color:{DARK};\
         font-family:inherit"
    );

    // Place type label for the active filter
    let type_label = type_tabs
        .iter()
        .find(|(s, _, _)| *s == ft)
        .map(|(_, l, _)| *l)
        .unwrap_or("All");
    let region_label = if fc.is_empty() {
        "worldwide".to_string()
    } else {
        fc.to_string()
    };

    // Group places by type for display when "All" is selected + no country, else by type or flat
    let items_html: String = if ft.is_empty() && fc.is_empty() {
        // All types, all countries — group by region, then by type within region
        let mut out = String::new();
        for (region, countries) in place_regions {
            let region_places: Vec<&PlaceRow> = places
                .iter()
                .filter(|p| {
                    let c = p.country.as_deref().unwrap_or("");
                    countries.contains(&c)
                })
                .collect();
            if region_places.is_empty() {
                continue;
            }

            out.push_str(&format!(
                "<div style=\"font-size:12px;font-weight:700;color:{BROWN};margin:18px 0 8px;\
                  border-left:3px solid {ORANGE};padding-left:8px\">{region} \
                  <span style=\"font-weight:400;color:{MID};font-size:11px\">({n})</span></div>",
                n = region_places.len(),
            ));

            // Sub-group by type within region
            for (slug, label, icon) in type_tabs.iter() {
                if slug.is_empty() {
                    continue;
                }
                let group: Vec<&PlaceRow> = region_places
                    .iter()
                    .filter(|p| p.place_type.as_deref().unwrap_or("") == *slug)
                    .cloned()
                    .collect();
                if group.is_empty() {
                    continue;
                }
                out.push_str(&format!(
                    "<div style=\"font-size:10px;font-weight:600;text-transform:uppercase;\
                      letter-spacing:.08em;color:{MID};margin:10px 0 4px\">{icon} {label} ({n})</div>",
                    n = group.len(),
                ));
                for p in &group {
                    out.push_str(&place_card(p));
                }
            }

            // Any uncategorised
            let other: Vec<&PlaceRow> = region_places
                .iter()
                .filter(|p| {
                    let pt = p.place_type.as_deref().unwrap_or("");
                    !type_tabs.iter().any(|(s, _, _)| !s.is_empty() && *s == pt)
                })
                .cloned()
                .collect();
            for p in &other {
                out.push_str(&place_card(p));
            }
        }
        // Any countries not in the region list
        let other_places: Vec<&PlaceRow> = places
            .iter()
            .filter(|p| {
                let c = p.country.as_deref().unwrap_or("");
                !place_regions.iter().any(|(_, cs)| cs.contains(&c))
            })
            .collect();
        if !other_places.is_empty() {
            out.push_str(&format!(
                "<div style=\"font-size:12px;font-weight:700;color:{BROWN};margin:18px 0 8px;\
                  border-left:3px solid {TAN};padding-left:8px\">Other</div>"
            ));
            for p in &other_places {
                out.push_str(&place_card(p));
            }
        }
        out
    } else if ft.is_empty() && !fc.is_empty() {
        // Specific country, all types — group by type
        type_tabs.iter().filter_map(|(slug, label, icon)| {
            if slug.is_empty() { return None; }
            let group: Vec<&PlaceRow> = places.iter()
                .filter(|p| p.place_type.as_deref().unwrap_or("") == *slug)
                .collect();
            if group.is_empty() { return None; }
            let cards: String = group.iter().map(|p| place_card(p)).collect();
            Some(format!(
                "<div style=\"font-size:10px;font-weight:700;text-transform:uppercase;\
                  letter-spacing:.1em;color:{MID};margin:16px 0 6px\">{icon} {label} ({n})</div>{cards}",
                n = group.len(),
            ))
        }).collect()
    } else {
        // Specific type filter — flat list
        places.iter().map(place_card).collect()
    };

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:8px\">\
          Bear Venues <span style=\"font-size:13px;font-weight:400;color:{MID}\">{type_label} · {region_label}</span></h1>\
        \
        <div style=\"overflow-x:auto;-webkit-overflow-scrolling:touch;margin-bottom:10px\">\
          <div style=\"display:inline-flex;gap:6px;padding-bottom:4px;min-width:min-content\">\
            {tabs_html}\
          </div>\
        </div>\
        \
        <form method=\"get\" action=\"/\" style=\"margin-bottom:12px;display:flex;align-items:center;gap:8px\">\
          <input type=\"hidden\" name=\"zone\" value=\"places\">\
          <input type=\"hidden\" name=\"lang\" value=\"{lang}\">\
          {type_hidden}\
          <select name=\"place_country\" onchange=\"this.form.submit()\" style=\"{sel_style}\">\
            {ctry_sel}</select>\
          <span style=\"font-size:11px;color:{MID};white-space:nowrap\">{n} venues</span>\
        </form>\
        \
        {items_html}",
        type_hidden = if !ft.is_empty() {
            format!("<input type=\"hidden\" name=\"place_type\" value=\"{ft}\">")
        } else { String::new() },
        n = places.len(),
    );
    Html(shell(
        "Places",
        "Bear bars, saunas, campgrounds worldwide.",
        "places",
        &body,
        lang,
    ))
    .into_response()
}

pub(crate) fn place_card(p: &PlaceRow) -> String {
    let name = esc(p.name.as_str());
    let ptype = esc(p.place_type.as_deref().unwrap_or(""));
    let city = esc(p.city.as_deref().unwrap_or(""));
    let ctry = esc(p.country.as_deref().unwrap_or(""));
    let addr = esc(p.address.as_deref().unwrap_or(""));
    let hours = esc(p.hours_open.as_deref().unwrap_or(""));
    let site = esc(p.website.as_deref().unwrap_or(""));
    let book = esc(p.booking_link.as_deref().unwrap_or(""));
    let bn = esc(p.bear_night_schedule.as_deref().unwrap_or(""));
    let pop = p.bear_popular.unwrap_or(false);
    let fs: Vec<String> = p.inclusion_flag_codes.clone().unwrap_or_default();
    let site_html = if !site.is_empty() && site != "#" {
        format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>")
    } else {
        String::new()
    };
    let book_html = if !book.is_empty() && book != "#" {
        format!("<a href=\"{book}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Book</a>")
    } else {
        String::new()
    };
    card(&format!(
        "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
          <div style=\"flex:1;min-width:0\">\
            <div style=\"font-weight:600;font-size:14px\">{name}{pop_icon}\
              <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ptype}</span>\
            </div>\
            <div style=\"font-size:12px;color:{MID}\">{city}{sep}{ctry}</div>\
            {addr_h}{hours_h}{bn_h}\
            <div style=\"margin-top:4px\">{fhtml}</div>\
          </div>\
          <div style=\"display:flex;flex-direction:column;gap:6px\">{site_html}{book_html}</div>\
        </div>",
        sep      = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
        pop_icon = if pop { " 🐻" } else { "" },
        addr_h   = if !addr.is_empty() {
            format!("<div style=\"font-size:11px;color:{MID}\">{addr}</div>")
        } else { String::new() },
        hours_h  = if !hours.is_empty() {
            format!("<div style=\"font-size:11px;color:{GOLD}\">🕐 {}</div>",
                hours.chars().take(60).collect::<String>())
        } else { String::new() },
        bn_h     = if !bn.is_empty() {
            format!("<div style=\"font-size:11px;color:{ORANGE}\">🐻 {}</div>",
                bn.chars().take(80).collect::<String>())
        } else { String::new() },
        fhtml    = flags(&fs),
    ))
}
