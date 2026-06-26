//! Zone: campaigns — bear community fundraising, grouped by cause.
//! Campaigns with a verified one-click donate link surface first ("Give now");
//! event/ticket-based or info-only campaigns rank below ("ongoing giving").

use super::super::query::*;
use crate::db::{LogErr, SupabaseClient};
use crate::ui::*;
use axum::response::{Html, IntoResponse, Response};

fn cause_color(cause: &str) -> &'static str {
    match cause {
        "Refugees & Safety" => "#8c2020",
        "HIV/AIDS" => "#b5179e",
        "Elders & Seniors" => "#2c5aa0",
        "Disability & Inclusion" => "#4a6741",
        "Visibility & Safe Spaces" => ORANGE,
        "Community Funds" => BROWN,
        _ => MID,
    }
}

fn campaign_card(c: &CampaignRow) -> String {
    let name = esc(c.name.as_str());
    let org = esc(c.org.as_deref().unwrap_or(""));
    let desc = esc(c.description.as_deref().unwrap_or(""));
    let curr = esc(c.currency.as_deref().unwrap_or("USD"));
    let cause = c.cause.as_deref().unwrap_or("");

    let mut badges = String::new();
    if !cause.is_empty() {
        badges.push_str(&format!("<span style=\"font-size:9px;font-weight:700;text-transform:uppercase;letter-spacing:.04em;color:#fff;background:{col};border-radius:999px;padding:2px 8px\">{c}</span>", col = cause_color(cause), c = esc(cause)));
    }
    if c.urgent.unwrap_or(false) {
        badges.push_str("<span style=\"font-size:9px;font-weight:700;color:#fff;background:#c0392b;border-radius:999px;padding:2px 7px;margin-left:4px\">URGENT</span>");
    }
    if c.usdc_accepted.unwrap_or(false) {
        badges.push_str(&format!("<span style=\"font-size:9px;font-weight:700;color:{BROWN};background:{TAN};border-radius:999px;padding:2px 7px;margin-left:4px\">USDC \u{2713}</span>"));
    }

    let progress = match (c.raised, c.goal) {
        (Some(r), Some(g)) if g > 0.0 => {
            let pct = ((r / g) * 100.0).min(100.0) as u64;
            format!("<div style=\"margin-top:10px\"><div style=\"display:flex;justify-content:space-between;font-size:10px;color:{MID};margin-bottom:4px\"><span>{r:.0} {curr} raised</span><span>goal {g:.0}</span></div><div style=\"height:5px;border-radius:999px;background:{TAN}\"><div style=\"height:5px;border-radius:999px;background:{GOLD};width:{pct}%\"></div></div></div>")
        },
        (Some(r), None) if r > 0.0 => format!("<div style=\"margin-top:8px;font-size:11px;color:{MID}\">{r:.0} {curr} raised to date</div>"),
        _ => String::new(),
    };

    let action = match c.donate_url.as_deref() {
        Some(u) if !u.is_empty() => format!("<a href=\"{u}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Give now \u{2192}</a>", u = esc(u)),
        _ => match c.link.as_deref() {
            Some(l) if !l.is_empty() && l != "#" => format!("<a href=\"{l}\" target=\"_blank\" rel=\"noopener\" style=\"font-size:12px;color:{MID};white-space:nowrap\">Learn more \u{2192}</a>", l = esc(l)),
            _ => String::new(),
        },
    };

    let desc_h = if desc.is_empty() {
        String::new()
    } else {
        format!(
            "<div style=\"font-size:12px;color:{MID};margin-top:6px;line-height:1.5\">{}</div>",
            desc.chars().take(170).collect::<String>()
        )
    };

    card(&format!("<div><div style=\"margin-bottom:6px\">{badges}</div><div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:12px\"><div style=\"flex:1;min-width:0\"><div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div><div style=\"font-size:11px;color:{MID};margin-top:2px\">{org}</div>{desc_h}</div><div style=\"flex-shrink:0\">{action}</div></div>{progress}</div>"))
}

pub(crate) async fn zone_campaigns(db: SupabaseClient, lang: &str) -> Response {
    let url = format!("{}/rest/v1/campaigns?active=eq.true&privacy_mode=eq.false&select=name,org,description,link,goal,raised,currency,urgent,cause,donate_url,usdc_accepted&order=urgent.desc,raised.desc.nullslast", db.url);
    let campaigns: Vec<CampaignRow> = db
        .get_json::<Vec<CampaignRow>>(&url)
        .await
        .or_log("campaigns");
    let impact: Option<CharityImpactRow> = db
        .get_json::<Vec<CharityImpactRow>>(&format!("{}/rest/v1/charity_impact?select=*", db.url))
        .await
        .or_log("campaigns:impact")
        .into_iter()
        .next();
    let lineages: Vec<CharityLineageRow> = db
        .get_json::<Vec<CharityLineageRow>>(&format!(
            "{}/rest/v1/charity_lineage?people=gte.2&select=*&order=last_year.desc,people.desc",
            db.url
        ))
        .await
        .or_log("campaigns:lineage");

    let (give, ongoing): (Vec<&CampaignRow>, Vec<&CampaignRow>) = campaigns
        .iter()
        .partition(|c| c.donate_url.as_deref().is_some_and(|u| !u.is_empty()));

    let render =
        |list: &[&CampaignRow]| -> String { list.iter().map(|&c| campaign_card(c)).collect() };

    let mut body = format!("<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bears Taking Action</h1><p style=\"font-size:12px;color:{MID};margin-bottom:16px\">Community campaigns, grouped by cause. The bear community has quietly moved millions to HIV/AIDS care, refugee safety, elders and more \u{2014} mostly through events that give 100% of proceeds.</p>");

    if !give.is_empty() {
        body.push_str(&sh(
            "Give now \u{2014} help close these targets",
            Some(give.len()),
        ));
        body.push_str(&render(&give));
    }
    if !ongoing.is_empty() {
        body.push_str(&sh("Community funds & ongoing giving", Some(ongoing.len())));
        body.push_str(&format!("<p style=\"font-size:11px;color:{MID};margin:-2px 0 8px\">These give through events/tickets or have no one-click donation yet \u{2014} follow the link to support them.</p>"));
        body.push_str(&render(&ongoing));
    }

    // ── The Impact — retrospective / history ──────────────────
    let tr = |k: &str| crate::i18n::t(crate::i18n::translations(), lang, k);
    if let Some(im) = impact.as_ref() {
        let raised = im.total_raised.unwrap_or(0);
        let money = if raised >= 1_000_000 {
            format!("${:.1}M+", raised as f64 / 1_000_000.0)
        } else if raised >= 1_000 {
            format!("${}k+", raised / 1_000)
        } else {
            format!("${raised}")
        };
        let stat = |big: &str, label: &str| -> String {
            format!(
                "<div style=\"flex:1;min-width:110px;background:#f3eee3;border-radius:12px;\
                   padding:12px 14px;text-align:center\">\
                   <div style=\"font-size:22px;font-weight:700;color:{BROWN}\">{big}</div>\
                   <div style=\"font-size:11px;color:{MID};margin-top:3px\">{label}</div></div>"
            )
        };
        body.push_str(&sh(&tr("impact.heading"), None));
        body.push_str(&format!(
            "<p style=\"font-size:12px;color:{MID};margin:-2px 0 10px;line-height:1.5\">{}</p>",
            esc(&tr("impact.intro"))
        ));
        body.push_str(&format!(
            "<div style=\"display:flex;gap:8px;flex-wrap:wrap;margin-bottom:16px\">{}{}{}</div>",
            stat(&money, &tr("impact.raised")),
            stat(&im.causes.unwrap_or(0).to_string(), &tr("impact.causes")),
            stat(&im.pledges.unwrap_or(0).to_string(), &tr("impact.pledges")),
        ));
    }
    if !lineages.is_empty() {
        body.push_str(&format!(
            "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.08em;\
               color:{BROWN};margin:4px 0 8px\">{}</div>",
            esc(&tr("impact.sash"))
        ));
        for l in &lineages {
            let cause = esc(l.cause.as_deref().unwrap_or(""));
            let comp = esc(l.competition.as_deref().unwrap_or(""));
            let names = esc(l.names.as_deref().unwrap_or(""));
            let people = l.people.unwrap_or(0);
            let span = match (l.first_year, l.last_year) {
                (Some(a), Some(b)) if a != b => format!("{a}\u{2013}{b}"),
                (Some(a), _) => a.to_string(),
                _ => String::new(),
            };
            // Dollars raised for this cause, where a titleholder total is recorded.
            let raised = match l.raised {
                Some(r) if r > 0 => format!(
                    " \u{00b7} <span style=\"color:{GOLD};font-weight:700\">${r} {w}</span>",
                    w = tr("impact.cause_raised")
                ),
                _ => String::new(),
            };
            let th_word = tr("impact.titleholders");
            body.push_str(&card(&format!(
                "<div><div style=\"font-weight:600;font-size:14px;color:{BROWN};line-height:1.3\">{cause}</div>\
                 <div style=\"font-size:11px;color:{MID};margin-top:2px\">{comp} \u{00b7} {people} {th_word} \u{00b7} {span}{raised}</div>\
                 <div style=\"font-size:12px;color:{DARK};margin-top:6px;line-height:1.5\">{names}</div></div>"
            )));
        }
    }

    Html(shell(
        "Campaigns",
        "Bear community campaigns by cause \u{2014} give now where you can.",
        "now",
        &body,
        lang,
    ))
    .into_response()
}
