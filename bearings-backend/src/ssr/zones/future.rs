//! Zone: future

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_future(db: SupabaseClient, lang: &str) -> Response {
    // Fetch active campaigns
    let url_camps = format!(
        "{}/rest/v1/campaigns\
         ?active=eq.true&privacy_mode=eq.false\
         &select=name,org,description,link,raised,goal,currency,urgent,ends_at\
         &order=urgent.desc,ends_at.asc.nullslast\
         &limit=20",
        db.url
    );
    // Fetch recent titleholders (milestones)
    let url_recent = format!(
        "{}/rest/v1/title_holders\
         ?year=gte.2023&active=eq.true\
         &select=holder_name,year,city,country,title_name,competition_id,inclusion_flag_codes,bio\
         &order=year.desc&limit=20",
        db.url
    );
    let (camps_res, recent_res) = tokio::join!(
        db.get_json::<Vec<CampaignRow>>(&url_camps),
        db.get_json::<Vec<TitleHolderRow>>(&url_recent),
    );
    let campaigns: Vec<CampaignRow> = camps_res.or_log("future:camps_res");
    let recent_title: Vec<TitleHolderRow> = recent_res.or_log("future:recent_res");

    // ── Section 1: Active Campaigns ───────────────────────────
    let camp_cards: String = campaigns.iter().map(|c| {
        let name   = esc(c.name.as_str());
        let org    = esc(c.org.as_deref().unwrap_or(""));
        let desc   = esc(c.description.as_deref().unwrap_or(""));
        let link   = esc(c.link.as_deref().unwrap_or(""));
        let raised = c.raised.map(|x| x as i64);
        let goal   = c.goal.map(|x| x as i64);
        let curr   = esc(c.currency.as_deref().unwrap_or("USD"));
        let urgent = c.urgent.unwrap_or(false);
        let ends   = esc(c.ends_at.as_deref().unwrap_or(""));

        let progress_html = match (raised, goal) {
            (Some(r), Some(g)) if g > 0 => {
                let pct   = ((r as f64 / g as f64) * 100.0).min(100.0);
                let r_fmt = if r >= 1_000_000 { format!("${:.1}M", r as f64/1_000_000.0) }
                            else if r >= 1_000 { format!("${:.0}k", r as f64/1_000.0) }
                            else { format!("${r}") };
                let g_fmt = if g >= 1_000_000 { format!("${:.1}M", g as f64/1_000_000.0) }
                            else if g >= 1_000 { format!("${:.0}k", g as f64/1_000.0) }
                            else { format!("${g}") };
                format!(
                    "<div style=\"margin:8px 0\">\
                      <div style=\"background:{TAN};border-radius:4px;height:6px;overflow:hidden\">\
                        <div style=\"background:{ORANGE};height:100%;width:{pct:.0}%\"></div>\
                      </div>\
                      <div style=\"font-size:10px;color:{MID};margin-top:2px\">\
                        {r_fmt} raised of {g_fmt} {curr}</div>\
                    </div>"
                )
            },
            (Some(r), None) => {
                let r_fmt = if r >= 1_000_000 { format!("${:.1}M", r as f64/1_000_000.0) }
                            else if r >= 1_000 { format!("${:.0}k", r as f64/1_000.0) }
                            else { format!("${r}") };
                format!("<div style=\"font-size:11px;color:{ORANGE};margin:4px 0\">{r_fmt} raised {curr}</div>")
            },
            _ => String::new(),
        };

        let link_btn = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Donate</a>")
        } else { String::new() };

        let ends_html = if !ends.is_empty() {
            format!("<div style=\"font-size:10px;color:{MID}\">Ends {ends}</div>")
        } else { String::new() };

        let urgent_badge = if urgent {
            "<span style=\"font-size:9px;background:#C0392B;color:#fff;\
                      border-radius:6px;padding:2px 6px;margin-right:4px\">URGENT</span>".to_string()
        } else { String::new() };

        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{urgent_badge}{name}</div>\
                <div style=\"font-size:11px;color:{ORANGE};font-weight:600;margin:2px 0\">{org}</div>\
                <div style=\"font-size:12px;color:{MID};line-height:1.6;margin-top:4px\">{short_desc}</div>\
                {progress_html}\
                {ends_html}\
              </div>\
              <div style=\"flex-shrink:0\">{link_btn}</div>\
            </div>",
            short_desc = desc.chars().take(220).collect::<String>(),
        ))
    }).collect();

    // ── Section 2: Breaking Ground ────────────────────────────
    let milestone_cards: String = recent_title.iter().map(|h| {
        let name  = esc(h.holder_name.as_str());
        let year  = h.year.unwrap_or(0) as i64;
        let title = esc(h.title_name.as_deref().unwrap_or(""));
        let city  = esc(h.city.as_deref().unwrap_or(""));
        let ctry  = esc(h.country.as_deref().unwrap_or(""));
        let bio   = esc(h.bio.as_deref().unwrap_or(""));
        let fs: Vec<String> = h.inclusion_flag_codes.clone().unwrap_or_default();
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1\">\
                <div style=\"font-weight:600;font-size:14px\">{name}</div>\
                <div style=\"font-size:11px;color:{ORANGE};font-weight:600\">{title}</div>\
                <div style=\"font-size:11px;color:{MID}\">{loc}</div>\
                {bio_h}\
                {fhtml}\
              </div>\
              <div style=\"font-size:20px;font-weight:700;color:{GOLD};flex-shrink:0\">{year}</div>\
            </div>",
            loc   = match (city, ctry) {
                (c, ct) if !c.is_empty() && !ct.is_empty() => format!("{c}, {ct}"),
                (c, _)  if !c.is_empty() => c.to_string(),
                (_, ct) if !ct.is_empty() => ct.to_string(),
                _ => String::new(),
            },
            bio_h = if !bio.is_empty() {
                format!("<div style=\"font-size:11px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>",
                    bio.chars().take(200).collect::<String>())
            } else { String::new() },
            fhtml = if !fs.is_empty() {
                format!("<div style=\"margin-top:4px\">{}</div>", flags(&fs))
            } else { String::new() },
        ))
    }).collect();

    // ── Section 3: New Bear Territories ──────────────────────
    let regions_html = format!(
        "<div class=\"card\">\
          <div style=\"font-size:13px;line-height:1.8;color:{DARK}\">\
            <p style=\"margin:0 0 10px\">The bear community is growing into new regions — some with full \
              support from local law and culture, others where community members face serious legal risk.</p>\
            <p style=\"margin:0 0 8px\">Safety data is maintained by \
              <a href=\"https://ilga.org\" target=\"_blank\" rel=\"noopener\" \
                 style=\"color:{ORANGE}\">ILGA World</a>, \
              which tracks the legal status of same-sex relationships in every country.</p>\
            <div style=\"border-left:3px solid {ORANGE};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Malaysia</strong> — Homosexuality is criminalized under both \
              civil and Sharia law. In 2026, the first Mr Bear International titleholder from Malaysia (name withheld for safety) struggled to find a venue willing to host the national qualifier. \
              His reign is an act of visibility under genuine personal risk.\
            </div>\
            <div style=\"border-left:3px solid {GOLD};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Middle East &amp; North Africa</strong> — \
              Dori, a Syrian refugee resettled in Canada, was crowned Mr Ottawa Bear (2020) and champions Northern Lights Refuge, the LGBTQ+ refugee organisation that once helped him. Bilal Sakr (Mr Bear Montreal 2024) fundraises for Rainbow Railroad, which supports LGBTQ+ people fleeing the region.\
            </div>\
            <div style=\"border-left:3px solid {TAN};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Eastern Europe &amp; Central Asia</strong> — \
              Poland and Czech Republic have established competitions; Hungary has regressed on rights. \
              ILGA''s rainbow map is the current best reference.\
            </div>\
            <div style=\"border-left:3px solid {TAN};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Sub-Saharan Africa &amp; Southeast Asia</strong> — \
              South Africa remains the continent''s primary bear hub. Thailand hosts Mr Bear International. \
              Other countries require significant caution — check ILGA data before travel.\
            </div>\
          </div>\
        </div>"
    );

    let h3 = format!(
        "<div style=\"font-size:14px;font-weight:700;color:{BROWN};margin:12px 0 6px;\
               border-left:3px solid {ORANGE};padding-left:8px\">New Bear Territories</div>"
    );
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bear Future</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          How bears are already making tomorrow better.</p>\
        \
        {h1}\
        {camp_cards}\
        {empty_camps}\
        \
        {h2}\
        {milestone_cards}\
        {empty_milestones}\
        \
        {h3}\
        {regions_html}",
        h1 = sh("Bears Taking Action", Some(campaigns.len())),
        h2 = sh("Breaking Ground — Recent Milestones", Some(recent_title.len())),
        empty_camps = if campaigns.is_empty() {
            format!("<div style=\"font-size:12px;color:{MID};padding:8px\">No active campaigns.</div>")
        } else { String::new() },
        empty_milestones = if recent_title.is_empty() {
            format!("<div style=\"font-size:12px;color:{MID};padding:8px\">No recent titleholders.</div>")
        } else { String::new() },
    );
    Html(shell(
        "Bear Future",
        "Community direction and what comes next.",
        "future",
        &body,
        lang,
    ))
    .into_response()
}
