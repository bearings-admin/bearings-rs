//! Zone: creators — bear creators grouped by craft. Authors list their books with buy links.

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
use std::collections::{HashMap, HashSet};

/// Display sections in page order: (heading, creator_type values it collects).
const SECTIONS: &[(&str, &[&str])] = &[
    ("Musicians", &["musician"]),
    ("DJs", &["dj"]),
    ("Authors & historians", &["author", "historian"]),
    (
        "Visual artists & illustrators",
        &[
            "illustrator",
            "visual-artist",
            "photographer",
            "tattoo-artist",
        ],
    ),
    ("Film & video", &["filmmaker"]),
    ("Performers & drag", &["comedian", "drag", "performer"]),
    ("Podcasters & media", &["podcaster"]),
];

pub(crate) async fn zone_creators(db: SupabaseClient, lang: &str) -> Response {
    let url_creators = format!(
        "{}/rest/v1/creators?active=eq.true\
         &select=id,name,creator_type,city,country,bio,website,\
         spotify_link,youtube_link,bandcamp_link,etsy_link,instagram\
         &order=name.asc&limit=200",
        db.url
    );
    let url_media = format!(
        "{}/rest/v1/media?active=eq.true\
         &select=title,creator_id,media_type,year,link,streaming_link,affiliate_link\
         &order=year.desc&limit=300",
        db.url
    );
    let (creators_res, media_res) =
        tokio::join!(db.get_json(&url_creators), db.get_json(&url_media),);
    let creators: Vec<CreatorRow> = creators_res.or_log("creators:creators_res");
    let media_all: Vec<MediaRow> = media_res.or_log("creators:media_res");

    let mut media_by_creator: HashMap<i64, Vec<&MediaRow>> = HashMap::new();
    for m in &media_all {
        if let Some(cid) = m.creator_id {
            media_by_creator.entry(cid).or_default().push(m);
        }
    }

    let creator_card = |c: &CreatorRow| -> String {
        let id = c.id;
        let name = esc(c.name.as_str());
        let ctype = esc(c.creator_type.as_deref().unwrap_or("creator"));
        let city = esc(c.city.as_deref().unwrap_or(""));
        let ctry = esc(c.country.as_deref().unwrap_or(""));
        let bio = esc(&crate::content_tx::tc(c.bio.as_deref().unwrap_or(""), lang));
        let site = esc(c.website.as_deref().unwrap_or(""));
        let sp = esc(c.spotify_link.as_deref().unwrap_or(""));
        let yt = esc(c.youtube_link.as_deref().unwrap_or(""));
        let bc = esc(c.bandcamp_link.as_deref().unwrap_or(""));
        let etsy = esc(c.etsy_link.as_deref().unwrap_or(""));
        let ig = esc(c.instagram.as_deref().unwrap_or(""));

        let mut link_badges: Vec<String> = Vec::new();
        if !sp.is_empty() && sp != "#" {
            link_badges.push(link_badge(&sp, "Spotify", "#1DB954"));
        }
        if !yt.is_empty() && yt != "#" {
            link_badges.push(link_badge(&yt, "YouTube", "#FF0000"));
        }
        if !bc.is_empty() && bc != "#" {
            link_badges.push(link_badge(&bc, "Bandcamp", "#1DA0C3"));
        }
        if !etsy.is_empty() && etsy != "#" {
            link_badges.push(link_badge(&etsy, "Etsy", "#F1641E"));
        }
        if !ig.is_empty() && ig != "#" {
            link_badges.push(link_badge(&ig, "Instagram", "#E1306C"));
        }
        let site_btn = if !site.is_empty() && site != "#" {
            format!(
                "<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>"
            )
        } else {
            String::new()
        };

        let media_html: String = media_by_creator.get(&id).map(|items| {
            items.iter().take(6).map(|m| {
                let mtitle  = esc(m.title.as_deref().unwrap_or(""));
                let mtype   = m.media_type.as_deref().unwrap_or("");
                let myear   = m.year.map(|y| format!(" ({})", y as i64)).unwrap_or_default();
                let aff     = esc(m.affiliate_link.as_deref().unwrap_or(""));
                let mlink   = esc(m.link.as_deref().unwrap_or(""));
                let mstream = esc(m.streaming_link.as_deref().unwrap_or(""));
                let is_book_aff = mtype == "book" && !aff.is_empty() && aff != "#";
                let href = if is_book_aff { aff.clone() }
                    else if !mstream.is_empty() && mstream != "#" { mstream }
                    else { mlink };
                let dot_col = match mtype {
                    "album" => "#1DB954", "documentary" => "#c44444", "book" => "#D4A017",
                    "podcast" => "#8940FA", "music-video" => "#E1306C", _ => "#999999",
                };
                let label = if !href.is_empty() && href != "#" {
                    format!("<a href=\"{href}\" target=\"_blank\" rel=\"noopener\" style=\"color:{BROWN};text-decoration:none\">{mtitle}</a>")
                } else { mtitle.to_string() };
                // Gold-on-dark "Buy" pill (link_badge is white-on-colour), so this one stays explicit.
                let buy = if is_book_aff {
                    format!(" <a href=\"{aff}\" target=\"_blank\" rel=\"noopener\" class=\"badge\" style=\"background:{GOLD};color:{DARK}\">Buy on Amazon \u{2197}</a>")
                } else { String::new() };
                format!("<div style=\"font-size:11px;margin-top:3px;display:flex;align-items:center;gap:5px;flex-wrap:wrap\"><span style=\"display:inline-block;width:6px;height:6px;border-radius:50%;background:{dot_col};flex-shrink:0\"></span>{label}<span style=\"color:{MID}\">{mtype}{myear}</span>{buy}</div>")
            }).collect()
        }).unwrap_or_default();

        let content = format!(
            "<div style=\"font-weight:600;font-size:14px\">{name} <span style=\"font-weight:400;font-size:11px;color:{MID}\">{ctype}</span></div><div style=\"font-size:12px;color:{MID}\">{city}{sep}{ctry}</div>{bio_h}{media_html}{links_h}",
            sep    = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            bio_h  = if !bio.is_empty() { format!("<div style=\"font-size:12px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>", bio.chars().take(160).collect::<String>()) } else { String::new() },
            links_h = if !link_badges.is_empty() { format!("<div style=\"margin-top:6px;display:flex;gap:4px;flex-wrap:wrap\">{}</div>", link_badges.join("")) } else { String::new() },
        );
        card(&split(&content, &site_btn))
    };

    let mut body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Creators &amp; Makers</h1>\
         <p style=\"font-size:12px;color:{MID};margin-bottom:6px\">Musicians, authors, illustrators, filmmakers and performers building bear culture.</p>\
         <p style=\"font-size:10px;color:{MID};margin-bottom:14px\">Book links are Amazon affiliate links \u{2014} a small % funds Bearings, at no extra cost to you.</p>"
    );

    let mut shown: HashSet<i64> = HashSet::new();
    for (heading, types) in SECTIONS {
        let group: Vec<&CreatorRow> = creators
            .iter()
            .filter(|c| types.contains(&c.creator_type.as_deref().unwrap_or("")))
            .collect();
        if group.is_empty() {
            continue;
        }
        body.push_str(&sh(heading, Some(group.len())));
        for &c in &group {
            shown.insert(c.id);
            body.push_str(&creator_card(c));
        }
    }
    let rest: Vec<&CreatorRow> = creators.iter().filter(|c| !shown.contains(&c.id)).collect();
    if !rest.is_empty() {
        body.push_str(&sh("More creators", Some(rest.len())));
        for &c in &rest {
            body.push_str(&creator_card(c));
        }
    }

    Html(shell(
        "Creators & Makers",
        "Bear community creators, by craft.",
        "creators",
        &body,
        lang,
    ))
    .into_response()
}
