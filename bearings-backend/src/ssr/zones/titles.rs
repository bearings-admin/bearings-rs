//! Zone: titles

use axum::response::{Html, IntoResponse, Response};
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;
use super::super::query::*;

pub(crate) async fn zone_titles(db: SupabaseClient, lang: &str) -> Response {
    // Fetch competitions with scope/country info
    let url_comps = format!(
        "{}/rest/v1/competitions\
         ?active=eq.true\
         &select=id,name,scope,country,city,website,founded_year,owning_club_id\
         &order=scope.asc,name.asc&limit=100",
        db.url
    );
    // Fetch all title holders (historical + current)
    let url_holders = format!(
        "{}/rest/v1/title_holders\
         ?select=competition_id,holder_name,year,city,country,inclusion_flag_codes,holder_status\
         &order=competition_id.asc,year.desc&limit=500",
        db.url
    );
    // Fetch clubs for linking
    let url_clubs = format!(
        "{}/rest/v1/clubs?select=id,name,website&active=eq.true&limit=200",
        db.url
    );
    let (comps_res, holders_res, clubs_res) = tokio::join!(
        db.get_json::<Vec<CompetitionRow>>(&url_comps),
        db.get_json::<Vec<TitleHolderRow>>(&url_holders),
        db.get_json::<Vec<ClubRow>>(&url_clubs),
    );
    let comps: Vec<CompetitionRow> = comps_res.or_log("titles:comps_res");
    let holders: Vec<TitleHolderRow> = holders_res.or_log("titles:holders_res");
    let clubs: Vec<ClubRow> = clubs_res.or_log("titles:clubs_res");

    // Index clubs by id
    let club_map: std::collections::HashMap<i64, (String, String)> = clubs.iter()
        .filter_map(|c| {
            let id   = c.id?;
            let name = esc(c.name.as_str());
            let site = esc(c.website.as_deref().unwrap_or(""));
            Some((id, (name, site)))
        })
        .collect();

    // Group holders by competition_id
    let mut holders_by_comp: std::collections::HashMap<i64, Vec<&TitleHolderRow>> = std::collections::HashMap::new();
    for h in &holders {
        if let Some(cid) = h.competition_id {
            holders_by_comp.entry(cid).or_default().push(h);
        }
    }

    // Scope order and icons
    let scope_order = ["international", "continental", "national", "regional", "local"];
    let scope_icon  = |s: &str| match s {
        "international" => "🌍", "continental" => "🌎",
        "national" => "🏳️",    "regional"      => "📍",
        "local"    => "🏙️",    _               => "🐻",
    };

    // Build sections by scope
    let mut sections = String::new();
    for scope in scope_order {
        let scope_comps: Vec<&CompetitionRow> = comps.iter()
            .filter(|c| c.scope.as_deref().unwrap_or("") == scope)
            .collect();
        if scope_comps.is_empty() { continue; }

        let scope_label = match scope {
            "international" => "International",
            "continental"   => "Continental",
            "national"      => "National",
            "regional"      => "Regional",
            "local"         => "Local",
            _               => scope,
        };
        sections.push_str(&format!(
            "<div style=\"font-size:10px;font-weight:700;text-transform:uppercase;\
              letter-spacing:.1em;color:{MID};margin:16px 0 6px\">{scope_label}</div>"
        ));

        for comp in scope_comps {
            let _cid     = comp.id;
            let cname   = esc(comp.name.as_str());
            let ccountry = esc(comp.country.as_deref().unwrap_or(""));
            let ccity   = esc(comp.city.as_deref().unwrap_or(""));
            let csite   = esc(comp.website.as_deref().unwrap_or("#"));
            let cfounded = comp.founded_year.unwrap_or(0) as i64;
            let club_id = comp.owning_club_id.unwrap_or(0);
            let icon    = scope_icon(scope);

            let site_btn = if !csite.is_empty() && csite != "#" {
                format!("<a href=\"{csite}\" target=\"_blank\" rel=\"noopener\" \
                          style=\"font-size:10px;color:{ORANGE};text-decoration:none;\
                                  border:1px solid {ORANGE};border-radius:8px;\
                                  padding:2px 8px;white-space:nowrap\">Site</a>")
            } else { String::new() };

            let club_btn = if club_id > 0 {
                if let Some((cln, cls)) = club_map.get(&club_id) {
                    if !cls.is_empty() && *cls != "#" {
                        format!("<a href=\"{cls}\" target=\"_blank\" rel=\"noopener\" \
                                  style=\"font-size:10px;color:{BROWN};text-decoration:none;\
                                          border:1px solid {TAN};border-radius:8px;\
                                          padding:2px 8px;white-space:nowrap\">{cln}</a>")
                    } else {
                        format!("<span style=\"font-size:10px;color:{MID}\">{cln}</span>")
                    }
                } else { String::new() }
            } else { String::new() };

            let meta = {
                let mut parts = vec![];
                if !ccity.is_empty() { parts.push(ccity.to_string()); }
                if !ccountry.is_empty() { parts.push(ccountry.to_string()); }
                if cfounded > 0 { parts.push(format!("est. {cfounded}")); }
                parts.join(" · ")
            };

            // Titleholders sublist
            let comp_holders = holders_by_comp.get(&comp.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let holder_rows: String = comp_holders.iter().take(12).map(|h| {
                let name   = esc(h.holder_name.as_str());
                let year   = h.year.unwrap_or(0) as i64;
                let hcity  = esc(h.city.as_deref().unwrap_or(""));
                let hctry  = esc(h.country.as_deref().unwrap_or(""));
                let status = h.holder_status.as_deref().unwrap_or("");
                let fs: Vec<String> = h.inclusion_flag_codes.clone().unwrap_or_default();
                let loc = match (hcity, hctry) {
                    (c, ct) if !c.is_empty() && !ct.is_empty() => format!("{c}, {ct}"),
                    (c, _) if !c.is_empty() => c.to_string(),
                    (_, ct) if !ct.is_empty() => ct.to_string(),
                    _ => String::new(),
                };
                let status_badge = match status {
                    "current"  => format!("<span style=\"font-size:9px;background:{ORANGE};\
                                            color:#fff;border-radius:6px;padding:1px 5px\">current</span> "),
                    "holdover" => format!("<span style=\"font-size:9px;background:{GOLD};\
                                            color:{DARK};border-radius:6px;padding:1px 5px\">holdover</span> "),
                    _ => String::new(),
                };
                format!(
                    "<div style=\"display:flex;justify-content:space-between;\
                                  align-items:center;padding:5px 0;\
                                  border-bottom:1px solid {OFF_WHITE}\">\
                      <div>\
                        <span style=\"font-size:13px;font-weight:500\">{status_badge}{name}</span>\
                        {loc_h}\
                        {fhtml}\
                      </div>\
                      <div style=\"font-size:16px;font-weight:700;color:{TAN};flex-shrink:0\">{yr}</div>\
                    </div>",
                    loc_h = if !loc.is_empty() {
                        format!("<div style=\"font-size:11px;color:{MID}\">{loc}</div>")
                    } else { String::new() },
                    fhtml = if !fs.is_empty() {
                        format!("<div style=\"margin-top:2px\">{}</div>", flags(&fs))
                    } else { String::new() },
                    yr = if year > 0 { year.to_string() } else { String::new() },
                )
            }).collect();

            let more_note = if comp_holders.len() > 12 {
                format!("<div style=\"font-size:11px;color:{MID};padding:4px 0\">+ <a href=\'/?zone=archive\' style=\'color:{ORANGE}\'>{} more in archive →</a></div>",
                    comp_holders.len() - 12)
            } else { String::new() };

            sections.push_str(&card(&format!(
                "<div>\
                  <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:8px\">\
                    <div style=\"flex:1\">\
                      <div style=\"font-weight:700;font-size:15px\">{icon} {cname}</div>\
                      <div style=\"font-size:11px;color:{MID};margin-top:2px\">{meta}</div>\
                    </div>\
                    <div style=\"display:flex;flex-direction:column;gap:4px;align-items:flex-end\">\
                      {site_btn}{club_btn}\
                    </div>\
                  </div>\
                  {holders_h}\
                </div>",
                holders_h = if !holder_rows.is_empty() {
                    format!("<div style=\"margin-top:10px\">{holder_rows}{more_note}</div>")
                } else {
                    format!("<div style=\"font-size:11px;color:{MID};margin-top:6px;\
                              font-style:italic\">No recorded titleholders yet.</div>")
                },
            )));
        }
    }

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bear Title Holders</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:8px\">\
          Competitions and their titleholders. IBR complete 1992–2011.</p>\
        {sections}"
    );
    Html(shell("Titles", "Bear title holders worldwide.", "archive", &body, lang)).into_response()
}


