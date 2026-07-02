//! Zone: shops — bear-owned shops and community gear (split out from creators).

use super::super::query::*;
use crate::db::{LogErr, SupabaseClient};
use crate::ui::*;
use axum::response::{Html, IntoResponse, Response};

fn store_card(st: &StoreRow, lang: &str) -> String {
    let sname = esc(st.name.as_str());
    let stype = esc(st.store_type.as_deref().unwrap_or(""));
    let slink = esc(st.link.as_deref().unwrap_or(""));
    let saff = esc(st.affiliate_link.as_deref().unwrap_or(""));
    let sdesc = esc(&crate::content_tx::tc(
        st.description.as_deref().unwrap_or(""),
        lang,
    ));

    let mut badges: Vec<String> = Vec::new();
    if st.bear_owned.unwrap_or(false) {
        badges.push(badge("bear-owned", GOLD, DARK));
    }
    if st.size_inclusive.unwrap_or(false) {
        badges.push(badge("size incl.", TAN, BROWN));
    }
    if st.ships_global.unwrap_or(false) {
        badges.push(format!("<span class=\"badge\" style=\"background:{OFF_WHITE};color:{MID};border:1px solid {TAN}\">ships worldwide</span>"));
    }

    // Marketplace product -> affiliate "Buy" + disclosure; bear-owned/direct -> "Shop".
    let (aside, aff_note) = if !saff.is_empty() && saff != "#" {
        (format!("<a href=\"{saff}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Buy \u{2192}</a>"),
         format!("<div style=\"font-size:9px;color:{MID};margin-top:4px\">affiliate \u{00b7} a small % funds Bearings, at no extra cost to you</div>"))
    } else if !slink.is_empty() && slink != "#" {
        (format!("<a href=\"{slink}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Shop \u{2192}</a>"), String::new())
    } else {
        (String::new(), String::new())
    };

    let desc_h = if sdesc.is_empty() {
        String::new()
    } else {
        format!(
            "<div style=\"font-size:12px;color:{MID};margin-top:3px;line-height:1.5\">{}</div>",
            sdesc.chars().take(140).collect::<String>()
        )
    };
    let content = format!(
        "<div style=\"font-weight:600;font-size:14px\">{sname} <span style=\"font-weight:400;font-size:11px;color:{MID}\">{stype}</span></div>{desc_h}<div style=\"margin-top:5px;display:flex;gap:4px;flex-wrap:wrap\">{badges_h}</div>{aff_note}",
        badges_h = badges.join(""),
    );
    card(&split(&content, &aside))
}

pub(crate) async fn zone_shops(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/stores?active=eq.true&type=neq.book\
         &select=name,type,link,description,bear_owned,size_inclusive,ships_global,featured,affiliate_link,affiliate_pct\
         &order=bear_owned.desc.nullslast,featured.desc.nullslast,name.asc&limit=200",
        db.url
    );
    let stores: Vec<StoreRow> = db.get_json::<Vec<StoreRow>>(&url).await.or_log("shops");
    let (owned, other): (Vec<&StoreRow>, Vec<&StoreRow>) =
        stores.iter().partition(|s| s.bear_owned.unwrap_or(false));

    let mut body = format!("<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bear Shops &amp; Gear</h1><p style=\"font-size:12px;color:{MID};margin-bottom:16px\">Bear-owned shops first \u{2014} buying direct keeps the most in community hands \u{2014} then other gear, leather and marketplaces.</p>");
    if !owned.is_empty() {
        body.push_str(&sh("Bear-owned shops", Some(owned.len())));
        for &s in &owned {
            body.push_str(&store_card(s, lang));
        }
    }
    if !other.is_empty() {
        body.push_str(&sh("More gear & marketplaces", Some(other.len())));
        for &s in &other {
            body.push_str(&store_card(s, lang));
        }
    }
    Html(shell(
        "Bear Shops & Gear",
        "Bear-owned shops and community gear.",
        "shops",
        &body,
        lang,
    ))
    .into_response()
}
