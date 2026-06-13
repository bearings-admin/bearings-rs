//! Zone: archive

use super::super::query::*;
use crate::db::LogErr;
use crate::{db::SupabaseClient, i18n, ui::*};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
#[allow(unused_imports)]
use chrono::{Months, Utc};
#[allow(unused_imports)]
use std::collections::HashMap;

pub(crate) async fn zone_archive(
    db: SupabaseClient,
    decade: Option<String>,
    fragment: Option<String>,
    lang: &str,
) -> Response {
    let url = format!(
        "{}/rest/v1/bear_history?active=eq.true\
         &select=year,title,description,category,significance,link\
         &order=year.asc&limit=100",
        db.url
    );
    let titles_url = format!(
        "{}/rest/v1/title_holders?select=title_name,holder_name,year,city,country\
         &order=year.desc&limit=200",
        db.url
    );
    let stories_url = format!(
        "{}/rest/v1/stories?active=eq.true&privacy_mode=eq.false\
         &select=title,story_type,year,excerpt\
         &order=year.desc&limit=100",
        db.url
    );

    let (history_res, titles_res, stories_res) = tokio::join!(
        db.get_json::<Vec<BearHistoryRow>>(&url),
        db.get_json::<Vec<TitleHolderRow>>(&titles_url),
        db.get_json::<Vec<CommunityStoryRow>>(&stories_url),
    );

    let history = match history_res {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };
    let all_titles: Vec<TitleHolderRow> = titles_res.or_log("archive:titles_res");
    let all_stories: Vec<CommunityStoryRow> = stories_res.or_log("archive:stories_res");

    let decades = ["1980s", "1990s", "2000s", "2010s", "2020s"];
    let active = decade.as_deref().unwrap_or("2020s");
    let d_start: i64 = match active {
        "1980s" => 1980,
        "1990s" => 1990,
        "2000s" => 2000,
        "2010s" => 2010,
        _ => 2020,
    };

    let decade_context: &str = match active {
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
        _       => "The pandemic years forced a pivot to virtual events, then a hungry return to in-person \
                    gatherings. Inclusion of trans and non-binary community members became an explicit priority. \
                    New title competitions launched in Latin America, Africa, and Southeast Asia. Digital \
                    infrastructure — iCal feeds, community directories, and shared databases — became \
                    maintenance priorities as founding-generation stewards aged.",
    };

    // Decade tabs with milestone counts
    let tabs: String = decades.iter().map(|&d| {
        let on = d == active;
        let ds: i64 = match d { "1980s"=>1980,"1990s"=>1990,"2000s"=>2000,"2010s"=>2010,_=>2020 };
        let count = history.iter()
            .filter(|h| h.year.map(|y| y as i64 >= ds && (y as i64) < ds+10).unwrap_or(false))
            .count();
        format!(
            "<a href=\"/?zone=archive&decade={d}&lang={lang}\"\
               hx-get=\"/?zone=archive&decade={d}&fragment=tl&lang={lang}\"\
               hx-target=\"#archive-tl\" hx-swap=\"outerHTML\"\
               hx-indicator=\"#archive-spin\"\
               class=\"dtab {cls}\">{d} <span style=\"font-size:10px;opacity:.7\">({count})</span></a>",
            cls = if on { "dtab-on" } else { "dtab-off" },
        )
    }).collect::<Vec<_>>().join("");

    // Filter everything to active decade
    let decade_entries: Vec<&BearHistoryRow> = history
        .iter()
        .filter(|h| {
            h.year
                .map(|y| y as i64)
                .map(|y| y >= d_start && y < d_start + 10)
                .unwrap_or(false)
        })
        .collect();

    let decade_titles: Vec<&TitleHolderRow> = all_titles
        .iter()
        .filter(|t| {
            t.year
                .map(|y| y as i64)
                .map(|y| y >= d_start && y < d_start + 10)
                .unwrap_or(false)
        })
        .collect();

    let decade_stories: Vec<&CommunityStoryRow> = all_stories
        .iter()
        .filter(|st| {
            st.year
                .map(|y| y as i64)
                .map(|y| y >= d_start && y < d_start + 10)
                .unwrap_or(false)
        })
        .collect();

    let timeline = build_timeline(&decade_entries);

    let title_cards: String = decade_titles
        .iter()
        .map(|t| {
            let title = esc(t.title_name.as_deref().unwrap_or(""));
            let holder = esc(t.holder_name.as_str());
            let year = t.year.unwrap_or(0) as i64;
            let city = esc(t.city.as_deref().unwrap_or(""));
            let ctry = esc(t.country.as_deref().unwrap_or(""));
            card(&format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center\">\
              <div>\
                <div style=\"font-weight:600;font-size:14px\">{title}</div>\
                <div style=\"font-size:12px;color:{MID}\">{holder}</div>\
                <div style=\"font-size:11px;color:{MID}\">{city}{sep}{ctry}</div>\
              </div>\
              <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">{year}</div>\
            </div>",
                sep = if !city.is_empty() && !ctry.is_empty() {
                    ", "
                } else {
                    ""
                },
            ))
        })
        .collect();

    let story_cards: String = decade_stories.iter().map(|st| {
        let stitle   = esc(st.title.as_deref().unwrap_or(""));
        let stype    = st.story_type.as_deref().unwrap_or("story");
        let syear    = st.year.unwrap_or(0) as i64;
        let sexcerpt = esc(st.excerpt.as_deref().unwrap_or(""));
        let type_label = match stype {
            "interview"              => "Interview",
            "first-person"           => "First Person",
            "scholarship"            => "Scholarship",
            "institutional-history"  => "History",
            "oral_history"           => "Oral History",
            _                        => stype,
        };
        card(&format!(
            "<div>\
              <div style=\"display:flex;justify-content:space-between;align-items:flex-start\">\
                <div style=\"font-weight:600;font-size:14px;flex:1;padding-right:8px\">{stitle}</div>\
                <div style=\"font-size:20px;font-weight:700;color:{ORANGE};flex-shrink:0\">{syear}</div>\
              </div>\
              <div style=\"margin:3px 0 6px\">\
                <span style=\"font-size:10px;font-weight:600;text-transform:uppercase;\
                             letter-spacing:.08em;color:{BROWN};background:{OFF_WHITE};\
                             border:1px solid {TAN};border-radius:12px;\
                             padding:2px 8px\">{type_label}</span>\
              </div>\
              {excerpt_h}\
            </div>",
            excerpt_h = if !sexcerpt.is_empty() {
                format!(
                    "<div style=\"font-size:12px;color:{MID};line-height:1.6;font-style:italic\">\
                      &ldquo;{}&rdquo;</div>",
                    sexcerpt.chars().take(280).collect::<String>()
                )
            } else { String::new() },
        ))
    }).collect();

    let titles_section = if decade_titles.is_empty() {
        String::new()
    } else {
        format!(
            "{}{}\
             <a href=\"/?zone=titles\" style=\"display:block;text-align:center;font-size:13px;\
                color:{ORANGE};padding:4px 0 12px\">Full title archive →</a>",
            sh("Title Holders of the Era", Some(decade_titles.len())),
            title_cards,
        )
    };

    let stories_section = if decade_stories.is_empty() {
        String::new()
    } else {
        format!(
            "{}{}",
            sh(
                "Voices — Oral Histories &amp; Scholarship",
                Some(decade_stories.len())
            ),
            story_cards,
        )
    };

    let tl_inner = format!(
        "<div style=\"background:{OFF_WHITE};border-left:4px solid {ORANGE};\
                     padding:12px 14px;border-radius:0 6px 6px 0;\
                     font-size:13px;color:{MID};line-height:1.7;\
                     margin-bottom:16px\">{decade_context}</div>\
         {sh_milestones}\
         {timeline}\
         {titles_section}\
         {stories_section}",
        sh_milestones = sh(
            &format!("{} Milestones", esc(active)),
            Some(decade_entries.len())
        ),
    );

    // Return only the tl fragment for HTMX decade tab swaps
    if fragment.as_deref() == Some("tl") {
        return Html(format!("<div id=\"archive-tl\">{tl_inner}</div>")).into_response();
    }

    let page_archive_title = i18n::t(i18n::translations(), lang, "page.history.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">{page_archive_title}</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          Community history from 1987 to present — {n} milestones documented.</p>\
        <div style=\"display:flex;gap:6px;flex-wrap:wrap;margin-bottom:16px\">{tabs}</div>\
        <div id=\"archive-spin\" class=\"htmx-indicator\">Loading…</div>\
        <div id=\"archive-tl\">{tl_inner}</div>\
        <div class=\"card\">\
          <div style=\"font-weight:600;font-size:14px;margin-bottom:4px\">Clubs &amp; Organisations</div>\
          <div style=\"font-size:12px;color:{MID};margin-bottom:6px\">49 clubs across 20+ countries.</div>\
          <a href=\"/?zone=clubs\" style=\"font-size:12px;color:{ORANGE}\">View all clubs →</a>\
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

pub(crate) fn build_timeline(entries: &[&BearHistoryRow]) -> String {
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
        // Use replace with string slice, not char literal — avoids SQL parser ambiguity
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
                "<div style=\"margin-top:5px\">\n                  <a href=\"{link}\" target=\"_blank\" rel=\"noopener\" \n                     style=\"font-size:10px;color:{ORANGE};text-decoration:none;\n                            border:1px solid {ORANGE};border-radius:10px;\n                            padding:1px 8px\">source ↗</a>\n                </div>"
            )
        } else { String::new() };


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
                {sig_html}\n                {src_html}\
              </div>\
            </div>"
        )
    }).collect()
}
// ── ZONE: BEAR FUTURE ─────────────────────────────────────────
