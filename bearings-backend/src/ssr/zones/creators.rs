//! Zone: creators

use axum::response::{Html, IntoResponse, Response};
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;
use super::super::query::*;

pub(crate) async fn zone_creators(db: SupabaseClient, lang: &str) -> Response {
    // Fetch creators, their media, and stores in parallel
    let url_creators = format!(
        "{}/rest/v1/creators?active=eq.true\
         &select=id,name,creator_type,city,country,bio,website,\
         spotify_link,youtube_link,bandcamp_link,etsy_link,instagram\
         &order=creator_type.asc,name.asc&limit=100",
        db.url
    );
    let url_media = format!(
        "{}/rest/v1/media?active=eq.true\
         &select=title,creator_id,media_type,year,link,streaming_link\
         &order=year.desc&limit=200",
        db.url
    );
    let url_stores = format!(
        "{}/rest/v1/stores?active=eq.true\
         &select=name,type,link,description,bear_owned,size_inclusive,ships_global,featured\
         &order=featured.desc.nullslast,name.asc&limit=100",
        db.url
    );
    let (creators_res, media_res, stores_res) = tokio::join!(
        db.get_json(&url_creators),
        db.get_json(&url_media),
        db.get_json(&url_stores),
    );
    let creators: Vec<CreatorRow> = creators_res.or_log("creators:creators_res");
    let media_all: Vec<MediaRow> = media_res.or_log("creators:media_res");
    let stores: Vec<StoreRow> = stores_res.or_log("creators:stores_res");

    // Group media by creator_id
    let mut media_by_creator: std::collections::HashMap<i64, Vec<&MediaRow>> =
        std::collections::HashMap::new();
    for m in &media_all {
        if let Some(cid) = m.creator_id {
            media_by_creator.entry(cid).or_default().push(m);
        }
    }

    let creator_cards: String = creators.iter().map(|c| {
        let id    = c.id;
        let name  = esc(c.name.as_str());
        let ctype = esc(c.creator_type.as_deref().unwrap_or("creator"));
        let city  = esc(c.city.as_deref().unwrap_or(""));
        let ctry  = esc(c.country.as_deref().unwrap_or(""));
        let bio   = esc(c.bio.as_deref().unwrap_or(""));
        let site  = esc(c.website.as_deref().unwrap_or(""));
        let sp    = esc(c.spotify_link.as_deref().unwrap_or(""));
        let yt    = esc(c.youtube_link.as_deref().unwrap_or(""));
        let bc    = esc(c.bandcamp_link.as_deref().unwrap_or(""));
        let etsy  = esc(c.etsy_link.as_deref().unwrap_or(""));
        let ig    = esc(c.instagram.as_deref().unwrap_or(""));

        let mut link_badges: Vec<String> = Vec::new();
        if !sp.is_empty() && sp != "#" {
            link_badges.push(format!(
                "<a href=\"{sp}\" target=\"_blank\" class=\"badge\" \
                   style=\"background:#1DB954;color:#fff\">Spotify</a>"
            ));
        }
        if !yt.is_empty() && yt != "#" {
            link_badges.push(format!(
                "<a href=\"{yt}\" target=\"_blank\" class=\"badge\" \
                   style=\"background:#FF0000;color:#fff\">YouTube</a>"
            ));
        }
        if !bc.is_empty() && bc != "#" {
            link_badges.push(format!(
                "<a href=\"{bc}\" target=\"_blank\" class=\"badge\" \
                   style=\"background:#1DA0C3;color:#fff\">Bandcamp</a>"
            ));
        }
        if !etsy.is_empty() && etsy != "#" {
            link_badges.push(format!(
                "<a href=\"{etsy}\" target=\"_blank\" class=\"badge\" \
                   style=\"background:#F1641E;color:#fff\">Etsy</a>"
            ));
        }
        if !ig.is_empty() && ig != "#" {
            link_badges.push(format!(
                "<a href=\"{ig}\" target=\"_blank\" class=\"badge\" \
                   style=\"background:#E1306C;color:#fff\">Instagram</a>"
            ));
        }
        let site_btn = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>")
        } else { String::new() };

        let media_html: String = media_by_creator
            .get(&id)
            .map(|items| {
                items.iter().take(4).map(|m| {
                    let mtitle  = esc(m.title.as_deref().unwrap_or(""));
                    let mtype   = m.media_type.as_deref().unwrap_or("");
                    let myear   = m.year.map(|y| y as i64)
                        .map(|y| format!(" ({y})"))
                        .unwrap_or_default();
                    let mlink   = esc(m.link.as_deref().unwrap_or(""));
                    let mstream = esc(m.streaming_link.as_deref().unwrap_or(""));
                    let href    = if !mstream.is_empty() && mstream != "#" { mstream } else { mlink };
                    let dot_col = match mtype {
                        "album"         => "#1DB954",
                        "documentary"   => "#c44444",
                        "book"          => "#D4A017",
                        "podcast"       => "#8940FA",
                        "music-video"   => "#E1306C",
                        _               => "#999999",
                    };
                    let label = if !href.is_empty() && href != "#" {
                        format!(
                            "<a href=\"{href}\" target=\"_blank\" \
                               style=\"color:{BROWN};text-decoration:none\">{mtitle}</a>"
                        )
                    } else {
                        mtitle.to_string()
                    };
                    format!(
                        "<div style=\"font-size:11px;margin-top:3px;\
                                     display:flex;align-items:center;gap:5px\">\
                          <span style=\"display:inline-block;width:6px;height:6px;\
                                       border-radius:50%;background:{dot_col};\
                                       flex-shrink:0\"></span>\
                          {label}\
                          <span style=\"color:{MID}\">{mtype}{myear}</span>\
                        </div>"
                    )
                }).collect()
            })
            .unwrap_or_default();

        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;\
                         align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ctype}</span>\
                </div>\
                <div style=\"font-size:12px;color:{MID}\">{city}{sep}{ctry}</div>\
                {bio_h}\
                {media_html}\
                {links_h}\
              </div>\
              {site_btn}\
            </div>",
            sep    = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            bio_h  = if !bio.is_empty() {
                format!(
                    "<div style=\"font-size:12px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>",
                    bio.chars().take(160).collect::<String>()
                )
            } else { String::new() },
            links_h = if !link_badges.is_empty() {
                format!(
                    "<div style=\"margin-top:6px;display:flex;gap:4px;flex-wrap:wrap\">{}</div>",
                    link_badges.join("")
                )
            } else { String::new() },
        ))
    }).collect();

    let store_cards: String = stores.iter().map(|st| {
        let sname   = esc(st.name.as_str());
        let stype   = esc(st.store_type.as_deref().unwrap_or(""));
        let slink   = esc(st.link.as_deref().unwrap_or(""));
        let sdesc   = esc(st.description.as_deref().unwrap_or(""));
        let owned   = st.bear_owned.unwrap_or(false);
        let szinc   = st.size_inclusive.unwrap_or(false);
        let sglobal = st.ships_global.unwrap_or(false);
        let mut badges: Vec<String> = Vec::new();
        if owned  { badges.push(format!("<span class=\"badge\" style=\"background:{GOLD};color:{DARK}\">bear-owned</span>")); }
        if szinc  { badges.push(format!("<span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">size incl.</span>")); }
        if sglobal { badges.push(format!("<span class=\"badge\" style=\"background:{OFF_WHITE};color:{MID};border:1px solid {TAN}\">ships worldwide</span>")); }
        let shop_btn = if !slink.is_empty() && slink != "#" {
            format!("<a href=\"{slink}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Shop</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;\
                         align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{sname}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {stype}</span>\
                </div>\
                {desc_h}\
                <div style=\"margin-top:5px;display:flex;gap:4px;flex-wrap:wrap\">{badges_h}</div>\
              </div>\
              {shop_btn}\
            </div>",
            desc_h  = if !sdesc.is_empty() {
                format!(
                    "<div style=\"font-size:12px;color:{MID};margin-top:3px;line-height:1.5\">{}</div>",
                    sdesc.chars().take(140).collect::<String>()
                )
            } else { String::new() },
            badges_h = badges.join(""),
        ))
    }).collect();

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">\
          Creators &amp; Makers</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          Musicians, filmmakers, illustrators, historians and more building bear culture.\
        </p>\
        {h_creators}\
        {creator_cards}\
        {h_shops}\
        {store_cards}",
        h_creators = sh("Bear Creators", Some(creators.len())),
        h_shops    = sh("Bear Shops", Some(stores.len())),
    );
    Html(shell(
        "Creators & Makers",
        "Bear community creators and shops.",
        "creators",
        &body,
        lang,
    )).into_response()
}


