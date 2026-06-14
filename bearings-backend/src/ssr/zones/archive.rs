//! Zone: archive — almanac layout: a "this month in history" hero over a single
//! decade-chunked timeline. Voices (oral histories / scholarship) attach to the
//! milestone they speak about, behind an expandable button. Title-holder lineage
//! lives in the Titles zone, not here.

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, i18n, ui::*};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use chrono::{Datelike, Utc};
use std::collections::HashMap;

const VOICES_CSS: &str = "<style>.vx{margin-top:8px}.vx>summary{list-style:none;cursor:pointer;display:inline-flex;align-items:center;font-size:11px;color:#993C1D;background:#fff;border:1px solid #D2691E;border-radius:20px;padding:3px 11px}.vx>summary::-webkit-details-marker{display:none}.vx>summary::after{content:\"\\25BE\";margin-left:5px}.vx[open]>summary::after{content:\"\\25B4\"}.vx[open]>summary{background:#FBF0E0}</style>";

pub(crate) async fn zone_archive(
    db: SupabaseClient,
    _decade: Option<String>,
    _fragment: Option<String>,
    lang: &str,
) -> Response {
    let url = format!(
        "{}/rest/v1/bear_history?active=eq.true\
         &select=id,year,month,title,description,category,significance,link,featured\
         &order=year.asc&limit=200",
        db.url
    );
    let stories_url = format!(
        "{}/rest/v1/stories?active=eq.true&privacy_mode=eq.false\
         &select=title,story_type,year,excerpt,bear_history_id,link\
         &order=year.desc&limit=100",
        db.url
    );

    let closed_places_url = format!(
        "{}/rest/v1/places?active=eq.true&closed_year=not.is.null\
         &select=id,name,city,country,place_type,closed_year,revival_votes\
         &order=closed_year.desc.nullslast&limit=80",
        db.url
    );
    let closed_clubs_url = format!(
        "{}/rest/v1/clubs?active=eq.true&closed_year=not.is.null\
         &select=id,name,city,country,club_type,closed_year,revival_votes\
         &order=closed_year.desc.nullslast&limit=80",
        db.url
    );

    let (history_res, stories_res, cplaces_res, cclubs_res) = tokio::join!(
        db.get_json::<Vec<BearHistoryRow>>(&url),
        db.get_json::<Vec<CommunityStoryRow>>(&stories_url),
        db.get_json::<Vec<ClosedVenueRow>>(&closed_places_url),
        db.get_json::<Vec<ClosedVenueRow>>(&closed_clubs_url),
    );

    let history = match history_res {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };
    let all_stories: Vec<CommunityStoryRow> = stories_res.or_log("archive:stories_res");
    let closed_places: Vec<ClosedVenueRow> = cplaces_res.or_log("archive:closed_places");
    let closed_clubs: Vec<ClosedVenueRow> = cclubs_res.or_log("archive:closed_clubs");

    // Voices grouped by the milestone they speak about.
    let mut voices: HashMap<i64, Vec<&CommunityStoryRow>> = HashMap::new();
    for s in &all_stories {
        if let Some(hid) = s.bear_history_id {
            voices.entry(hid).or_default().push(s);
        }
    }

    // ── "This month in history" hero ────────────────────────────
    let now = Utc::now();
    let cur_month = now.month() as i32;
    let cur_year = now.year() as i64;
    let month_hero = history
        .iter()
        .filter(|h| h.month == Some(cur_month))
        .max_by_key(|h| (h.featured.unwrap_or(false), h.year.unwrap_or(0)));
    let (kicker, hero_entry) = match month_hero {
        Some(h) => (
            format!("This month in history &middot; {}", month_name(cur_month)),
            Some(h),
        ),
        None => {
            let feat = history
                .iter()
                .filter(|h| h.featured.unwrap_or(false))
                .max_by_key(|h| h.year.unwrap_or(0))
                .or_else(|| history.iter().max_by_key(|h| h.year.unwrap_or(0)));
            ("From the archive".to_string(), feat)
        }
    };
    let hero = hero_entry
        .map(|h| build_hero(h, &kicker, cur_year))
        .unwrap_or_default();

    // ── Single decade-chunked timeline ──────────────────────────
    let decades: [(&str, i64); 5] = [
        ("1980s", 1980),
        ("1990s", 1990),
        ("2000s", 2000),
        ("2010s", 2010),
        ("2020s", 2020),
    ];

    let chips: String = decades
        .iter()
        .filter(|(_, start)| {
            history.iter().any(|h| {
                h.year
                    .map(|y| (y as i64) >= *start && (y as i64) < *start + 10)
                    .unwrap_or(false)
            })
        })
        .map(|(label, _)| {
            format!(
                "<a href=\"#dec-{label}\" style=\"font-size:12px;color:{BROWN};background:{OFF_WHITE};\
                   border:1px solid {TAN};border-radius:14px;padding:4px 11px;text-decoration:none\">{label}</a>"
            )
        })
        .collect();

    let mut tl = String::new();
    for (label, start) in decades.iter() {
        let entries: Vec<&BearHistoryRow> = history
            .iter()
            .filter(|h| {
                h.year
                    .map(|y| (y as i64) >= *start && (y as i64) < *start + 10)
                    .unwrap_or(false)
            })
            .collect();
        if entries.is_empty() {
            continue;
        }
        tl.push_str(&format!(
            "<div id=\"dec-{label}\" style=\"margin-top:24px\">\
               <div style=\"display:inline-block;background:{BROWN};color:{OFF_WHITE};font-size:12px;\
                    font-weight:600;letter-spacing:.04em;border-radius:8px;padding:3px 12px;margin-bottom:9px\">{label}</div>\
               <div style=\"background:{OFF_WHITE};border-left:4px solid {ORANGE};padding:10px 13px;\
                    border-radius:0 6px 6px 0;font-size:12px;color:{MID};line-height:1.65;margin-bottom:16px\">{ctx}</div>\
               {nodes}\
             </div>",
            ctx = decade_context(label),
            nodes = build_timeline(&entries, &voices),
        ));
    }

    let memorial = build_memorial(&closed_places, &closed_clubs);
    let page_archive_title = i18n::t(i18n::translations(), lang, "page.history.title");
    let body = format!(
        "{VOICES_CSS}\
        <h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">{page_archive_title}</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          Community memory from 1987 to present &mdash; {n} milestones documented.</p>\
        {hero}\
        <div style=\"display:flex;gap:6px;flex-wrap:wrap;margin-bottom:4px\">{chips}</div>\
        {tl}\
        {memorial}\
        <div class=\"card\" style=\"margin-top:24px\">\
          <div style=\"font-weight:600;font-size:14px;margin-bottom:4px\">Title holders</div>\
          <div style=\"font-size:12px;color:{MID};margin-bottom:6px\">The full lineage of every competition lives in its own zone.</div>\
          <a href=\"/?zone=titles\" style=\"font-size:12px;color:{ORANGE}\">View the Titles zone &rarr;</a>\
        </div>",
        n = history.len(),
    );
    Html(shell(
        "Bear Archives",
        "Community history 1987 to present.",
        "archive",
        &body,
        lang,
    ))
    .into_response()
}

fn month_name(m: i32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

fn decade_context(active: &str) -> &'static str {
    match active {
        "1980s" => "The bear community emerged from leather bars in San Francisco and New York as a deliberate \
                    counter to mainstream gay culture's body standards. The AIDS crisis both devastated and \
                    galvanised — fundraising events, mutual aid networks, and the early bear clubs became \
                    community lifelines. The International Bear Rendezvous held its first gathering in 1987.",
        "1990s" => "IBR peaked with 5,000+ attendees. Regional bear runs spread across North America and into \
                    Europe. The community debated its own boundaries — who counted as a bear, whether leather \
                    and bear culture were separating, and how to document an oral tradition before it was lost. \
                    The first bear title competitions formalised community leadership.",
        "2000s" => "The internet reshaped everything. Online communities let isolated bears connect without \
                    geography. Bear-specific social platforms launched and merged. Pride events globally began \
                    programming dedicated bear spaces. Title competitions expanded to national and continental \
                    levels, and European clubs found their own voice distinct from the North American model.",
        "2010s" => "Social media brought visibility and new friction — global audiences, broader inclusion \
                    debates, and algorithmic culture clash with community-organised space. Bear Run attendance \
                    diversified in body type, kink background, and nationality. Scholarship and oral history \
                    projects launched to archive first-generation memories. COVID cast a shadow at decade's end.",
        _ => "The pandemic years forced a pivot to virtual events, then a hungry return to in-person \
                    gatherings. Inclusion of trans and non-binary community members became an explicit priority. \
                    New title competitions launched in Latin America, Africa, and Southeast Asia. Digital \
                    infrastructure — iCal feeds, community directories, and shared databases — became \
                    maintenance priorities as founding-generation stewards aged.",
    }
}

fn build_hero(h: &BearHistoryRow, kicker: &str, cur_year: i64) -> String {
    let year = h.year.unwrap_or(0) as i64;
    let title = esc(h.title.as_str());
    let body_txt = esc(h
        .significance
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(h.description.as_deref())
        .unwrap_or(""));
    let cat = h.category.as_deref().unwrap_or("");
    let ago = if year > 0 && cur_year > year {
        format!(" &middot; {} years ago", cur_year - year)
    } else {
        String::new()
    };
    let band = if cat == "flag-symbol" {
        "<div style=\"height:54px;display:flex;flex-direction:column\">\
           <div style=\"flex:1;background:#5b3a1a\"></div><div style=\"flex:1;background:#b86a2b\"></div>\
           <div style=\"flex:1;background:#e0a82e\"></div><div style=\"flex:1;background:#ead7a8\"></div>\
           <div style=\"flex:1;background:#f5f0e7\"></div><div style=\"flex:1;background:#5a5a5a\"></div>\
           <div style=\"flex:1;background:#161616\"></div></div>"
            .to_string()
    } else {
        format!("<div style=\"height:6px;background:{ORANGE}\"></div>")
    };
    let link = esc(h.link.as_deref().unwrap_or(""));
    let src = if !link.is_empty() && link != "#" {
        format!(
            "<div style=\"margin-top:10px\"><a href=\"{link}\" target=\"_blank\" rel=\"noopener\" \
               style=\"font-size:11px;color:{ORANGE};border:1px solid {ORANGE};border-radius:12px;\
                       padding:2px 10px;text-decoration:none\">source &#8599;</a></div>"
        )
    } else {
        String::new()
    };
    format!(
        "<div class=\"card\" style=\"padding:0;overflow:hidden;margin-bottom:18px\">{band}\
           <div style=\"padding:13px 15px\">\
             <div style=\"font-size:11px;letter-spacing:.12em;text-transform:uppercase;color:{ORANGE};margin-bottom:7px\">{kicker}</div>\
             <div style=\"font-family:Georgia,serif;font-size:18px;font-weight:600;line-height:1.25;color:{BROWN};margin-bottom:5px\">{title}</div>\
             <div style=\"font-size:12px;color:{MID};margin-bottom:9px\">{year}{ago}</div>\
             <div style=\"font-size:13px;color:{DARK};line-height:1.6\">{body_txt}</div>\
             {src}\
           </div>\
         </div>"
    )
}

pub(crate) fn build_timeline(
    entries: &[&BearHistoryRow],
    voices: &HashMap<i64, Vec<&CommunityStoryRow>>,
) -> String {
    if entries.is_empty() {
        return format!(
            "<div style=\"text-align:center;color:{MID};font-size:13px;padding:24px 0\">\
              No records for this decade yet.</div>"
        );
    }
    entries.iter().map(|h| {
        let year  = h.year.unwrap_or(0) as i64;
        let title = esc(h.title.as_str());
        let desc  = esc(h.description.as_deref().unwrap_or(""));
        let sig   = esc(h.significance.as_deref().unwrap_or(""));
        let cat   = h.category.as_deref().unwrap_or("milestone");
        let (cat_bg, cat_fg) = match cat {
            "title-competition" => ("#FBF0E0", ORANGE),
            "club-founding"     => ("#EDE0D4", BROWN),
            "milestone"         => ("#EDF5EC", "#4a6741"),
            "technology"        => ("#E8EEF8", "#2c5aa0"),
            "legislation"       => ("#ECE8F5", "#5c3891"),
            "media"             => ("#F5E8F0", "#8c2060"),
            "scholarship"       => ("#E8F2EE", "#2a6b4a"),
            "publication"       => ("#FFF8E0", "#7a6000"),
            "cultural-moment"   => ("#F8ECE8", "#8c3020"),
            "tragedy"           => ("#F0E8E8", "#8c2020"),
            "fundraising"       => ("#E8F0F8", "#1c4f8c"),
            "flag-symbol"       => ("#FBF0E0", "#9c4000"),
            "archive"           => ("#EEF0E8", "#4a5a30"),
            _                   => ("#F0F0F0",  MID),
        };
        let cat_label = cat.replace("-", " ");
        let sig_html = if !sig.is_empty() {
            format!(
                "<div style=\"font-size:11px;font-style:italic;color:{BROWN};\
                    border-left:3px solid {ORANGE};padding-left:8px;\
                    margin-top:6px;line-height:1.6\">{sig}</div>"
            )
        } else { String::new() };

        let link     = esc(h.link.as_deref().unwrap_or(""));
        let src_html = if !link.is_empty() && link != "#" {
            format!(
                "<div style=\"margin-top:5px\"><a href=\"{link}\" target=\"_blank\" rel=\"noopener\" \
                     style=\"font-size:10px;color:{ORANGE};text-decoration:none;\
                            border:1px solid {ORANGE};border-radius:10px;padding:1px 8px\">source &#8599;</a></div>"
            )
        } else { String::new() };

        let vx_html = match h.id.and_then(|id| voices.get(&id)) {
            Some(vs) if !vs.is_empty() => {
                let cards: String = vs.iter().map(|s| build_voice_card(s)).collect();
                let n = vs.len();
                let word = if n == 1 { "voice" } else { "voices" };
                format!(
                    "<details class=\"vx\"><summary>{n} {word}</summary>\
                       <div style=\"margin-top:8px;border:1px solid {TAN};border-radius:10px;overflow:hidden;background:#fff\">{cards}</div>\
                     </details>"
                )
            }
            _ => String::new(),
        };

        format!(
            "<div style=\"display:flex;gap:12px;margin-bottom:16px\">\
              <div style=\"display:flex;flex-direction:column;align-items:center\">\
                <div class=\"tl-dot\">{year}</div>\
                <div class=\"tl-line\"></div>\
              </div>\
              <div style=\"padding-top:4px;padding-bottom:12px;flex:1\">\
                <span class=\"cat\" style=\"background:{cat_bg};color:{cat_fg}\">{cat_label}</span>\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3;margin-bottom:6px\">{title}</div>\
                <div style=\"font-size:12px;color:{MID};line-height:1.6\">{desc}</div>\
                {sig_html}{src_html}{vx_html}\
              </div>\
            </div>"
        )
    }).collect()
}

fn build_voice_card(s: &CommunityStoryRow) -> String {
    let stitle = esc(s.title.as_deref().unwrap_or(""));
    let stype = s.story_type.as_deref().unwrap_or("story");
    let exc = esc(s.excerpt.as_deref().unwrap_or(""));
    let type_label = match stype {
        "interview" => "Interview",
        "first-person" => "First person",
        "scholarship" => "Scholarship",
        "institutional-history" => "History",
        "oral_history" => "Oral history",
        _ => stype,
    };
    let link = esc(s.link.as_deref().unwrap_or(""));
    let link_html = if !link.is_empty() && link != "#" {
        format!(
            "<div style=\"margin-top:6px\"><a href=\"{link}\" target=\"_blank\" rel=\"noopener\" \
               style=\"font-size:11px;color:{ORANGE};text-decoration:none\">Read &rarr;</a></div>"
        )
    } else {
        String::new()
    };
    let exc_html = if !exc.is_empty() {
        format!(
            "<div style=\"font-size:12px;color:{MID};line-height:1.55;font-style:italic;margin-top:5px\">\
              &ldquo;{}&rdquo;</div>",
            exc.chars().take(240).collect::<String>()
        )
    } else {
        String::new()
    };
    format!(
        "<div style=\"padding:10px 11px;border-top:1px solid {OFF_WHITE}\">\
           <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:8px\">\
             <span style=\"font-size:12px;font-weight:600;color:{BROWN};flex:1\">{stitle}</span>\
             <span style=\"font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:.06em;\
                  color:{BROWN};background:{OFF_WHITE};border:1px solid {TAN};border-radius:12px;\
                  padding:2px 8px;flex-shrink:0\">{type_label}</span>\
           </div>\
           {exc_html}{link_html}\
         </div>"
    )
}

fn build_memorial(places: &[ClosedVenueRow], clubs: &[ClosedVenueRow]) -> String {
    if places.is_empty() && clubs.is_empty() {
        return String::new();
    }
    let card = |v: &ClosedVenueRow, kind: &str| -> String {
        let name = esc(&v.name);
        let city = esc(v.city.as_deref().unwrap_or(""));
        let ctry = esc(v.country.as_deref().unwrap_or(""));
        let loc = match (city.is_empty(), ctry.is_empty()) {
            (false, false) => format!("{city}, {ctry}"),
            (false, true) => city,
            (true, false) => ctry,
            _ => String::new(),
        };
        let typ = v
            .kind_type
            .as_deref()
            .map(|t| esc(&t.replace('-', " ")))
            .unwrap_or_default();
        let meta = if typ.is_empty() {
            loc
        } else if loc.is_empty() {
            typ
        } else {
            format!("{loc} &middot; {typ}")
        };
        let closed = match v.closed_year {
            Some(y) => format!("closed {y}"),
            None => "closed".to_string(),
        };
        let id = v.id.unwrap_or(0);
        let votes = v.revival_votes.unwrap_or(0);
        format!(
            "<div style=\"background:#efe9dd;border:1px solid #ddd1bd;border-radius:11px;\
                  padding:10px 12px;margin-bottom:9px\">\
              <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:8px\">\
                <div>\
                  <div style=\"font-size:14px;font-weight:600;color:#6f6253\">{name}</div>\
                  <div style=\"font-size:11px;color:#9a8c79;margin-top:2px\">{meta}</div>\
                </div>\
                <span style=\"font-size:10px;color:#8a6a4a;background:#e6d9c4;border-radius:6px;\
                      padding:1px 7px;white-space:nowrap\">{closed}</span>\
              </div>\
              <div style=\"margin-top:9px;padding-top:8px;border-top:1px solid #e0d4c0;\
                    display:flex;justify-content:flex-end\">\
                <button hx-post=\"/api/revival/{kind}/{id}\" hx-swap=\"outerHTML\" hx-target=\"this\" \
                  style=\"font-size:11px;color:#9c4000;background:#fff;border:1px solid {ORANGE};\
                         border-radius:20px;padding:3px 11px;cursor:pointer\">\u{25B2} {votes} would return</button>\
              </div>\
            </div>"
        )
    };
    let cards: String = places
        .iter()
        .map(|v| card(v, "place"))
        .chain(clubs.iter().map(|v| card(v, "club")))
        .collect();
    format!(
        "<div style=\"margin-top:26px\">\
          <div style=\"font-family:Georgia,serif;font-size:16px;font-weight:600;color:#6f6253;\
                margin-bottom:3px\">Gone but not forgotten</div>\
          <div style=\"font-size:12px;color:{MID};margin-bottom:12px\">\
            Places the scene remembers. Tap &lsquo;would return&rsquo; to signal a revival.</div>\
          {cards}\
        </div>"
    )
}
