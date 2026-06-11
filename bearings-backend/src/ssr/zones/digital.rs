//! Zone: digital

use axum::response::{Html, IntoResponse, Response};
use crate::db::LogErr;
use crate::{db::SupabaseClient, i18n, ui::*};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;
use super::super::query::*;

pub(crate) async fn zone_digital(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/digital_spaces?active=eq.true\
         &select=name,space_type,description,url,member_count,\
         instagram,tiktok_handle,bluesky_handle,youtube_handle\
         &order=space_type.asc,name.asc&limit=100",
        db.url
    );
    let spaces: Vec<DigitalSpaceRow> = db.get_json::<Vec<DigitalSpaceRow>>(&url).await.or_log("digital");
    let items: String = spaces.iter().map(|s| {
        let name    = esc(s.name.as_str());
        let stype   = esc(s.space_type.as_deref().unwrap_or(""));
        let desc    = esc(s.description.as_deref().unwrap_or(""));
        let url_s   = esc(s.url.as_deref().unwrap_or(""));
        let members = s.member_count;
        let ig      = esc(s.instagram.as_deref().unwrap_or(""));
        let tt      = esc(s.tiktok_handle.as_deref().unwrap_or(""));
        let bs      = esc(s.bluesky_handle.as_deref().unwrap_or(""));
        let yt      = esc(s.youtube_handle.as_deref().unwrap_or(""));
        let mut sc  = Vec::new();
        if !ig.is_empty() { sc.push(format!("<a href=\"https://instagram.com/{ig}\" target=\"_blank\" class=\"badge\" style=\"background:#E1306C;color:#fff\">IG</a>")); }
        if !tt.is_empty() { sc.push(format!("<a href=\"https://tiktok.com/@ {tt}\" target=\"_blank\" class=\"badge\" style=\"background:#000;color:#fff\">TT</a>")); }
        if !bs.is_empty() { sc.push(format!("<a href=\"https://bsky.app/profile/{bs}\" target=\"_blank\" class=\"badge\" style=\"background:#0085ff;color:#fff\">BS</a>")); }
        if !yt.is_empty() { sc.push(format!("<a href=\"https://youtube.com/{yt}\" target=\"_blank\" class=\"badge\" style=\"background:#FF0000;color:#fff\">YT</a>")); }
        let link_html = if !url_s.is_empty() && url_s != "#" {
            format!("<a href=\"{url_s}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Visit</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {stype}</span>\
                  {m_html}\
                </div>\
                {desc_h}\
                <div style=\"margin-top:5px\">{sc_html}</div>\
              </div>\
              {link_html}\
            </div>",
            m_html  = members.map(|m| format!(
                "<span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{m} members</span>"
            )).unwrap_or_default(),
            desc_h  = if !desc.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:3px;line-height:1.5\">{}</div>",
                    desc.chars().take(120).collect::<String>())
            } else { String::new() },
            sc_html = sc.join(" "),
        ))
    }).collect();
    let page_digital_title = i18n::t(i18n::translations(), lang, "page.digital.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">{page_digital_title}</h1>{items}"
    );
    Html(shell("Digital Spaces", "Bear digital spaces.", "now", &body, lang)).into_response()
}
