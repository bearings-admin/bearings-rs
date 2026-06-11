//! Zone: campaigns

use axum::response::{Html, IntoResponse, Response};
use crate::db::LogErr;
use crate::{db::SupabaseClient, i18n, ui::*};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;
use super::super::query::*;

pub(crate) async fn zone_campaigns(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/campaigns?active=eq.true&privacy_mode=eq.false\
         &select=name,org,description,link,goal,raised,currency&order=name.asc",
        db.url
    );
    let campaigns: Vec<CampaignRow> = db.get_json::<Vec<CampaignRow>>(&url).await.or_log("campaigns");
    let items: String = campaigns.iter().map(|c| {
        let name   = esc(c.name.as_str());
        let org    = esc(c.org.as_deref().unwrap_or(""));
        let desc   = esc(c.description.as_deref().unwrap_or(""));
        let link   = esc(c.link.as_deref().unwrap_or(""));
        let goal   = c.goal;
        let raised = c.raised;
        let curr   = esc(c.currency.as_deref().unwrap_or("USD"));
        let progress = match (raised, goal) {
            (Some(r), Some(g)) if g > 0.0 => {
                let pct = ((r / g) * 100.0).min(100.0) as u64;
                format!(
                    "<div style=\"margin-top:8px\">\
                      <div style=\"display:flex;justify-content:space-between;font-size:10px;\
                                  color:{MID};margin-bottom:4px\">\
                        <span>{r:.0} {curr} raised</span><span>goal: {g:.0}</span>\
                      </div>\
                      <div style=\"height:4px;border-radius:999px;background:{TAN}\">\
                        <div style=\"height:4px;border-radius:999px;background:{GOLD};width:{pct}%\"></div>\
                      </div>\
                    </div>"
                )
            },
            _ => String::new(),
        };
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-g\">Donate</a>")
        } else { String::new() };
        card(&format!(
            "<div>\
              <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
                <div style=\"flex:1;min-width:0\">\
                  <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                  <div style=\"font-size:11px;color:{MID};margin-top:2px\">{org}</div>\
                  {desc_h}\
                </div>\
                {link_html}\
              </div>\
              {progress}\
            </div>",
            desc_h = if !desc.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>",
                    desc.chars().take(160).collect::<String>())
            } else { String::new() },
        ))
    }).collect();
    let page_campaigns_title = i18n::t(i18n::translations(), lang, "page.campaigns.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">{page_campaigns_title}</h1>{items}"
    );
    Html(shell("Campaigns", "Community campaigns.", "now", &body, lang)).into_response()
}

// ── ZONE: ICAL EXPORT ─────────────────────────────────────────────────────

