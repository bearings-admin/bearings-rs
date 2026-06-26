//! Zone: titles

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, ui::*};
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

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
         ?select=competition_id,holder_name,year,city,country,inclusion_flag_codes,holder_status,charity_name,charity_link\
         &order=competition_id.asc,year.desc&limit=500",
        db.url
    );
    // Fetch clubs for linking
    let url_clubs = format!(
        "{}/rest/v1/clubs?select=id,name,website&active=eq.true&limit=200",
        db.url
    );
    let url_artifacts = format!(
        "{}/rest/v1/artifacts?active=eq.true&entity_type=eq.competition\
         &select=id,entity_id,kind,title,description,transcription,contributor,provenance,captured_on,image_url\
         &order=captured_on.desc.nullslast&limit=200",
        db.url
    );
    let (comps_res, holders_res, clubs_res, arts_res) = tokio::join!(
        db.get_json::<Vec<CompetitionRow>>(&url_comps),
        db.get_json::<Vec<TitleHolderRow>>(&url_holders),
        db.get_json::<Vec<ClubRow>>(&url_clubs),
        db.get_json::<Vec<ArtifactRow>>(&url_artifacts),
    );
    let comps: Vec<CompetitionRow> = comps_res.or_log("titles:comps_res");
    let holders: Vec<TitleHolderRow> = holders_res.or_log("titles:holders_res");
    let clubs: Vec<ClubRow> = clubs_res.or_log("titles:clubs_res");
    let artifacts: Vec<ArtifactRow> = arts_res.or_log("titles:arts_res");
    let mut artifacts_by_comp: std::collections::HashMap<i64, Vec<ArtifactRow>> =
        std::collections::HashMap::new();
    for a in artifacts {
        if let Some(eid) = a.entity_id {
            artifacts_by_comp.entry(eid).or_default().push(a);
        }
    }

    // Index clubs by id
    let club_map: std::collections::HashMap<i64, (String, String)> = clubs
        .iter()
        .filter_map(|c| {
            let id = c.id?;
            let name = esc(c.name.as_str());
            let site = esc(c.website.as_deref().unwrap_or(""));
            Some((id, (name, site)))
        })
        .collect();

    // Group holders by competition_id
    let mut holders_by_comp: std::collections::HashMap<i64, Vec<&TitleHolderRow>> =
        std::collections::HashMap::new();
    for h in &holders {
        if let Some(cid) = h.competition_id {
            holders_by_comp.entry(cid).or_default().push(h);
        }
    }

    let tl = |k: &str| crate::i18n::t(crate::i18n::translations(), lang, k);

    // Scope order and icons
    let scope_order = [
        "international",
        "continental",
        "national",
        "regional",
        "local",
    ];
    let scope_icon = |s: &str| match s {
        "international" => "🌍",
        "continental" => "🌎",
        "national" => "🏳️",
        "regional" => "📍",
        "local" => "🏙️",
        _ => "🐻",
    };

    // Build sections by scope
    let mut sections = String::new();
    for scope in scope_order {
        let scope_comps: Vec<&CompetitionRow> = comps
            .iter()
            .filter(|c| c.scope.as_deref().unwrap_or("") == scope)
            .collect();
        if scope_comps.is_empty() {
            continue;
        }

        let scope_label = tl(match scope {
            "international" => "titles.scope.international",
            "continental" => "titles.scope.continental",
            "national" => "titles.scope.national",
            "regional" => "titles.scope.regional",
            "local" => "titles.scope.local",
            _ => scope,
        });
        sections.push_str(&format!(
            "<div style=\"font-size:10px;font-weight:700;text-transform:uppercase;\
              letter-spacing:.1em;color:{MID};margin:16px 0 6px\">{scope_label}</div>"
        ));

        for comp in scope_comps {
            let _cid = comp.id;
            let cname = esc(comp.name.as_str());
            let ccountry = esc(comp.country.as_deref().unwrap_or(""));
            let ccity = esc(comp.city.as_deref().unwrap_or(""));
            let csite = esc(comp.website.as_deref().unwrap_or("#"));
            let cfounded = comp.founded_year.unwrap_or(0) as i64;
            let club_id = comp.owning_club_id.unwrap_or(0);
            let icon = scope_icon(scope);

            let site_btn = if !csite.is_empty() && csite != "#" {
                format!(
                    "<a href=\"{csite}\" target=\"_blank\" rel=\"noopener\" \
                          style=\"font-size:10px;color:{ORANGE};text-decoration:none;\
                                  border:1px solid {ORANGE};border-radius:8px;\
                                  padding:2px 8px;white-space:nowrap\">Site</a>"
                )
            } else {
                String::new()
            };

            let club_btn = if club_id > 0 {
                if let Some((cln, cls)) = club_map.get(&club_id) {
                    if !cls.is_empty() && *cls != "#" {
                        format!(
                            "<a href=\"{cls}\" target=\"_blank\" rel=\"noopener\" \
                                  style=\"font-size:10px;color:{BROWN};text-decoration:none;\
                                          border:1px solid {TAN};border-radius:8px;\
                                          padding:2px 8px;white-space:nowrap\">{cln}</a>"
                        )
                    } else {
                        format!("<span style=\"font-size:10px;color:{MID}\">{cln}</span>")
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let meta = {
                let mut parts = vec![];
                if !ccity.is_empty() {
                    parts.push(ccity.to_string());
                }
                if !ccountry.is_empty() {
                    parts.push(ccountry.to_string());
                }
                if cfounded > 0 {
                    parts.push(format!("est. {cfounded}"));
                }
                parts.join(" · ")
            };

            // Titleholders sublist
            let comp_holders = holders_by_comp
                .get(&comp.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let render_row = |h: &&TitleHolderRow| -> String {
                let name = esc(h.holder_name.as_str());
                let year = h.year.unwrap_or(0) as i64;
                let hcity = esc(h.city.as_deref().unwrap_or(""));
                let hctry = esc(h.country.as_deref().unwrap_or(""));
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
            };

            // First 12 inline; collapse the rest behind a pure-CSS "see more"
            // toggle that expands the remaining rows in place (no redirect).
            const VISIBLE: usize = 12;
            let holder_rows: String = comp_holders.iter().take(VISIBLE).map(&render_row).collect();
            let hidden_rows: String = comp_holders.iter().skip(VISIBLE).map(render_row).collect();
            let overflow = comp_holders.len().saturating_sub(VISIBLE);

            let more_note = if overflow > 0 {
                format!(
                    "<input type=\"checkbox\" id=\"thmore-{cid}\" class=\"th-toggle\">\
                     <div class=\"th-extra\">{hidden_rows}</div>\
                     <label for=\"thmore-{cid}\" class=\"th-more\">+ {overflow} more \u{25BE}</label>",
                    cid = comp.id
                )
            } else {
                String::new()
            };

            let charity_h = comp_holders.iter().find_map(|h| {
                let n = h.charity_name.as_deref().filter(|s| !s.is_empty())?;
                let label = esc(n);
                let link = esc(h.charity_link.as_deref().unwrap_or(""));
                Some(if !link.is_empty() && link != "#" {
                    format!("<div style=\"font-size:11px;color:{BROWN};margin-top:4px\">\u{1f49a} Supports <a href=\"{link}\" target=\"_blank\" rel=\"noopener\" style=\"color:{BROWN}\">{label}</a></div>")
                } else {
                    format!("<div style=\"font-size:11px;color:{BROWN};margin-top:4px\">\u{1f49a} Supports {label}</div>")
                })
            }).unwrap_or_default();

            let artifact_h = build_artifacts(
                artifacts_by_comp
                    .get(&comp.id)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]),
                lang,
            );

            sections.push_str(&card(&format!(
                "<div>\
                  <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:8px\">\
                    <div style=\"flex:1\">\
                      <div style=\"font-weight:700;font-size:15px\">{icon} {cname}</div>\
                      <div style=\"font-size:11px;color:{MID};margin-top:2px\">{meta}</div>{charity_h}\
                    </div>\
                    <div style=\"display:flex;flex-direction:column;gap:4px;align-items:flex-end\">\
                      {site_btn}{club_btn}\
                    </div>\
                  </div>\
                  {holders_h}\
                  {artifact_h}\
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
    Html(shell(
        "Titles",
        "Bear title holders worldwide.",
        "archive",
        &body,
        lang,
    ))
    .into_response()
}

/// Render artifact source-badges (pure-CSS expanders) for a competition card.
fn build_artifacts(arts: &[ArtifactRow], lang: &str) -> String {
    let src = crate::i18n::t(crate::i18n::translations(), lang, "artifact.source");
    if arts.is_empty() {
        return String::new();
    }
    arts.iter()
        .map(|a| {
            let id = a.id;
            let title = esc(&a.title);
            let kind = esc(&a.kind.as_deref().unwrap_or("artifact").replace('-', " "));
            let prov = esc(a.provenance.as_deref().unwrap_or(""));
            let trans = esc(a.transcription.as_deref().unwrap_or(""));
            let captured = esc(a.captured_on.as_deref().unwrap_or(""));
            let media = match a.image_url.as_deref() {
                Some(u) if !u.is_empty() => {
                    let ue = esc(u);
                    format!(
                        "<img src=\"{ue}\" alt=\"{title}\" style=\"max-width:100%;\
                           border-radius:6px;margin:6px 0\"/>\
                         <div><a href=\"{ue}\" target=\"_blank\" rel=\"noopener\" download \
                           style=\"font-size:11px;color:{ORANGE}\">\u{2193} View / download image</a></div>"
                    )
                }
                _ => format!(
                    "<div style=\"font-size:11px;color:{MID};font-style:italic\">\
                       Image on file with the steward \u{2014} not yet published.</div>"
                ),
            };
            let cap = if captured.is_empty() {
                String::new()
            } else {
                format!(" \u{00b7} {captured}")
            };
            let prov_block = if prov.is_empty() {
                String::new()
            } else {
                format!("<div style=\"font-size:11px;color:{BROWN};margin-top:6px\">{prov}</div>")
            };
            let trans_block = if trans.is_empty() {
                String::new()
            } else {
                format!(
                    "<div style=\"font-size:10px;color:{MID};margin-top:6px;line-height:1.5\">\
                       Transcribed: {trans}</div>"
                )
            };
            format!(
                "<div style=\"margin-top:8px\">\
                   <input type=\"checkbox\" id=\"art-{id}\" class=\"art-chk\">\
                   <label for=\"art-{id}\" class=\"art-badge\">\u{1f4dc} {src}: {title}</label>\
                   <div class=\"art-panel\">\
                     <div style=\"font-size:12px;font-weight:600;color:{BROWN}\">{title}</div>\
                     <div style=\"font-size:11px;color:{MID};margin:2px 0 6px\">{kind}{cap}</div>\
                     {media}{prov_block}{trans_block}\
                   </div>\
                 </div>"
            )
        })
        .collect()
}
