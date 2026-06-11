//! Zone: clubs

use axum::response::{Html, IntoResponse, Response};
use crate::db::LogErr;
use crate::{db::SupabaseClient, i18n, ui::*};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;
use super::super::query::*;

pub(crate) async fn zone_clubs(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/clubs?active=eq.true\
         &select=name,city,country,club_type,description,website,founded_year\
         &order=country.asc,name.asc&limit=100",
        db.url
    );
    let clubs: Vec<ClubRow> = db.get_json::<Vec<ClubRow>>(&url).await.or_log("clubs");
    let items: String = clubs.iter().map(|c| {
        let name  = esc(c.name.as_str());
        let city  = esc(c.city.as_deref().unwrap_or(""));
        let ctry  = esc(c.country.as_deref().unwrap_or(""));
        let yr    = c.founded_year.map(|y| format!(" (est. {y})")).unwrap_or_default();
        let desc  = esc(c.description.as_deref().unwrap_or(""));
        let site  = esc(c.website.as_deref().unwrap_or(""));
        let site_html = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}{yr}</div>\
                <div style=\"font-size:12px;color:{MID}\">{city}, {ctry}</div>\
                {desc_h}\
              </div>\
              {site_html}\
            </div>",
            desc_h = if !desc.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:4px\">{}</div>",
                    desc.chars().take(100).collect::<String>())
            } else { String::new() },
        ))
    }).collect();
    let page_clubs_title = i18n::t(i18n::translations(), lang, "page.clubs.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">{page_clubs_title}</h1>{items}"
    );
    Html(shell("Clubs", "Bear clubs worldwide.", "archive", &body, lang)).into_response()
}


