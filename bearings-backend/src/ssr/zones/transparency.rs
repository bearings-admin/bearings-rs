//! Zone: transparency — public budget, costs, and the wallet that keeps the lights on.

use crate::db::{LogErr, SupabaseClient};
use crate::repositories::transparency_repo::{
    SupabaseTransparencyRepository, TransparencyRepository,
};
use crate::ui::*;
use axum::response::{Html, IntoResponse, Response};

pub(crate) async fn zone_transparency(db: SupabaseClient, lang: &str) -> Response {
    let repo = SupabaseTransparencyRepository::new(db);
    let costs = repo.costs().await.or_log("transparency:costs");
    let wallet = repo.wallet().await.or_log("transparency:wallet");
    let sources = repo.sources().await.or_log("transparency:sources");
    let feeds = repo.feeds().await.or_log("transparency:feeds");

    let monthly: f64 = costs
        .iter()
        .filter(|c| c.cadence == "monthly")
        .map(|c| c.amount_usd)
        .sum();
    let annual: f64 = costs
        .iter()
        .filter(|c| c.cadence == "annual")
        .map(|c| c.amount_usd)
        .sum();
    let monthly_burn = monthly + annual / 12.0;

    let runway = if monthly_burn > 0.0 {
        format!("{:.1} months", wallet.balance_usd / monthly_burn)
    } else if wallet.balance_usd > 0.0 {
        "fully covered".to_string()
    } else {
        "—".to_string()
    };

    let rows: String = costs.iter().map(|c| {
        let amt = if c.amount_usd == 0.0 { "free".to_string() } else { format!("${:.2}", c.amount_usd) };
        let note = c.note.as_deref().map(|n| format!(
            "<div style=\"font-size:11px;color:{MID};margin-top:2px\">{}</div>", esc(n)
        )).unwrap_or_default();
        format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:baseline;\
                         padding:8px 0;border-bottom:1px solid {TAN}\">\
              <div><span style=\"font-weight:600\">{label}</span>\
                <span style=\"font-size:11px;color:{MID};margin-left:6px\">/{cadence}</span>{note}</div>\
              <div style=\"font-weight:700;color:{BROWN}\">{amt}</div>\
            </div>",
            label = esc(&c.label), cadence = esc(&c.cadence),
        )
    }).collect();

    let explorer = if wallet.address.is_empty() {
        format!("<div style=\"font-size:12px;color:{MID}\">Wallet address not yet published.</div>")
    } else {
        let url = if wallet.chain.eq_ignore_ascii_case("base") {
            format!("https://basescan.org/address/{}", wallet.address)
        } else {
            format!("https://blockscan.com/address/{}", wallet.address)
        };
        format!(
            "<a href=\"{url}\" target=\"_blank\" style=\"font-size:12px;color:{ORANGE};word-break:break-all\">{addr} \u{2197}</a>",
            url = esc(&url), addr = esc(&wallet.address),
        )
    };

    let updated = if wallet.updated.is_empty() {
        String::new()
    } else {
        format!(
            "<div style=\"font-size:11px;color:{MID};margin-top:6px\">Balance updated {}</div>",
            esc(&wallet.updated)
        )
    };

    // --- Source transparency: the credit that mirrors the financial disclosure ---

    let sources_block = if sources.is_empty() {
        String::new()
    } else {
        let items: String = sources.iter().map(|s| {
            let lang = s.language.as_deref()
                .filter(|l| !l.is_empty() && !l.eq_ignore_ascii_case("en"))
                .map(|l| format!(
                    " <span style=\"font-size:10px;color:{MID};border:1px solid {TAN};border-radius:3px;padding:0 4px\">{}</span>",
                    esc(&l.to_uppercase())
                )).unwrap_or_default();
            let kind = s.kind.as_deref()
                .map(|k| format!(" <span style=\"font-size:11px;color:{MID}\">· {}</span>", esc(k)))
                .unwrap_or_default();
            let blurb = s.blurb.as_deref()
                .map(|b| format!("<div style=\"font-size:11px;color:{MID};margin-top:2px\">{}</div>", esc(b)))
                .unwrap_or_default();
            format!(
                "<div style=\"padding:8px 0;border-bottom:1px solid {TAN}\">\
                  <a href=\"{url}\" target=\"_blank\" rel=\"noopener\" \
                     style=\"font-weight:600;color:{ORANGE}\">{name} \u{2197}</a>{kind}{lang}{blurb}</div>",
                url = esc(&s.url), name = esc(&s.name),
            )
        }).collect();
        format!(
            "{h}<p style=\"font-size:11px;color:{MID};margin:-6px 0 8px\">\
               The guides, magazines and archives we lean on.</p>\
             <div class=\"card\">{items}</div>\
             <p style=\"font-size:11px;color:{MID};margin-top:8px;line-height:1.5\">\
               We don\u{2019}t sell ads. These are resources we rely on and admire \u{2014} go support them.</p>",
            h = sh("Kindred sources", None),
        )
    };

    let feeds_block = if feeds.is_empty() {
        String::new()
    } else {
        let items: String = feeds.iter().map(|f| {
            let label = f.org_name.as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| f.url.clone());
            let ft = match f.feed_type.as_deref().unwrap_or("feed").to_lowercase().as_str() {
                "ical" => "iCal".to_string(),
                "rss" => "RSS".to_string(),
                other => other.to_uppercase(),
            };
            format!(
                "<div style=\"display:flex;justify-content:space-between;gap:10px;align-items:baseline;\
                             padding:6px 0;border-bottom:1px solid {TAN}\">\
                  <a href=\"{url}\" target=\"_blank\" rel=\"noopener\" \
                     style=\"color:{BROWN};font-size:13px;word-break:break-word\">{label}</a>\
                  <span style=\"font-size:10px;color:{MID};white-space:nowrap\">{ft}</span></div>",
                url = esc(&f.url), label = esc(&label), ft = esc(&ft),
            )
        }).collect();
        format!(
            "{h}<p style=\"font-size:11px;color:{MID};margin:-6px 0 8px\">\
               Public iCal &amp; RSS sources our agents check nightly.</p>\
             <div class=\"card\">{items}</div>\
             <p style=\"font-size:11px;color:{MID};margin-top:8px;line-height:1.5\">\
               Our nightly reader pulls these public iCal &amp; RSS feeds from community organisers \u{2014} \
               many of the events you see start here. Thank you for keeping them open.</p>",
            h = sh("Feeds we read", None),
        )
    };

    let ai_block = format!(
        "{h}<p style=\"font-size:11px;color:{MID};margin:-6px 0 8px\">\
           Disclosed, like our affiliates and our costs.</p>\
         <div class=\"card\">\
          <p style=\"font-size:12px;color:{DARK};line-height:1.6;margin:0\">\
            Research is assisted by AI agents \u{2014} a nightly feed reader and a weekly \u{201c}keeper.\u{201d} \
            They <strong>propose, never publish</strong>: a human approves every change. \
            Records sourced from others carry that source in their notes, and primary-source \
            evidence (photos, plaques, documents) is preserved as artifacts in the Archive.</p></div>",
        h = sh("How we use AI", None),
    );

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Transparency</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          Bearings is community infrastructure, run as lean as possible. Here is exactly what it \
          costs to keep the lights on, the wallet that pays for it, and the sources we build on \u{2014} \
          money and knowledge, disclosed the same way.</p>\
        {budget_h}\
        <div class=\"card\">{rows}\
          <div style=\"display:flex;justify-content:space-between;align-items:baseline;padding:10px 0 2px\">\
            <div style=\"font-weight:700\">Monthly burn (annual costs \u{00f7} 12)</div>\
            <div style=\"font-weight:700;color:{ORANGE}\">${burn:.2}/mo</div></div>\
        </div>\
        {wallet_h}\
        <div class=\"card\">\
          <div style=\"display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px\">\
            <div style=\"font-size:13px;color:{MID}\">Balance ({chain})</div>\
            <div style=\"font-size:22px;font-weight:700;color:{BROWN}\">${bal:.2}</div></div>\
          <div style=\"display:flex;justify-content:space-between;align-items:baseline;margin-bottom:10px\">\
            <div style=\"font-size:13px;color:{MID}\">Runway at current burn</div>\
            <div style=\"font-weight:700;color:{GOLD}\">{runway}</div></div>\
          {explorer}{updated}\
        </div>\
        <p style=\"font-size:11px;color:{MID};margin-top:14px;line-height:1.5\">\
          SSL is free (Let\u{2019}s Encrypt, auto-renewing). The project takes no profit \u{2014} any \
          surplus stays in the wallet as runway. Anyone can verify the balance on the explorer above.</p>\
        {sources_block}{feeds_block}{ai_block}\
        <p style=\"font-size:11px;color:{MID};margin-top:14px;line-height:1.5\">\
          World map: <a href=\"https://github.com/flekschas/simple-world-map\" target=\"_blank\" \
            style=\"color:{ORANGE}\">simple-world-map</a> by Al MacDonald / Fritz Lekschas, \
          <a href=\"https://creativecommons.org/licenses/by-sa/3.0/\" target=\"_blank\" \
            style=\"color:{ORANGE}\">CC BY-SA 3.0</a>.</p>",
        budget_h = sh("What it costs", None),
        wallet_h = sh("Keep-the-lights-on wallet", None),
        burn = monthly_burn, bal = wallet.balance_usd,
        chain = esc(&wallet.chain),
    );

    Html(shell(
        "Transparency",
        "What it costs to run Bearings, the wallet that pays for it, and the sources we credit.",
        "transparency",
        &body,
        lang,
    ))
    .into_response()
}
