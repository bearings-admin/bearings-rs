//! Zone: digital — online bear community, grouped + showing what is live (events) and where (region).

use super::super::query::*;
use crate::db::{LogErr, SupabaseClient};
use crate::ui::*;
use axum::response::{Html, IntoResponse, Response};
use std::collections::{HashMap, HashSet};

fn bucket_of(stype: &str) -> usize {
    match stype {
        "dating-app" => 0,
        "web-community" | "discord-server" | "game-community" => 1,
        "podcast" | "youtube-channel" | "twitch-channel" => 2,
        "bear-media" => 3,
        _ => 4,
    }
}

fn digital_card(s: &DigitalSpaceRow, events_html: &str, lang: &str) -> String {
    let name = esc(s.name.as_str());
    let stype = esc(s.space_type.as_deref().unwrap_or(""));
    let desc = esc(&crate::content_tx::tc(
        s.description.as_deref().unwrap_or(""),
        lang,
    ));
    let url_s = esc(s.url.as_deref().unwrap_or(""));
    let ig = esc(s.instagram.as_deref().unwrap_or(""));
    let tt = esc(s.tiktok_handle.as_deref().unwrap_or(""));
    let bs = esc(s.bluesky_handle.as_deref().unwrap_or(""));
    let yt = esc(s.youtube_handle.as_deref().unwrap_or(""));

    let mut sc = Vec::new();
    if !ig.is_empty() {
        sc.push(link_badge(
            &format!("https://instagram.com/{ig}"),
            "IG",
            "#E1306C",
        ));
    }
    if !tt.is_empty() {
        sc.push(link_badge(
            &format!("https://tiktok.com/@{tt}"),
            "TikTok",
            "#000",
        ));
    }
    if !bs.is_empty() {
        sc.push(link_badge(
            &format!("https://bsky.app/profile/{bs}"),
            "Bluesky",
            "#0085ff",
        ));
    }
    if !yt.is_empty() {
        sc.push(link_badge(
            &format!("https://youtube.com/{yt}"),
            "YouTube",
            "#FF0000",
        ));
    }

    let region = s
        .country
        .as_deref()
        .filter(|c| !c.is_empty())
        .map(|c| badge(&format!("\u{1f4cd} {c}"), TAN, BROWN))
        .unwrap_or_default();
    let m_html = s
        .member_count
        .map(|m| badge(&format!("{m} members"), TAN, BROWN))
        .unwrap_or_default();
    let link_html = if !url_s.is_empty() && url_s != "#" {
        format!("<a href=\"{url_s}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Visit \u{2192}</a>")
    } else {
        String::new()
    };
    let desc_h = if desc.is_empty() {
        String::new()
    } else {
        format!(
            "<div style=\"font-size:12px;color:{MID};margin-top:3px;line-height:1.5\">{}</div>",
            desc.chars().take(150).collect::<String>()
        )
    };
    let sc_block = if sc.is_empty() {
        String::new()
    } else {
        format!(
            "<div style=\"margin-top:6px;display:flex;gap:4px;flex-wrap:wrap\">{}</div>",
            sc.join("")
        )
    };
    let content = format!("<div style=\"font-weight:600;font-size:14px\">{name} <span style=\"font-weight:400;font-size:11px;color:{MID}\">{stype}</span> {region} {m_html}</div>{desc_h}{events_html}{sc_block}");
    card(&split(&content, &link_html))
}

pub(crate) async fn zone_digital(db: SupabaseClient, lang: &str) -> Response {
    let url = format!("{}/rest/v1/digital_spaces?active=eq.true&select=id,name,space_type,description,url,member_count,instagram,tiktok_handle,bluesky_handle,youtube_handle,country&order=member_count.desc.nullslast,name.asc&limit=200", db.url);
    let spaces: Vec<DigitalSpaceRow> = db
        .get_json::<Vec<DigitalSpaceRow>>(&url)
        .await
        .or_log("digital");

    #[derive(serde::Deserialize)]
    struct EvLite {
        name: String,
        start_date: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Link {
        digital_space_id: i64,
        relationship: Option<String>,
        events: Option<EvLite>,
    }
    let links_url = format!("{}/rest/v1/digital_space_event_links?select=digital_space_id,relationship,events(name,start_date)&limit=300", db.url);
    let links: Vec<Link> = db
        .get_json::<Vec<Link>>(&links_url)
        .await
        .or_log("digital:links");

    let mut ev_map: HashMap<i64, Vec<String>> = HashMap::new();
    let mut seen: HashMap<i64, HashSet<String>> = HashMap::new();
    for l in &links {
        if let Some(ev) = &l.events {
            if !seen
                .entry(l.digital_space_id)
                .or_default()
                .insert(ev.name.clone())
            {
                continue;
            }
            let date = ev
                .start_date
                .as_deref()
                .map(|d| format!(" \u{00b7} {}", esc(d)))
                .unwrap_or_default();
            let rel = esc(l.relationship.as_deref().unwrap_or("hosts"));
            ev_map.entry(l.digital_space_id).or_default().push(format!(
                "<div>{} {}{}</div>",
                esc(&ev.name),
                rel_paren(&rel),
                date
            ));
        }
    }

    let events_html_for = |id: Option<i64>| -> String {
        match id.and_then(|i| ev_map.get(&i)) {
            Some(list) if !list.is_empty() => format!("<div style=\"margin-top:6px;font-size:11px;color:{BROWN}\"><span style=\"font-weight:600\">\u{1f5d3} What is on here</span>{}</div>", list.join("")),
            _ => String::new(),
        }
    };

    const BUCKETS: [&str; 5] = [
        "Apps to meet bears",
        "Communities & servers",
        "Watch & listen",
        "Bear media & press",
        "More online spaces",
    ];
    let mut body = format!("<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Digital Spaces</h1><p style=\"font-size:12px;color:{MID};margin-bottom:14px\">Where the bear community gathers online \u{2014} apps, servers, shows and media, with what is live and where. For regions without bear venues, these are the front door.</p>");
    for (i, label) in BUCKETS.iter().enumerate() {
        let group: Vec<&DigitalSpaceRow> = spaces
            .iter()
            .filter(|s| bucket_of(s.space_type.as_deref().unwrap_or("")) == i)
            .collect();
        if group.is_empty() {
            continue;
        }
        body.push_str(&sh(label, Some(group.len())));
        for &s in &group {
            body.push_str(&digital_card(s, &events_html_for(s.id), lang));
        }
    }
    Html(shell(
        "Digital Spaces",
        "Where the bear community gathers online.",
        "now",
        &body,
        lang,
    ))
    .into_response()
}

/// Render a relationship as " (organiser)" unless it is the default "hosts".
fn rel_paren(rel: &str) -> String {
    if rel == "hosts" {
        String::new()
    } else {
        format!(" ({rel})")
    }
}
