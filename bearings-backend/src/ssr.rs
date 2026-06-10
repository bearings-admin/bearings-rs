//! ssr.rs — Bearings frontend prototype v2 (2026-06-07)
//!
//! CHANGES FROM v1:
//! - All internal nav links now work: single root handler at GET /
//!   dispatches on ?zone= query param. Every bottom nav item, every
//!   "View all →" link, and every archive decade tab is a plain href
//!   that works without JavaScript.
//! - Bear Archives: decade tabs (1980s|1990s|2000s|2010s|2020s)
//!   Each tab is a link + HTMX hx-get for JS-enhanced swapping.
//!   Entries show: year bubble, category badge, title, description,
//!   significance quote (italic, orange left border).
//! - All external links use target=_blank rel=noopener.
//! - "View all" links corrected to /?zone=X pattern.
//!
//! ROUTING:
//!   GET /  ?zone=now         → NOW zone (default)
//!   GET /  ?zone=coming-up  → COMING UP
//!   GET /  ?zone=archive    → BEAR ARCHIVES
//!   GET /  ?zone=archive&decade=1990s → decade filtered
//!   GET /  ?zone=future     → BEAR FUTURE
//!   GET /  ?zone=places     → Places
//!   GET /  ?zone=events     → Events
//!   GET /  ?zone=clubs      → Clubs
//!   GET /  ?zone=titles     → Titles
//!   GET /  ?zone=creators   → Creators
//!   GET /  ?zone=campaigns  → Campaigns
//!   GET /  ?zone=digital-spaces → Digital Spaces
//!
//! PROTOTYPE NOTE (Gaspar):
//!   Each html-producing function is a candidate for a Tera template
//!   (Option A) or Leptos component (Option B). The structure is
//!   intentionally clean for either extraction path.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{Utc, Months};
use crate::db::SupabaseClient;
use crate::i18n;
use serde::Deserialize;

// ── Query params ──────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct ZoneQuery {
    pub zone:     Option<String>,
    pub decade:   Option<String>,
    pub month:    Option<u32>,
    pub fragment:     Option<String>,  // "events" → return bare #event-list HTML for HTMX
    pub place_type:    Option<String>,
    pub place_country: Option<String>,
    pub lang:          Option<String>,
    pub months_ahead:  Option<u32>,
    pub event_country: Option<String>,
}

// ── Design system (Variant G) ─────────────────────────────────

const BROWN:     &str = "#5C4033";
const ORANGE:    &str = "#D2691E";
const GOLD:      &str = "#D4A017";
const TAN:       &str = "#C8B89A";
const OFF_WHITE: &str = "#F9F5F0";
const DARK:      &str = "#1A1A1A";
const MID:       &str = "#777777";

// ── Shell ─────────────────────────────────────────────────────

fn shell(title: &str, description: &str, active: &str, body: &str, lang: &str) -> String {
    let i18n = i18n::translations();
    let tl = |key: &str| i18n::t(i18n, lang, key);

    // Language switcher
    let lang_switcher: String = [("en","EN"),("es","ES"),("fr","FR")].iter().map(|(code, label)| {
        let active_style = if *code == lang {
            format!("background:{BROWN};color:#fff")
        } else {
            format!("background:transparent;color:{MID}")
        };
        format!(
            "<a href=\"/?zone={active}&lang={code}\" \
               style=\"font-size:10px;font-weight:700;padding:3px 7px;\
                       border-radius:999px;text-decoration:none;{active_style}\">{label}</a>"
        )
    }).collect();

    // Bottom nav — 4 temporal zones, inline SVG icons
    let tnav_svg = |zone: &str, label: &str, svg: &str| {
        let on  = zone == active;
        let col = if on { ORANGE } else { BROWN };
        let fw  = if on { "700" } else { "400" };
        format!(
            "<a href=\"/?zone={zone}&lang={lang}\" \
               style=\"display:flex;flex-direction:column;align-items:center;\
                       gap:3px;text-decoration:none;padding:5px 10px;\
                       border-radius:10px;color:{col};font-weight:{fw};font-size:10px;\
                       letter-spacing:.03em\">\
              <span style=\"display:flex;align-items:center;justify-content:center;height:22px;width:22px\">{svg}</span>\
              {label}\
            </a>"
        )
    };

    // Directory menu items (hamburger)
    let dir_items: &[(&str, &str, &str)] = &[
        ("places",        "🍺", "nav.places"),
        ("clubs",         "🏳\u{fe0f}", "nav.clubs"),
        ("creators",      "🎨", "nav.creators"),
        ("titles",        "🏆", "nav.titles"),
        ("campaigns",     "💚", "nav.campaigns"),
        ("digital-spaces","📱", "nav.digital"),
    ];
    let dir_links: String = dir_items.iter().map(|(zone, icon, key)| {
        let on = zone == &active;
        format!(
            "<a href=\"/?zone={zone}&lang={lang}\" \
               style=\"display:flex;align-items:center;gap:10px;\
                       padding:10px 0;border-bottom:1px solid {TAN};\
                       text-decoration:none;color:{col};font-weight:{fw}\">\
              <span style=\"font-size:18px\">{icon}</span>\
              <span style=\"font-size:14px\">{label}</span>\
            </a>",
            col   = if on { ORANGE } else { DARK },
            fw    = if on { "700" } else { "400" },
            label = tl(key),
        )
    }).collect();

    let _ = tl;
    format!(
        "<!DOCTYPE html>\n\
<html lang=\"{lang}\">\n\
<head>\n\
  <meta charset=\"UTF-8\">\n\
  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n\
  <title>{title} — Bearings</title>\n\
  <meta name=\"description\" content=\"{description}\">\n\
  <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n\
  <link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap\">\n\
  <script src=\"https://unpkg.com/htmx.org@1.9.12\"></script>\n\
  <style>\n\
    *{{box-sizing:border-box;margin:0;padding:0}}\n\
    html,body{{background:{OFF_WHITE};color:{DARK};font-family:Inter,sans-serif;font-size:15px}}\n\
    a{{color:inherit;text-decoration:none}}\n\
    .stripe{{height:5px;background:linear-gradient(to right,\n\
      {DARK} 0% 14.3%,{MID} 14.3% 28.6%,{BROWN} 28.6% 42.9%,\n\
      {ORANGE} 42.9% 57.1%,{GOLD} 57.1% 71.4%,{TAN} 71.4% 85.7%,\n\
      #fff 85.7% 100%)}}\n\
    .card{{background:#fff;border-radius:16px;border:1px solid {TAN};\n\
           padding:14px 16px;margin-bottom:10px;\n\
           box-shadow:0 1px 4px rgba(0,0,0,.06)}}\n\
    .badge{{display:inline-block;font-size:.63rem;font-weight:600;\n\
            padding:2px 7px;border-radius:999px;margin-left:3px}}\n\
    .btn-o{{background:{ORANGE};color:#fff;border-radius:999px;\n\
            padding:7px 16px;font-size:12px;font-weight:600;\n\
            text-decoration:none;white-space:nowrap;display:inline-block}}\n\
    .btn-g{{background:{GOLD};color:{DARK};border-radius:999px;\n\
            padding:7px 16px;font-size:12px;font-weight:600;\n\
            text-decoration:none;white-space:nowrap;display:inline-block}}\n\
    .btn-t{{background:{TAN};color:{BROWN};border-radius:999px;\n\
            padding:7px 12px;font-size:12px;font-weight:600;\n\
            text-decoration:none;white-space:nowrap;display:inline-block}}\n\
    .sh{{font-size:11px;font-weight:700;letter-spacing:.1em;\n\
         text-transform:uppercase;color:{MID};\n\
         margin:22px 0 10px;display:flex;align-items:center;gap:8px}}\n\
    .cp{{font-size:10px;background:{TAN};color:{BROWN};\n\
         border-radius:999px;padding:1px 8px}}\n\
    .htmx-indicator{{opacity:0;transition:opacity .2s;font-size:12px;\n\
                     color:{GOLD};text-align:center;padding:6px 0}}\n\
    .htmx-request .htmx-indicator,\n\
    .htmx-request.htmx-indicator{{opacity:1}}\n\
    .dtab{{padding:6px 14px;border-radius:999px;font-size:12px;\n\
           font-weight:600;border:1px solid {TAN};cursor:pointer;\n\
           white-space:nowrap;text-decoration:none;display:inline-block}}\n\
    .dtab-on{{background:{BROWN};color:#fff;border-color:{BROWN}}}\n\
    .dtab-off{{background:#fff;color:{MID}}}\n\
    .tl-dot{{width:36px;height:36px;border-radius:50%;\n\
             background:{BROWN};color:#fff;font-size:10px;font-weight:700;\n\
             display:flex;align-items:center;justify-content:center;\n\
             flex-shrink:0;line-height:1.1;text-align:center}}\n\
    .tl-line{{width:2px;background:{TAN};flex:1;margin:4px auto 0}}\n\
    .cat{{font-size:9px;font-weight:700;padding:2px 7px;border-radius:999px;\n\
          text-transform:uppercase;letter-spacing:.05em;display:inline-block;\n\
          margin-bottom:6px}}\n\
    /* Hamburger drawer */\n\
    .drawer-chk{{display:none}}\n\
    .drawer-backdrop{{display:none;position:fixed;inset:0;\n\
                      background:rgba(0,0,0,.45);z-index:200}}\n\
    .drawer-panel{{position:fixed;top:0;right:0;bottom:0;width:280px;\n\
                   background:{OFF_WHITE};z-index:201;padding:20px 20px 80px;\n\
                   overflow-y:auto;transform:translateX(100%);\n\
                   transition:transform .2s ease}}\n\
    .drawer-chk:checked ~ .drawer-backdrop{{display:block}}\n\
    .drawer-chk:checked ~ .drawer-panel{{transform:translateX(0)}}\n\
    .drawer-open-btn{{background:none;border:1px solid {TAN};border-radius:8px;\n\
                       padding:6px 10px;cursor:pointer;font-size:16px;\n\
                       color:{BROWN};display:flex;align-items:center;gap:4px;\n\
                       font-family:inherit}}\n\
    .drawer-close-btn{{display:block;text-align:right;font-size:20px;\n\
                        color:{MID};cursor:pointer;margin-bottom:16px;\n\
                        text-decoration:none}}\n\
  </style>\n\
</head>\n\
<body style=\"padding-bottom:72px\">\n\
\n\
  <div class=\"stripe\"></div>\n\
\n\
  <!-- Hamburger drawer (pure CSS) -->\n\
  <input type=\"checkbox\" id=\"drawer-toggle\" class=\"drawer-chk\">\n\
  <label for=\"drawer-toggle\" class=\"drawer-backdrop\"></label>\n\
  <div class=\"drawer-panel\">\n\
    <label for=\"drawer-toggle\" class=\"drawer-close-btn\">✕</label>\n\
    <div style=\"font-size:10px;font-weight:700;letter-spacing:.1em;\
text-transform:uppercase;color:{MID};margin-bottom:4px\">Directory</div>\n\
    {dir_links}\n\
    <div style=\"margin-top:20px;font-size:10px;font-weight:700;letter-spacing:.1em;\
text-transform:uppercase;color:{MID};margin-bottom:4px\">Timeline</div>\n\
    <a href=\"/?zone=archive&lang={lang}\" style=\"display:flex;align-items:center;gap:10px;\
padding:10px 0;border-bottom:1px solid {TAN};text-decoration:none;color:{DARK}\">\
<span style=\"font-size:18px\">📚</span><span style=\"font-size:14px\">Bear Archive</span></a>\n\
    <a href=\"/?zone=future&lang={lang}\" style=\"display:flex;align-items:center;gap:10px;\
padding:10px 0;border-bottom:1px solid {TAN};text-decoration:none;color:{DARK}\">\
<span style=\"font-size:18px\">🔭</span><span style=\"font-size:14px\">Bear Future</span></a>\n\
    <a href=\"/?zone=ical&lang={lang}\" style=\"display:flex;align-items:center;gap:10px;\
padding:10px 0;text-decoration:none;color:{DARK}\">\
<span style=\"font-size:18px\">📅</span><span style=\"font-size:14px\">iCal Export</span></a>\n\
  </div>\n\
\n\
  <header style=\"max-width:640px;margin:0 auto;padding:10px 16px 8px\">\n\
    <div style=\"display:flex;justify-content:space-between;align-items:center\">\n\
      <a href=\"/?zone=coming-up&lang={lang}\" style=\"display:flex;align-items:baseline;gap:8px\">\n\
        <span style=\"font-size:18px;font-weight:700;letter-spacing:.15em;\
color:{BROWN}\">BEARINGS</span>\n\
        <span style=\"font-size:11px;color:{MID}\">global bear community</span>\n\
      </a>\n\
      <div style=\"display:flex;align-items:center;gap:8px\">\n\
        <div style=\"display:flex;gap:2px;border:1px solid {TAN};\
border-radius:999px;padding:2px 3px\">{lang_switcher}</div>\n\
        <label for=\"drawer-toggle\" class=\"drawer-open-btn\">☰</label>\n\
      </div>\n\
    </div>\n\
  </header>\n\
\n\
  <main style=\"max-width:640px;margin:0 auto;padding:4px 16px 16px\">\n\
    {body}\n\
  </main>\n\
\n\
  <nav style=\"position:fixed;bottom:0;left:0;right:0;background:{OFF_WHITE};\n\
              border-top:1px solid {TAN};z-index:100\">\n\
    <div style=\"max-width:640px;margin:0 auto;display:flex;\n\
                justify-content:space-around;align-items:center;\
padding:5px 8px 10px\">\n\
      {n_archive}{n_now}{n_upcoming}{n_future}\n\
    </div>\n\
  </nav>\n\
\n\
</body>\n\
</html>",
        n_archive  = tnav_svg("archive",   "Archive",  "<svg width=\'22\' height=\'22\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'1.8\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><circle cx=\'12\' cy=\'12\' r=\'9\'/><polyline points=\'12 7 12 12 9 15\'/></svg>"),
        n_now      = tnav_svg("now",        "Now",      "<svg width=\'22\' height=\'22\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'1.8\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><path d=\'M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7z\'/><circle cx=\'12\' cy=\'9\' r=\'2.5\'/></svg>"),
        n_upcoming = tnav_svg("coming-up",  "Upcoming", "<svg width=\'22\' height=\'22\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'1.8\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><rect x=\'3\' y=\'4\' width=\'18\' height=\'18\' rx=\'2\'/><line x1=\'16\' y1=\'2\' x2=\'16\' y2=\'6\'/><line x1=\'8\' y1=\'2\' x2=\'8\' y2=\'6\'/><line x1=\'3\' y1=\'10\' x2=\'21\' y2=\'10\'/><line x1=\'8\' y1=\'15\' x2=\'10\' y2=\'15\'/><line x1=\'12\' y1=\'15\' x2=\'16\' y2=\'15\'/></svg>"),
        n_future   = tnav_svg("future",     "Future",   "<svg width=\'22\' height=\'22\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'1.8\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><circle cx=\'12\' cy=\'12\' r=\'9\'/><line x1=\'12\' y1=\'8\' x2=\'12\' y2=\'12\'/><line x1=\'12\' y1=\'12\' x2=\'15\' y2=\'14\'/><circle cx=\'12\' cy=\'12\' r=\'1.5\' fill=\'currentColor\'/></svg>"),
    )
}
fn card(c: &str) -> String {
    format!("<div class=\"card\">{c}</div>")
}

fn sh(label: &str, n: Option<usize>) -> String {
    let pill = n.map(|x| format!("<span class=\"cp\">{x}</span>")).unwrap_or_default();
    format!("<div class=\"sh\">{label}{pill}</div>")
}

fn flags(codes: &[String]) -> String {
    codes.iter().map(|c| {
        let (lbl, bg, fg) = match c.as_str() {
            "men_only"          => ("♂ men only",       "#EDE0D4", BROWN),
            "clothing_optional" => ("🌿 clothing opt.", "#F0EAD6", "#5a6f2b"),
            "members_only"      => ("🔒 members",       "#EDE0D4", BROWN),
            "adults_only"       => ("18+",              "#FCEBD5", ORANGE),
            "bear_focused"      => ("🐻 bear focused",  "#FBF0E0", ORANGE),
            _                   => (c.as_str(),          "#EEE",   "#555"),
        };
        format!("<span class=\"badge\" style=\"background:{bg};color:{fg}\">{lbl}</span>")
    }).collect()
}

fn dist(km: f64) -> String {
    let s = if km < 10.0 { format!("{:.1} km", km) } else { format!("{:.0} km", km) };
    format!("<span class=\"badge\" style=\"background:#EDE0D4;color:{BROWN}\">{s}</span>")
}

fn stat_row() -> String {
    let tiles = [("88","Events"),("170","Places"),("87","Titles"),("49","Clubs")];
    tiles.iter().map(|(v,l)| format!(
        "<div class=\"card\" style=\"text-align:center;padding:10px 4px\">\
          <div style=\"font-size:20px;font-weight:700;color:{ORANGE}\">{v}</div>\
          <div style=\"font-size:11px;color:{MID}\">{l}</div>\
        </div>"
    )).collect::<Vec<_>>().join("")
}

fn timeline_bar(evs: &[serde_json::Value], active_month: Option<u32>, href_base: &str, hx_target: &str) -> String {
    let months = ["Jan","Feb","Mar","Apr","May","Jun",
                  "Jul","Aug","Sep","Oct","Nov","Dec"];
    let bear_col = [DARK,MID,BROWN,BROWN,ORANGE,ORANGE,
                    GOLD,GOLD,TAN,TAN,"#AAAAAA",DARK];
    let mut counts = [0usize; 12];
    for e in evs {
        if let Some(d) = e["start_date"].as_str() {
            let parts: Vec<&str> = d.splitn(3, '-').collect();
            if let Some(m) = parts.get(1).and_then(|s| s.parse::<usize>().ok()) {
                if m >= 1 && m <= 12 { counts[m-1] += 1; }
            }
        }
    }
    let max = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
    let bars: String = (0..12).map(|i| {
        let h   = if counts[i] > 0 { (counts[i] as f64 / max * 40.0).max(5.0) } else { 2.0 };
        let on  = active_month == Some(i as u32 + 1);
        let col = if on { ORANGE } else { bear_col[i] };
        let lc  = if on { ORANGE } else { MID };
        let fw  = if on { "700" } else { "400" };
        let cnt = if counts[i] > 0 { counts[i].to_string() } else { String::new() };
        let htmx = if !hx_target.is_empty() {
            format!(
                " hx-get=\"{href_base}&month={mn}\"\
                  hx-target=\"{tgt}\" hx-select=\"{tgt}\" hx-swap=\"outerHTML\"\
                  hx-indicator=\"#bar-spin\"",
                mn  = i + 1,
                tgt = hx_target,
                href_base = href_base,
            )
        } else { String::new() };
        format!(
            "<a href=\"{href_base}&month={mn}\"{htmx}\
               style=\"flex:1;display:flex;flex-direction:column;align-items:center;gap:2px\"\
               title=\"{lbl}: {n} events\">\
              <span style=\"font-size:9px;color:{lc};font-weight:{fw}\">{cnt}</span>\
              <div style=\"width:100%;height:{h}px;background:{col};border-radius:3px;transition:all .15s\"></div>\
              <span style=\"font-size:8px;color:{lc};font-weight:{fw}\">{lbl}</span>\
            </a>",
            mn  = i + 1,
            lbl = months[i],
            n   = counts[i],
        )
    }).collect();
    let clear_link = if active_month.is_some() {
        format!(
            "<a href=\"{href_base}\" style=\"font-size:10px;color:{ORANGE};text-decoration:none;\
              display:block;text-align:right;margin-top:4px\">✕ clear filter</a>"
        )
    } else { String::new() };
    format!(
        "<div class=\"card\" style=\"padding:12px 14px\">\
          <div style=\"font-size:10px;font-weight:600;color:{MID};margin-bottom:8px;\
                      text-transform:uppercase;letter-spacing:.08em\">Events by month · click to filter</div>\
          <div style=\"display:flex;gap:3px;align-items:flex-end;height:56px\">{bars}</div>\
          <div id=\"bar-spin\" class=\"htmx-indicator\">loading…</div>\
          {clear_link}\
        </div>"
    )
}

fn extract_month(date_str: &str) -> Option<u32> {
    let parts: Vec<&str> = date_str.splitn(3, '-').collect();
    parts.get(1).and_then(|s| s.parse::<u32>().ok())
}

fn ev_flags(ev: &serde_json::Value) -> Vec<String> {
    ev["inclusion_flag_codes"].as_array()
        .map(|v| v.iter().filter_map(|s| s.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

// ── ROOT DISPATCHER ───────────────────────────────────────────

pub async fn root(
    State(db): State<SupabaseClient>,
    Query(q): Query<ZoneQuery>,
) -> Response {
    let lang_owned = match q.lang.as_deref() { Some("es") => "es", Some("fr") => "fr", _ => "en" };
    match q.zone.as_deref().unwrap_or("now") {
        "now"            => zone_now(db, lang_owned).await,
        "coming-up"      => zone_coming_up(db, q.months_ahead, q.event_country.clone(), q.month, lang_owned).await,
        "archive"        => zone_archive(db, q.decade, q.fragment.clone(), lang_owned).await,
        "future"         => zone_future(db, lang_owned).await,
        "places"         => zone_places(db, q.place_type.clone(), q.place_country.clone(), lang_owned).await,
        "events"         => zone_events(db, q.month, lang_owned).await,
        "clubs"          => zone_clubs(db, lang_owned).await,
        "titles"         => zone_titles(db, lang_owned).await,
        "creators"       => zone_creators(db, lang_owned).await,
        "campaigns"      => zone_campaigns(db, lang_owned).await,
        "digital-spaces" => zone_digital(db, lang_owned).await,
        "ical"           => zone_ical(lang_owned).await,
        _                => zone_coming_up(db, None, None, None, lang_owned).await,
    }
}

// ── ZONE: NOW ─────────────────────────────────────────────────

async fn zone_now(db: SupabaseClient, lang: &str) -> Response {
    // Worldwide events starting within the next 30 days
    let today   = Utc::now().date_naive();
    let in_30   = today.checked_add_days(chrono::Days::new(30)).unwrap_or(today);
    let from_s  = today.format("%Y-%m-%d").to_string();
    let to_s    = in_30.format("%Y-%m-%d").to_string();

    let events_url = format!(
        "{}/rest/v1/events\
         ?active=eq.true\
         &start_date=gte.{from_s}\
         &start_date=lte.{to_s}\
         &select=id,name,city,country,start_date,end_date,type,hot,link,description,\
                 event_mode,inclusion_flag_codes\
         &order=start_date.asc\
         &limit=40",
        db.url
    );

    // Active campaigns
    let camps_url = format!(
        "{}/rest/v1/campaigns?active=eq.true&select=name,org,link,raised,goal,currency,urgent,ends_at&order=urgent.desc,ends_at.asc.nullslast&limit=6",
        db.url
    );
    // Current title holders with competition scope via embedded join, year >= 2022
    let titles_url = format!(
        "{}/rest/v1/title_holders?year=gte.2022&select=title_name,holder_name,year,city,country,competitions(scope)&order=year.desc&limit=40",
        db.url
    );

    let (events_res, camps_res, titles_res) = tokio::join!(
        db.get_json::<Vec<serde_json::Value>>(&events_url),
        db.get_json::<Vec<serde_json::Value>>(&camps_url),
        db.get_json::<Vec<serde_json::Value>>(&titles_url),
    );

    let events = events_res.unwrap_or_default();
    let cmpg   = camps_res.unwrap_or_default();
    let ttls   = titles_res.unwrap_or_default();

    // ── Event cards ───────────────────────────────────────────
    let event_cards: String = events.iter().map(|ev| {
        let name   = ev["name"].as_str().unwrap_or("");
        let city   = ev["city"].as_str().unwrap_or("");
        let ctry   = ev["country"].as_str().unwrap_or("");
        let start  = ev["start_date"].as_str().unwrap_or("");
        let end    = ev["end_date"].as_str().unwrap_or("");
        let link   = ev["link"].as_str().unwrap_or("");
        let etype  = ev["type"].as_str().unwrap_or("");
        let hot    = ev["hot"].as_bool().unwrap_or(false);
        let fs     = ev_flags(ev);
        let dates  = if !end.is_empty() && end != start {
            format!("{start} → {end}")
        } else { start.to_string() };
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
        } else { String::new() };
        let hot_badge = if hot {
            format!("<span style=\"font-size:9px;background:{ORANGE};color:#fff;\
                      border-radius:6px;padding:1px 5px;margin-right:4px\">🔥 hot</span>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}{sep}{ctry}</div>\
                <div style=\"font-size:12px;color:{MID}\">{dates}</div>\
                <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                  {hot_badge}\
                  <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                  {fhtml}\
                </div>\
              </div>\
              {link_html}\
            </div>",
            sep   = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            fhtml = flags(&fs),
        ))
    }).collect();

    let empty_events = if events.is_empty() {
        format!(
            "<div style=\"text-align:center;padding:24px 0;color:{MID}\">\
              <div style=\"font-size:32px;margin-bottom:8px\">🐻</div>\
              <div style=\"font-size:13px;font-weight:600\">Nothing in the next 30 days</div>\
              <div style=\"font-size:12px;margin-top:4px\">\
                <a href=\"/?zone=coming-up&lang={lang}\" style=\"color:{ORANGE}\">Browse upcoming events →</a>\
              </div>\
            </div>"
        )
    } else { String::new() };

    // ── Campaign cards ────────────────────────────────────────
    let camp_cards: String = cmpg.iter().take(4).map(|c| {
        let name   = c["name"].as_str().unwrap_or("");
        let org    = c["org"].as_str().unwrap_or("");
        let link   = c["link"].as_str().unwrap_or("");
        let urgent = c["urgent"].as_bool().unwrap_or(false);
        let raised = c["raised"].as_i64();
        let goal   = c["goal"].as_i64();
        let curr   = c["currency"].as_str().unwrap_or("USD");
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-g\">Donate</a>")
        } else { String::new() };
        let urgent_badge = if urgent {
            format!("<span style=\"font-size:9px;background:#C0392B;color:#fff;\
                      border-radius:6px;padding:1px 5px;margin-right:4px\">URGENT</span>")
        } else { String::new() };
        let progress = match (raised, goal) {
            (Some(r), Some(g)) if g > 0 => {
                let pct = ((r as f64 / g as f64) * 100.0).min(100.0);
                format!(
                    "<div style=\"margin-top:6px;background:{TAN};border-radius:3px;height:4px\">\
                      <div style=\"background:{ORANGE};height:100%;width:{pct:.0}%\"></div>\
                    </div>"
                )
            },
            _ => String::new(),
        };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{urgent_badge}{name}</div>\
                <div style=\"font-size:11px;color:{MID};margin-top:2px\">{org}</div>\
                {progress}\
              </div>\
              <div style=\"flex-shrink:0\">{link_html}</div>\
            </div>"
        ))
    }).collect();

    // ── Title holder cards grouped by scope ─────────────────────
    let scope_order = ["international","continental","national","regional","local"];
    let title_cards: String = scope_order.iter().filter_map(|&sc| {
        let group: Vec<_> = ttls.iter().filter(|t| {
            t["competitions"]["scope"].as_str().unwrap_or("") == sc
        }).collect();
        if group.is_empty() { return None; }
        let scope_label = match sc {
            "international" => "International",
            "continental"   => "Continental",
            "national"      => "National",
            "regional"      => "Regional",
            _               => "Local",
        };
        let rows: String = group.iter().map(|t| {
            let title  = t["title_name"].as_str().unwrap_or("");
            let holder = t["holder_name"].as_str().unwrap_or("");
            let year   = t["year"].as_i64().unwrap_or(0);
            let city   = t["city"].as_str().unwrap_or("");
            let ctry   = t["country"].as_str().unwrap_or("");
            card(&format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center\">                  <div>                    <div style=\"font-weight:600;font-size:14px\">{title}</div>                    <div style=\"font-size:12px;color:{MID};margin-top:2px\">{holder}</div>                    <div style=\"font-size:11px;color:{MID}\">{city}{sep}{ctry}</div>                  </div>                  <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">{year}</div>                </div>",
                sep = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            ))
        }).collect();
        Some(format!(
            "<div style=\"font-size:11px;font-weight:700;text-transform:uppercase;                         letter-spacing:.1em;color:{BROWN};padding:8px 0 4px\">              {scope_label}</div>{rows}"
        ))
    }).collect();

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Now</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:12px\">\
          What the bear world is doing in the next 30 days.</p>\
        \
        {h_events}\
        {event_cards}\
        {empty_events}\
        <a href=\"/?zone=coming-up&lang={lang}\" \
           style=\"display:block;text-align:center;font-size:13px;\
                  color:{ORANGE};padding:8px 0 16px\">Browse all upcoming events →</a>\
        \
        {h_camps}\
        {camp_cards}\
        <a href=\"/?zone=future&lang={lang}\" \
           style=\"display:block;text-align:center;font-size:13px;\
                  color:{ORANGE};padding:4px 0 16px\">All campaigns →</a>\
        \
        {h_titles}\
        {title_cards}\
        <a href=\"/?zone=titles\" \
           style=\"display:block;text-align:center;font-size:13px;\
                  color:{ORANGE};padding:4px 0 24px\">Full competition archive →</a>",
        h_events = sh("Happening in the Next 30 Days", Some(events.len())),
        h_camps  = sh("Community Campaigns", Some(cmpg.len())),
        h_titles = sh("Current Title Holders", Some(ttls.len())),
    );

    Html(shell("Now", "Bear events in the next 30 days.", "now", &body, lang)).into_response()
}

// ── ZONE: COMING UP ───────────────────────────────────────────

async fn zone_coming_up(db: SupabaseClient, months_ahead: Option<u32>, event_country: Option<String>, month_filter: Option<u32>, lang: &str) -> Response {
    let months = months_ahead.unwrap_or(6).clamp(1, 24);
    let country = event_country.as_deref().unwrap_or("");

    // Compute date window
    let today    = Utc::now().date_naive();
    let to_date  = today.checked_add_months(Months::new(months)).unwrap_or(today);
    let from_str = today.format("%Y-%m-%d").to_string();
    let to_str   = to_date.format("%Y-%m-%d").to_string();

    let country_val = if country.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(country.to_string())
    };

    let rpc_body = serde_json::json!({
        "input_lat":    serde_json::Value::Null,
        "input_lng":    serde_json::Value::Null,
        "radius_km":    serde_json::Value::Null,
        "season":       serde_json::Value::Null,
        "from_date":    from_str,
        "to_date":      to_str,
        "event_type":   serde_json::Value::Null,
        "country":      country_val,
        "max_rows":     60,
    });
    let data: serde_json::Value = match db.post_rpc("coming_up", &rpc_body).await {
        Ok(v)  => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "coming_up rpc failed").into_response(),
    };
    let events = data["events"].as_array().cloned().unwrap_or_default();
    let venues = data["venues"].as_array().cloned().unwrap_or_default();

    // ── Selectors ─────────────────────────────────────────────
    let sel_style = "width:100%;padding:10px 12px;border-radius:12px;\
                     border:1px solid {TAN};background:#fff;\
                     font-size:13px;color:{DARK};font-family:inherit";

    let months_opts: &[(u32, &str)] = &[
        (1, "Next month"), (2, "Next 2 months"), (3, "Next 3 months"),
        (6, "Next 6 months"), (12, "Next year"),
    ];
    let months_sel: String = months_opts.iter().map(|(v, l)| {
        let sel = if *v == months { " selected" } else { "" };
        format!("<option value=\"{v}\"{sel}>{l}</option>")
    }).collect();

    // Country groups
    let regions: &[(&str, &[&str])] = &[
        ("North America",  &["Canada", "USA", "Mexico"]),
        ("Europe",         &["Belgium","Czech Republic","France","Germany","Iceland",
                              "Ireland","Italy","Luxembourg","Netherlands","Poland",
                              "Portugal","Scotland","Spain","Sweden","Switzerland","UK"]),
        ("Asia Pacific",   &["Australia","Japan","New Zealand","Thailand"]),
        ("Latin America",  &["Brazil","Argentina","Chile","Colombia","Mexico"]),
        ("Africa & Middle East", &["South Africa","Egypt","Morocco"]),
    ];
    let world_sel = if country.is_empty() { " selected" } else { "" };
    let mut country_opts = format!("<option value=\"\"{world_sel}>🌍 Worldwide</option>");
    for (region, countries) in regions {
        country_opts.push_str(&format!("<optgroup label=\"{region}\">"));
        for c in *countries {
            let sel = if *c == country { " selected" } else { "" };
            country_opts.push_str(&format!("<option value=\"{c}\"{sel}>{c}</option>"));
        }
        country_opts.push_str("</optgroup>");
    }

    let where_label = if country.is_empty() { "Worldwide".to_string() }
                      else { country.to_string() };
    let month_label = months_opts.iter()
        .find(|(v, _)| *v == months)
        .map(|(_, l)| *l)
        .unwrap_or("6 months");

    // ── Monthly bar chart + optional month filter ───────────────
    let country_enc = if country.is_empty() { String::new() } else {
        format!("&event_country={}", country.replace(' ', "%20"))
    };
    let bar_base = format!("/?zone=coming-up&months_ahead={months}&lang={lang}{country_enc}");
    let bar = timeline_bar(&events, month_filter, &bar_base, "#upcoming-results");

    // Filter displayed events by selected month
    let disp_events: Vec<serde_json::Value> = if let Some(m) = month_filter {
        events.iter().filter(|ev| {
            ev["start_date"].as_str()
                .and_then(|d| d.splitn(3, '-').nth(1))
                .and_then(|s| s.parse::<u32>().ok())
                .map(|em| em == m)
                .unwrap_or(false)
        }).cloned().collect()
    } else {
        events.clone()
    };

    // ── Event cards ───────────────────────────────────────────
    let ev_cards: String = disp_events.iter().map(|ev| {
        let name  = ev["name"].as_str().unwrap_or("");
        let city  = ev["city"].as_str().unwrap_or("");
        let ctry  = ev["country"].as_str().unwrap_or("");
        let start = ev["start_date"].as_str().unwrap_or("");
        let end   = ev["end_date"].as_str().unwrap_or("");
        let link  = ev["link"].as_str().unwrap_or("");
        let etype = ev["type"].as_str().unwrap_or("");
        let fs    = ev_flags(ev);
        let dates = if !end.is_empty() && end != start {
            format!("{start} → {end}")
        } else { start.to_string() };
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;\
                         align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">\
                  {city}{sep}{ctry} · {dates}</div>\
                <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                  <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                  {fhtml}\
                </div>\
              </div>\
              {link_html}\
            </div>",
            sep   = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            fhtml = flags(&fs),
        ))
    }).collect();

    let empty_html = if disp_events.is_empty() {
        format!(
            "<div style=\"text-align:center;padding:32px 0;color:{MID}\">\
              <div style=\"font-size:32px;margin-bottom:8px\">🐻</div>\
              <div style=\"font-size:14px;font-weight:600\">No events found</div>\
              <div style=\"font-size:12px;margin-top:4px\">\
                Try a longer time window or a different region.</div>\
            </div>"
        )
    } else { String::new() };

    // iCal subscribe block
    let ical_block = format!(
        "<div style=\"border-radius:14px;border:1px solid {GOLD};\
             background:#FFFEF5;padding:14px 16px;margin-bottom:10px;\
             display:flex;justify-content:space-between;align-items:center;gap:12px\">\
          <div>\
            <div style=\"font-weight:600;font-size:12px;color:{BROWN};margin-bottom:2px\">\
              📅 Subscribe to bear events</div>\
            <div style=\"font-size:11px;color:{MID}\">Auto-updates in any calendar app</div>\
          </div>\
          <a href=\"/api/events/ical.ics\" class=\"btn-g\">iCal</a>\
        </div>"
    );

    // Venues section (compact)
    let vn_cards: String = venues.iter().take(3).map(|v| {
        let name  = v["name"].as_str().unwrap_or("");
        let ptype = v["place_type"].as_str().unwrap_or("");
        let city  = v["city"].as_str().unwrap_or("");
        let ctry  = v["country"].as_str().unwrap_or("");
        let site  = v["website"].as_str().unwrap_or("");
        let site_btn = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" class=\"btn-t\">Visit</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:center\">\
              <div>\
                <div style=\"font-weight:600;font-size:13px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ptype}</span></div>\
                <div style=\"font-size:12px;color:{MID}\">{city}, {ctry}</div>\
              </div>\
              {site_btn}\
            </div>"
        ))
    }).collect();

    let body = format!(
        "<div style=\"text-align:center;padding:20px 0 12px\">\
          <h1 style=\"font-size:22px;font-weight:700;color:{BROWN};line-height:1.2;\
                      margin-bottom:6px\">When &amp; Where<br>\
            <span style=\"font-size:15px;font-weight:400;color:{MID}\">do you want to meet?</span>\
          </h1>\
        </div>\
        \
        <form id=\"upcoming-filters\"\
              hx-get=\"/?zone=coming-up\"\
              hx-target=\"#upcoming-results\"\
              hx-select=\"#upcoming-results\"\
              hx-swap=\"outerHTML\"\
              hx-trigger=\"change from:#upcoming-filters\"\
              hx-indicator=\"#cu-spin\"\
              style=\"margin-bottom:12px\">\
          <input type=\"hidden\" name=\"lang\" value=\"{lang}\">\
          <div style=\"display:grid;grid-template-columns:1fr 1fr;gap:8px\">\
            <select name=\"months_ahead\" style=\"{sel_style}\">{months_sel}</select>\
            <select name=\"event_country\" style=\"{sel_style}\">{country_opts}</select>\
          </div>\
        </form>\
        \
        <div id=\"cu-spin\" class=\"htmx-indicator\">Finding events…</div>\
        \
        {bar}\
        \
        <div id=\"upcoming-results\">\
          {h_ev}\
          {ev_cards}\
          {empty_html}\
          {ical_block}\
          {h_vn}\
          {vn_cards}\
        </div>",
        h_ev  = sh(&format!("{month_label} · {where_label}"), Some(disp_events.len())),
        h_vn  = if venues.is_empty() { String::new() } else {
            sh(&format!("Venues in {where_label}"), Some(venues.len()))
        },
    );
    Html(shell("Upcoming Events", "Find bear events near you.", "coming-up", &body, lang)).into_response()
}

// ── ZONE: BEAR ARCHIVES (decade tabs) ────────────────────────

async fn zone_archive(db: SupabaseClient, decade: Option<String>, fragment: Option<String>, lang: &str) -> Response {
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
        "{}/rest/v1/stories?active=eq.true\
         &select=title,story_type,year,excerpt\
         &order=year.desc&limit=100",
        db.url
    );

    let (history_res, titles_res, stories_res) = tokio::join!(
        db.get_json::<Vec<serde_json::Value>>(&url),
        db.get_json::<Vec<serde_json::Value>>(&titles_url),
        db.get_json::<Vec<serde_json::Value>>(&stories_url),
    );

    let history = match history_res {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };
    let all_titles  = titles_res.unwrap_or_default();
    let all_stories = stories_res.unwrap_or_default();

    let decades = ["1980s","1990s","2000s","2010s","2020s"];
    let active  = decade.as_deref().unwrap_or("2020s");
    let d_start: i64 = match active {
        "1980s" => 1980, "1990s" => 1990, "2000s" => 2000, "2010s" => 2010, _ => 2020,
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
            .filter(|h| h["year"].as_i64().map(|y| y >= ds && y < ds+10).unwrap_or(false))
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
    let decade_entries: Vec<&serde_json::Value> = history.iter()
        .filter(|h| h["year"].as_i64()
            .map(|y| y >= d_start && y < d_start + 10)
            .unwrap_or(false))
        .collect();

    let decade_titles: Vec<&serde_json::Value> = all_titles.iter()
        .filter(|t| t["year"].as_i64()
            .map(|y| y >= d_start && y < d_start + 10)
            .unwrap_or(false))
        .collect();

    let decade_stories: Vec<&serde_json::Value> = all_stories.iter()
        .filter(|st| st["year"].as_i64()
            .map(|y| y >= d_start && y < d_start + 10)
            .unwrap_or(false))
        .collect();

    let timeline = build_timeline(&decade_entries);

    let title_cards: String = decade_titles.iter().map(|t| {
        let title  = t["title_name"].as_str().unwrap_or("");
        let holder = t["holder_name"].as_str().unwrap_or("");
        let year   = t["year"].as_i64().unwrap_or(0);
        let city   = t["city"].as_str().unwrap_or("");
        let ctry   = t["country"].as_str().unwrap_or("");
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:center\">\
              <div>\
                <div style=\"font-weight:600;font-size:14px\">{title}</div>\
                <div style=\"font-size:12px;color:{MID}\">{holder}</div>\
                <div style=\"font-size:11px;color:{MID}\">{city}{sep}{ctry}</div>\
              </div>\
              <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">{year}</div>\
            </div>",
            sep = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
        ))
    }).collect();

    let story_cards: String = decade_stories.iter().map(|st| {
        let stitle   = st["title"].as_str().unwrap_or("");
        let stype    = st["story_type"].as_str().unwrap_or("story");
        let syear    = st["year"].as_i64().unwrap_or(0);
        let sexcerpt = st["excerpt"].as_str().unwrap_or("");
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
            sh("Voices — Oral Histories &amp; Scholarship", Some(decade_stories.len())),
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
        sh_milestones = sh(&format!("{active} Milestones"), Some(decade_entries.len())),
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
    Html(shell("Bear Archives", "Community history 1987 to present.", "archive", &body, lang)).into_response()
}

fn build_timeline(entries: &[&serde_json::Value]) -> String {
    if entries.is_empty() {
        return format!(
            "<div style=\"text-align:center;color:{MID};font-size:13px;padding:24px 0\">\
              No records for this decade yet.</div>"
        );
    }
    entries.iter().map(|h| {
        let year  = h["year"].as_i64().unwrap_or(0);
        let title = h["title"].as_str().unwrap_or("");
        let desc  = h["description"].as_str().unwrap_or("");
        let sig   = h["significance"].as_str().unwrap_or("");
        let cat   = h["category"].as_str().unwrap_or("milestone");
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

        let link     = h["link"].as_str().unwrap_or("");
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

async fn zone_future(db: SupabaseClient, lang: &str) -> Response {
    // Fetch active campaigns
    let url_camps = format!(
        "{}/rest/v1/campaigns\
         ?active=eq.true\
         &select=name,org,description,link,raised,goal,currency,urgent,ends_at\
         &order=urgent.desc,ends_at.asc.nullslast\
         &limit=20",
        db.url
    );
    // Fetch recent titleholders (milestones)
    let url_recent = format!(
        "{}/rest/v1/title_holders\
         ?year=gte.2023\
         &select=holder_name,year,city,country,title_name,competition_id,inclusion_flag_codes,bio\
         &order=year.desc&limit=20",
        db.url
    );
    // Fetch ideas from DB ordered by upvotes desc
    let url_ideas = format!(
        "{}/rest/v1/future_ideas\
         ?active=eq.true\
         &select=id,icon,title,description,upvotes,source\
         &order=upvotes.desc,id.asc\
         &limit=20",
        db.url
    );
    let (camps_res, recent_res, ideas_res) = tokio::join!(
        db.get_json::<Vec<serde_json::Value>>(&url_camps),
        db.get_json::<Vec<serde_json::Value>>(&url_recent),
        db.get_json::<Vec<serde_json::Value>>(&url_ideas),
    );
    let campaigns    = camps_res.unwrap_or_default();
    let recent_title = recent_res.unwrap_or_default();
    let ideas        = ideas_res.unwrap_or_default();

    // ── Section 1: Active Campaigns ───────────────────────────
    let camp_cards: String = campaigns.iter().map(|c| {
        let name   = c["name"].as_str().unwrap_or("");
        let org    = c["org"].as_str().unwrap_or("");
        let desc   = c["description"].as_str().unwrap_or("");
        let link   = c["link"].as_str().unwrap_or("");
        let raised = c["raised"].as_i64();
        let goal   = c["goal"].as_i64();
        let curr   = c["currency"].as_str().unwrap_or("USD");
        let urgent = c["urgent"].as_bool().unwrap_or(false);
        let ends   = c["ends_at"].as_str().unwrap_or("");

        let progress_html = match (raised, goal) {
            (Some(r), Some(g)) if g > 0 => {
                let pct   = ((r as f64 / g as f64) * 100.0).min(100.0);
                let r_fmt = if r >= 1_000_000 { format!("${:.1}M", r as f64/1_000_000.0) }
                            else if r >= 1_000 { format!("${:.0}k", r as f64/1_000.0) }
                            else { format!("${r}") };
                let g_fmt = if g >= 1_000_000 { format!("${:.1}M", g as f64/1_000_000.0) }
                            else if g >= 1_000 { format!("${:.0}k", g as f64/1_000.0) }
                            else { format!("${g}") };
                format!(
                    "<div style=\"margin:8px 0\">\
                      <div style=\"background:{TAN};border-radius:4px;height:6px;overflow:hidden\">\
                        <div style=\"background:{ORANGE};height:100%;width:{pct:.0}%\"></div>\
                      </div>\
                      <div style=\"font-size:10px;color:{MID};margin-top:2px\">\
                        {r_fmt} raised of {g_fmt} {curr}</div>\
                    </div>"
                )
            },
            (Some(r), None) => {
                let r_fmt = if r >= 1_000_000 { format!("${:.1}M", r as f64/1_000_000.0) }
                            else if r >= 1_000 { format!("${:.0}k", r as f64/1_000.0) }
                            else { format!("${r}") };
                format!("<div style=\"font-size:11px;color:{ORANGE};margin:4px 0\">{r_fmt} raised {curr}</div>")
            },
            _ => String::new(),
        };

        let link_btn = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Donate</a>")
        } else { String::new() };

        let ends_html = if !ends.is_empty() {
            format!("<div style=\"font-size:10px;color:{MID}\">Ends {ends}</div>")
        } else { String::new() };

        let urgent_badge = if urgent {
            format!("<span style=\"font-size:9px;background:#C0392B;color:#fff;\
                      border-radius:6px;padding:2px 6px;margin-right:4px\">URGENT</span>")
        } else { String::new() };

        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{urgent_badge}{name}</div>\
                <div style=\"font-size:11px;color:{ORANGE};font-weight:600;margin:2px 0\">{org}</div>\
                <div style=\"font-size:12px;color:{MID};line-height:1.6;margin-top:4px\">{short_desc}</div>\
                {progress_html}\
                {ends_html}\
              </div>\
              <div style=\"flex-shrink:0\">{link_btn}</div>\
            </div>",
            short_desc = desc.chars().take(220).collect::<String>(),
        ))
    }).collect();

    // ── Section 2: Breaking Ground ────────────────────────────
    let milestone_cards: String = recent_title.iter().map(|h| {
        let name  = h["holder_name"].as_str().unwrap_or("");
        let year  = h["year"].as_i64().unwrap_or(0);
        let title = h["title_name"].as_str().unwrap_or("");
        let city  = h["city"].as_str().unwrap_or("");
        let ctry  = h["country"].as_str().unwrap_or("");
        let bio   = h["bio"].as_str().unwrap_or("");
        let fs: Vec<String> = h["inclusion_flag_codes"].as_array()
            .map(|v| v.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1\">\
                <div style=\"font-weight:600;font-size:14px\">{name}</div>\
                <div style=\"font-size:11px;color:{ORANGE};font-weight:600\">{title}</div>\
                <div style=\"font-size:11px;color:{MID}\">{loc}</div>\
                {bio_h}\
                {fhtml}\
              </div>\
              <div style=\"font-size:20px;font-weight:700;color:{GOLD};flex-shrink:0\">{year}</div>\
            </div>",
            loc   = match (city, ctry) {
                (c, ct) if !c.is_empty() && !ct.is_empty() => format!("{c}, {ct}"),
                (c, _)  if !c.is_empty() => c.to_string(),
                (_, ct) if !ct.is_empty() => ct.to_string(),
                _ => String::new(),
            },
            bio_h = if !bio.is_empty() {
                format!("<div style=\"font-size:11px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>",
                    bio.chars().take(200).collect::<String>())
            } else { String::new() },
            fhtml = if !fs.is_empty() {
                format!("<div style=\"margin-top:4px\">{}</div>", flags(&fs))
            } else { String::new() },
        ))
    }).collect();

    // ── Section 3: New Bear Territories ──────────────────────
    let regions_html = format!(
        "<div class=\"card\">\
          <div style=\"font-size:13px;line-height:1.8;color:{DARK}\">\
            <p style=\"margin:0 0 10px\">The bear community is growing into new regions — some with full \
              support from local law and culture, others where community members face serious legal risk.</p>\
            <p style=\"margin:0 0 8px\">Safety data is maintained by \
              <a href=\"https://ilga.org\" target=\"_blank\" rel=\"noopener\" \
                 style=\"color:{ORANGE}\">ILGA World</a>, \
              which tracks the legal status of same-sex relationships in every country.</p>\
            <div style=\"border-left:3px solid {ORANGE};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Malaysia</strong> — Homosexuality is criminalized under both \
              civil and Sharia law. In 2026, Mr Bear International titleholder Gavin Chow (the first \
              Malaysian to hold the title) struggled to find a venue willing to host the national qualifier. \
              His reign is an act of visibility under genuine personal risk.\
            </div>\
            <div style=\"border-left:3px solid {GOLD};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Middle East &amp; North Africa</strong> — \
              Bilal Sakr (Mr Bear Canada 2025), the first openly Middle Eastern titleholder of a major \
              bear competition, actively fundraises for Rainbow Railroad supporting LGBTQ+ refugees \
              from the region.\
            </div>\
            <div style=\"border-left:3px solid {TAN};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Eastern Europe &amp; Central Asia</strong> — \
              Poland and Czech Republic have established competitions; Hungary has regressed on rights. \
              ILGA''s rainbow map is the current best reference.\
            </div>\
            <div style=\"border-left:3px solid {TAN};padding-left:12px;margin:10px 0\">\
              <strong style=\"color:{BROWN}\">Sub-Saharan Africa &amp; Southeast Asia</strong> — \
              South Africa remains the continent''s primary bear hub. Thailand hosts Mr Bear International. \
              Other countries require significant caution — check ILGA data before travel.\
            </div>\
          </div>\
        </div>"
    );

    // ── Section 4: What Could Be — DB ideas with upvote buttons ─
    let idea_cards: String = ideas.iter().enumerate().map(|(i, idea)| {
        let id      = idea["id"].as_i64().unwrap_or(0);
        let icon    = idea["icon"].as_str().unwrap_or("💡");
        let title   = idea["title"].as_str().unwrap_or("");
        let desc    = idea["description"].as_str().unwrap_or("");
        let upvotes = idea["upvotes"].as_i64().unwrap_or(0);
        let source  = idea["source"].as_str().unwrap_or("curated");

        let source_badge = match source {
            "community" => format!("<span style=\"font-size:9px;background:{GOLD};color:{DARK};\
                            border-radius:6px;padding:1px 5px;margin-left:6px\">community</span>"),
            "ai"        => format!("<span style=\"font-size:9px;background:{TAN};color:{BROWN};\
                            border-radius:6px;padding:1px 5px;margin-left:6px\">AI</span>"),
            _           => String::new(),
        };

        // Top idea gets a subtle highlight
        let card_extra = if i == 0 {
            format!("border:1px solid {ORANGE};")
        } else { String::new() };

        format!(
            "<div class=\"card\" style=\"margin-bottom:8px;{card_extra}\">\
              <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
                <div style=\"flex:1;min-width:0\">\
                  <div style=\"font-size:15px;font-weight:700;color:{BROWN};margin-bottom:4px\">\
                    {icon} {title}{source_badge}</div>\
                  <div style=\"font-size:12px;color:{MID};line-height:1.7\">{desc}</div>\
                </div>\
                <div hx-post=\"/api/future-ideas/{id}/upvote\"\
                     hx-swap=\"outerHTML\"\
                     hx-target=\"this\"\
                     style=\"flex-shrink:0;display:flex;flex-direction:column;\
                             align-items:center;cursor:pointer;padding:6px 10px;\
                             border-radius:10px;border:1px solid {TAN};\
                             background:{OFF_WHITE};color:{BROWN};transition:all .15s;\
                             user-select:none;min-width:44px\"\
                     onclick=\"this.style.background='{ORANGE}';this.style.color='#fff'\">\
                  <span style=\"font-size:16px;line-height:1\">▲</span>\
                  <span style=\"font-size:12px;font-weight:700;margin-top:2px\">{upvotes}</span>\
                </div>\
              </div>\
            </div>",
        )
    }).collect();

    let submit_card = format!(
        "<div class=\"card\" style=\"text-align:center;margin-top:4px\">\
          <div style=\"font-size:12px;color:{MID};margin-bottom:6px\">\
            Have an idea? The bottom of the list rotates with community suggestions.</div>\
          <a href=\"mailto:ursasteward@pm.me?subject=Bear%20Future%20Idea\" class=\"btn-o\">\
            Submit an idea</a>\
        </div>"
    );

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bear Future</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          How bears are already making tomorrow better — and what could be.</p>\
        \
        {h1}\
        {camp_cards}\
        {empty_camps}\
        \
        {h2}\
        {milestone_cards}\
        {empty_milestones}\
        \
        {h3}\
        {regions_html}\
        \
        {h4}\
        {idea_cards}\
        {submit_card}",
        h1 = sh("Bears Taking Action", Some(campaigns.len())),
        h2 = sh("Breaking Ground — Recent Milestones", Some(recent_title.len())),
        h3 = format!("<div style=\"font-size:14px;font-weight:700;color:{BROWN};margin:12px 0 6px;\
               border-left:3px solid {ORANGE};padding-left:8px\">New Bear Territories</div>"),
        h4 = format!("<div style=\"font-size:14px;font-weight:700;color:{BROWN};margin:12px 0 6px;\
               border-left:3px solid {GOLD};padding-left:8px\">What Could Be \
               <span style=\"font-size:11px;font-weight:400;color:{MID}\">· upvote to sort</span></div>"),
        empty_camps = if campaigns.is_empty() {
            format!("<div style=\"font-size:12px;color:{MID};padding:8px\">No active campaigns.</div>")
        } else { String::new() },
        empty_milestones = if recent_title.is_empty() {
            format!("<div style=\"font-size:12px;color:{MID};padding:8px\">No recent titleholders.</div>")
        } else { String::new() },
    );
    Html(shell("Bear Future", "Community direction and what comes next.", "future", &body, lang)).into_response()
}

async fn zone_places(db: SupabaseClient, filter_type: Option<String>, filter_country: Option<String>, lang: &str) -> Response {
    let ft = filter_type.as_deref().unwrap_or("");
    let fc = filter_country.as_deref().unwrap_or("");

    // Fetch ALL places for country (for tab counts) — no type filter
    let cc = if !fc.is_empty() { format!("&country=eq.{fc}") } else { String::new() };
    let url_all = format!(
        "{}/rest/v1/places?active=eq.true\
         &select=place_type,country\
         {cc}\
         &limit=500",
        db.url
    );
    // Fetch filtered places for display
    let tc = if !ft.is_empty() { format!("&place_type=eq.{ft}") } else { String::new() };
    let url_filtered = format!(
        "{}/rest/v1/places?active=eq.true\
         &select=name,place_type,city,country,address,hours_open,website,\
         booking_link,bear_popular,bear_night_schedule,inclusion_flag_codes\
         {tc}{cc}\
         &order=bear_popular.desc.nullslast,country.asc,city.asc&limit=200",
        db.url
    );
    let (all_res, filtered_res) = tokio::join!(
        db.get_json::<Vec<serde_json::Value>>(&url_all),
        db.get_json::<Vec<serde_json::Value>>(&url_filtered),
    );
    let all_places      = all_res.unwrap_or_default();
    let places          = filtered_res.unwrap_or_default();

    // Count per type across all (country-filtered) places
    let mut type_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for p in &all_places {
        let pt = p["place_type"].as_str().unwrap_or("other");
        *type_counts.entry(pt).or_insert(0) += 1;
    }
    let total_all = all_places.len();

    // Unique countries from ALL places
    let mut countries: Vec<String> = all_places.iter()
        .filter_map(|p| p["country"].as_str().map(String::from))
        .collect::<std::collections::HashSet<_>>().into_iter().collect();
    countries.sort();

    // Tab definitions — (slug, label, icon)
    let type_tabs: &[(&str, &str, &str)] = &[
        ("",                "All",      "🗺"),
        ("bar",             "Bar",      "🍺"),
        ("leather-bar",     "Leather",  "🧤"),
        ("sauna-bathhouse", "Sauna",    "♨"),
        ("campground",      "Camp",     "🏕"),
        ("party-venue",     "Party",    "🎉"),
        ("resort",          "Resort",   "🛖"),
        ("hotel",           "Hotel",    "🏨"),
        ("cruise-ship",     "Cruise",   "🚢"),
    ];

    let tabs_html: String = type_tabs.iter().filter_map(|(slug, label, icon)| {
        let count = if slug.is_empty() { total_all } else { *type_counts.get(*slug).unwrap_or(&0) };
        if count == 0 && !slug.is_empty() { return None; }  // hide empty tabs
        let on     = *slug == ft;
        let bg     = if on { ORANGE } else { OFF_WHITE };
        let fg     = if on { "#fff" }  else { BROWN };
        let border = if on { ORANGE }  else { TAN };
        let fw     = if on { "700" }   else { "400" };
        let country_qs = if !fc.is_empty() { format!("&place_country={fc}") } else { String::new() };
        Some(format!(
            "<a href=\"/?zone=places&place_type={slug}&lang={lang}{cqs}\" \
               style=\"display:inline-flex;flex-direction:column;align-items:center;\
                       gap:1px;text-decoration:none;padding:7px 10px;\
                       border-radius:12px;border:1px solid {border};\
                       background:{bg};color:{fg};font-weight:{fw};\
                       white-space:nowrap;min-width:52px\">\
              <span style=\"font-size:14px;line-height:1\">{icon}</span>\
              <span style=\"font-size:10px\">{label}</span>\
              <span style=\"font-size:9px;opacity:.8\">{count}</span>\
            </a>",
            cqs = country_qs,
        ))
    }).collect();

    // Country dropdown
    let ctry_opts: Vec<(String, String)> = std::iter::once(("".to_string(), "🌍 All regions".to_string()))
        .chain(countries.iter().map(|c| (c.clone(), c.clone())))
        .collect();
    let ctry_sel: String = ctry_opts.iter().map(|(v, l)| {
        let sel = if v.as_str() == fc { " selected" } else { "" };
        format!("<option value=\"{v}\"{sel}>{l}</option>")
    }).collect();
    let sel_style = format!(
        "font-size:12px;padding:6px 12px;border-radius:20px;\
         border:1px solid {TAN};background:{OFF_WHITE};color:{DARK};\
         font-family:inherit"
    );

    // Place type label for the active filter
    let type_label = type_tabs.iter()
        .find(|(s, _, _)| *s == ft)
        .map(|(_, l, _)| *l)
        .unwrap_or("All");
    let region_label = if fc.is_empty() { "worldwide".to_string() } else { fc.to_string() };

    // Region grouping definitions (same optgroups as coming-up country selector)
    let place_regions: &[(&str, &[&str])] = &[
        ("North America",        &["Canada","USA","Mexico","Puerto Rico"]),
        ("Europe",               &["Belgium","Czech Republic","Estonia","France","Germany",
                                    "Iceland","Ireland","Italy","Luxembourg","Netherlands",
                                    "Norway","Poland","Portugal","Scotland","Spain","Sweden",
                                    "Switzerland","UK"]),
        ("Asia Pacific",         &["Australia","Japan","New Zealand","South Korea","Taiwan","Thailand"]),
        ("Latin America",        &["Argentina","Brazil","Chile","Colombia"]),
        ("Africa & Middle East", &["Egypt","Morocco","South Africa"]),
    ];

    // Helper: which region does a country belong to?
    let region_for = |country: &str| -> &'static str {
        for (region, countries) in place_regions {
            if countries.contains(&country) { return region; }
        }
        "Other"
    };

    // Group places by type for display when "All" is selected + no country, else by type or flat
    let items_html: String = if ft.is_empty() && fc.is_empty() {
        // All types, all countries — group by region, then by type within region
        let mut out = String::new();
        for (region, countries) in place_regions {
            let region_places: Vec<&serde_json::Value> = places.iter()
                .filter(|p| {
                    let c = p["country"].as_str().unwrap_or("");
                    countries.contains(&c)
                })
                .collect();
            if region_places.is_empty() { continue; }

            out.push_str(&format!(
                "<div style=\"font-size:12px;font-weight:700;color:{BROWN};margin:18px 0 8px;\
                  border-left:3px solid {ORANGE};padding-left:8px\">{region} \
                  <span style=\"font-weight:400;color:{MID};font-size:11px\">({n})</span></div>",
                n = region_places.len(),
            ));

            // Sub-group by type within region
            for (slug, label, icon) in type_tabs.iter() {
                if slug.is_empty() { continue; }
                let group: Vec<&serde_json::Value> = region_places.iter()
                    .filter(|p| p["place_type"].as_str().unwrap_or("") == *slug)
                    .cloned()
                    .collect();
                if group.is_empty() { continue; }
                out.push_str(&format!(
                    "<div style=\"font-size:10px;font-weight:600;text-transform:uppercase;\
                      letter-spacing:.08em;color:{MID};margin:10px 0 4px\">{icon} {label} ({n})</div>",
                    n = group.len(),
                ));
                for p in &group { out.push_str(&place_card(p)); }
            }

            // Any uncategorised
            let other: Vec<&serde_json::Value> = region_places.iter()
                .filter(|p| {
                    let pt = p["place_type"].as_str().unwrap_or("");
                    !type_tabs.iter().any(|(s, _, _)| !s.is_empty() && *s == pt)
                })
                .cloned()
                .collect();
            for p in &other { out.push_str(&place_card(p)); }
        }
        // Any countries not in the region list
        let other_places: Vec<&serde_json::Value> = places.iter()
            .filter(|p| {
                let c = p["country"].as_str().unwrap_or("");
                !place_regions.iter().any(|(_, cs)| cs.contains(&c))
            })
            .collect();
        if !other_places.is_empty() {
            out.push_str(&format!(
                "<div style=\"font-size:12px;font-weight:700;color:{BROWN};margin:18px 0 8px;\
                  border-left:3px solid {TAN};padding-left:8px\">Other</div>"));
            for p in &other_places { out.push_str(&place_card(p)); }
        }
        out
    } else if ft.is_empty() && !fc.is_empty() {
        // Specific country, all types — group by type
        type_tabs.iter().filter_map(|(slug, label, icon)| {
            if slug.is_empty() { return None; }
            let group: Vec<&serde_json::Value> = places.iter()
                .filter(|p| p["place_type"].as_str().unwrap_or("") == *slug)
                .collect();
            if group.is_empty() { return None; }
            let cards: String = group.iter().map(|p| place_card(p)).collect();
            Some(format!(
                "<div style=\"font-size:10px;font-weight:700;text-transform:uppercase;\
                  letter-spacing:.1em;color:{MID};margin:16px 0 6px\">{icon} {label} ({n})</div>{cards}",
                n = group.len(),
            ))
        }).collect()
    } else {
        // Specific type filter — flat list
        places.iter().map(|p| place_card(p)).collect()
    };

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:8px\">\
          Bear Venues <span style=\"font-size:13px;font-weight:400;color:{MID}\">{type_label} · {region_label}</span></h1>\
        \
        <div style=\"overflow-x:auto;-webkit-overflow-scrolling:touch;margin-bottom:10px\">\
          <div style=\"display:inline-flex;gap:6px;padding-bottom:4px;min-width:min-content\">\
            {tabs_html}\
          </div>\
        </div>\
        \
        <form method=\"get\" action=\"/\" style=\"margin-bottom:12px;display:flex;align-items:center;gap:8px\">\
          <input type=\"hidden\" name=\"zone\" value=\"places\">\
          <input type=\"hidden\" name=\"lang\" value=\"{lang}\">\
          {type_hidden}\
          <select name=\"place_country\" onchange=\"this.form.submit()\" style=\"{sel_style}\">\
            {ctry_sel}</select>\
          <span style=\"font-size:11px;color:{MID};white-space:nowrap\">{n} venues</span>\
        </form>\
        \
        {items_html}",
        type_hidden = if !ft.is_empty() {
            format!("<input type=\"hidden\" name=\"place_type\" value=\"{ft}\">")
        } else { String::new() },
        n = places.len(),
    );
    Html(shell("Places", "Bear bars, saunas, campgrounds worldwide.", "places", &body, lang)).into_response()
}

fn place_card(p: &serde_json::Value) -> String {
    let name  = p["name"].as_str().unwrap_or("");
    let ptype = p["place_type"].as_str().unwrap_or("");
    let city  = p["city"].as_str().unwrap_or("");
    let ctry  = p["country"].as_str().unwrap_or("");
    let addr  = p["address"].as_str().unwrap_or("");
    let hours = p["hours_open"].as_str().unwrap_or("");
    let site  = p["website"].as_str().unwrap_or("");
    let book  = p["booking_link"].as_str().unwrap_or("");
    let bn    = p["bear_night_schedule"].as_str().unwrap_or("");
    let pop   = p["bear_popular"].as_bool().unwrap_or(false);
    let fs: Vec<String> = p["inclusion_flag_codes"].as_array()
        .map(|v| v.iter().filter_map(|s| s.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let site_html = if !site.is_empty() && site != "#" {
        format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>")
    } else { String::new() };
    let book_html = if !book.is_empty() && book != "#" {
        format!("<a href=\"{book}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Book</a>")
    } else { String::new() };
    card(&format!(
        "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
          <div style=\"flex:1;min-width:0\">\
            <div style=\"font-weight:600;font-size:14px\">{name}{pop_icon}\
              <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ptype}</span>\
            </div>\
            <div style=\"font-size:12px;color:{MID}\">{city}{sep}{ctry}</div>\
            {addr_h}{hours_h}{bn_h}\
            <div style=\"margin-top:4px\">{fhtml}</div>\
          </div>\
          <div style=\"display:flex;flex-direction:column;gap:6px\">{site_html}{book_html}</div>\
        </div>",
        sep      = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
        pop_icon = if pop { " 🐻" } else { "" },
        addr_h   = if !addr.is_empty() {
            format!("<div style=\"font-size:11px;color:{MID}\">{addr}</div>")
        } else { String::new() },
        hours_h  = if !hours.is_empty() {
            format!("<div style=\"font-size:11px;color:{GOLD}\">🕐 {}</div>",
                hours.chars().take(60).collect::<String>())
        } else { String::new() },
        bn_h     = if !bn.is_empty() {
            format!("<div style=\"font-size:11px;color:{ORANGE}\">🐻 {}</div>",
                bn.chars().take(80).collect::<String>())
        } else { String::new() },
        fhtml    = flags(&fs),
    ))
}

async fn zone_events(db: SupabaseClient, month: Option<u32>, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/events?active=eq.true&status=neq.past\
         &select=name,city,country,start_date,end_date,type,hot,link,inclusion_flag_codes\
         &order=start_date.asc&limit=100",
        db.url
    );
    let events: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = events.iter().filter(|e| {
        if let Some(mn) = month {
            e["start_date"].as_str().and_then(|d| extract_month(d)) == Some(mn)
        } else { true }
    }).map(|ev| {
        let name  = ev["name"].as_str().unwrap_or("");
        let city  = ev["city"].as_str().unwrap_or("");
        let ctry  = ev["country"].as_str().unwrap_or("");
        let start = ev["start_date"].as_str().unwrap_or("");
        let link  = ev["link"].as_str().unwrap_or("");
        let hot   = ev["hot"].as_bool().unwrap_or(false);
        let fs    = ev_flags(ev);
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}{h}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}, {ctry} · {start}</div>\
                <div style=\"margin-top:5px\">{fhtml}</div>\
              </div>\
              {link_html}\
            </div>",
            h     = if hot { " 🔥" } else { "" },
            fhtml = flags(&fs),
        ))
    }).collect();
    let body = format!(
        "<div style=\"display:flex;justify-content:space-between;align-items:center;margin-bottom:16px\">\
          <h1 style=\"font-size:18px;font-weight:700;color:{BROWN}\">Bear Events</h1>\
          <a href=\"/api/events/ical.ics\" class=\"btn-g\">📅 Subscribe</a>\
        </div>{items}"
    );
    Html(shell("Events", "Bear events worldwide.", "now", &body, lang)).into_response()
}

async fn zone_clubs(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/clubs?active=eq.true\
         &select=name,city,country,club_type,description,website,founded_year\
         &order=country.asc,name.asc&limit=100",
        db.url
    );
    let clubs: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = clubs.iter().map(|c| {
        let name  = c["name"].as_str().unwrap_or("");
        let city  = c["city"].as_str().unwrap_or("");
        let ctry  = c["country"].as_str().unwrap_or("");
        let yr    = c["founded_year"].as_i64().map(|y| format!(" (est. {y})")).unwrap_or_default();
        let desc  = c["description"].as_str().unwrap_or("");
        let site  = c["website"].as_str().unwrap_or("");
        let site_html = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}{yr}</div>\
                <div style=\"font-size:12px;color:{MID}\">{city}, {ctry}</div>\
                {desc_h}\
              </div>\
              {site_html}\
            </div>",
            desc_h = if !desc.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:4px\">{}</div>",
                    desc.chars().take(100).collect::<String>())
            } else { String::new() },
        ))
    }).collect();
    let page_clubs_title = i18n::t(i18n::translations(), lang, "page.clubs.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">{page_clubs_title}</h1>{items}"
    );
    Html(shell("Clubs", "Bear clubs worldwide.", "archive", &body, lang)).into_response()
}

async fn zone_titles(db: SupabaseClient, lang: &str) -> Response {
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
        db.get_json::<Vec<serde_json::Value>>(&url_comps),
        db.get_json::<Vec<serde_json::Value>>(&url_holders),
        db.get_json::<Vec<serde_json::Value>>(&url_clubs),
    );
    let comps   = comps_res.unwrap_or_default();
    let holders = holders_res.unwrap_or_default();
    let clubs   = clubs_res.unwrap_or_default();

    // Index clubs by id
    let club_map: std::collections::HashMap<i64, (&str, &str)> = clubs.iter()
        .filter_map(|c| {
            let id   = c["id"].as_i64()?;
            let name = c["name"].as_str().unwrap_or("");
            let site = c["website"].as_str().unwrap_or("");
            Some((id, (name, site)))
        })
        .collect();

    // Group holders by competition_id
    let mut holders_by_comp: std::collections::HashMap<i64, Vec<&serde_json::Value>> = std::collections::HashMap::new();
    for h in &holders {
        if let Some(cid) = h["competition_id"].as_i64() {
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
        let scope_comps: Vec<&serde_json::Value> = comps.iter()
            .filter(|c| c["scope"].as_str().unwrap_or("") == scope)
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
            let cid     = comp["id"].as_i64().unwrap_or(0);
            let cname   = comp["name"].as_str().unwrap_or("");
            let ccountry = comp["country"].as_str().unwrap_or("");
            let ccity   = comp["city"].as_str().unwrap_or("");
            let csite   = comp["website"].as_str().unwrap_or("#");
            let cfounded = comp["founded_year"].as_i64().unwrap_or(0);
            let club_id = comp["owning_club_id"].as_i64().unwrap_or(0);
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
            let comp_holders = holders_by_comp.get(&cid)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let holder_rows: String = comp_holders.iter().take(12).map(|h| {
                let name   = h["holder_name"].as_str().unwrap_or("");
                let year   = h["year"].as_i64().unwrap_or(0);
                let hcity  = h["city"].as_str().unwrap_or("");
                let hctry  = h["country"].as_str().unwrap_or("");
                let status = h["holder_status"].as_str().unwrap_or("");
                let fs: Vec<String> = h["inclusion_flag_codes"].as_array()
                    .map(|v| v.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                    .unwrap_or_default();
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

async fn zone_creators(db: SupabaseClient, lang: &str) -> Response {
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
    let creators: Vec<serde_json::Value>  = creators_res.unwrap_or_default();
    let media_all: Vec<serde_json::Value> = media_res.unwrap_or_default();
    let stores: Vec<serde_json::Value>    = stores_res.unwrap_or_default();

    // Group media by creator_id
    let mut media_by_creator: std::collections::HashMap<i64, Vec<&serde_json::Value>> =
        std::collections::HashMap::new();
    for m in &media_all {
        if let Some(cid) = m["creator_id"].as_i64() {
            media_by_creator.entry(cid).or_default().push(m);
        }
    }

    let creator_cards: String = creators.iter().map(|c| {
        let id    = c["id"].as_i64().unwrap_or(0);
        let name  = c["name"].as_str().unwrap_or("");
        let ctype = c["creator_type"].as_str().unwrap_or("creator");
        let city  = c["city"].as_str().unwrap_or("");
        let ctry  = c["country"].as_str().unwrap_or("");
        let bio   = c["bio"].as_str().unwrap_or("");
        let site  = c["website"].as_str().unwrap_or("");
        let sp    = c["spotify_link"].as_str().unwrap_or("");
        let yt    = c["youtube_link"].as_str().unwrap_or("");
        let bc    = c["bandcamp_link"].as_str().unwrap_or("");
        let etsy  = c["etsy_link"].as_str().unwrap_or("");
        let ig    = c["instagram"].as_str().unwrap_or("");

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
                    let mtitle  = m["title"].as_str().unwrap_or("");
                    let mtype   = m["media_type"].as_str().unwrap_or("");
                    let myear   = m["year"].as_i64()
                        .map(|y| format!(" ({y})"))
                        .unwrap_or_default();
                    let mlink   = m["link"].as_str().unwrap_or("");
                    let mstream = m["streaming_link"].as_str().unwrap_or("");
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
        let sname   = st["name"].as_str().unwrap_or("");
        let stype   = st["type"].as_str().unwrap_or("");
        let slink   = st["link"].as_str().unwrap_or("");
        let sdesc   = st["description"].as_str().unwrap_or("");
        let owned   = st["bear_owned"].as_bool().unwrap_or(false);
        let szinc   = st["size_inclusive"].as_bool().unwrap_or(false);
        let sglobal = st["ships_global"].as_bool().unwrap_or(false);
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

async fn zone_campaigns(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/campaigns?active=eq.true&privacy_mode=eq.false\
         &select=name,org,description,link,goal,raised,currency&order=name.asc",
        db.url
    );
    let campaigns: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = campaigns.iter().map(|c| {
        let name   = c["name"].as_str().unwrap_or("");
        let org    = c["org"].as_str().unwrap_or("");
        let desc   = c["description"].as_str().unwrap_or("");
        let link   = c["link"].as_str().unwrap_or("");
        let goal   = c["goal"].as_f64();
        let raised = c["raised"].as_f64();
        let curr   = c["currency"].as_str().unwrap_or("USD");
        let progress = match (raised, goal) {
            (Some(r), Some(g)) if g > 0.0 => {
                let pct = ((r / g) * 100.0).min(100.0) as u64;
                format!(
                    "<div style=\"margin-top:8px\">\
                      <div style=\"display:flex;justify-content:space-between;font-size:10px;\
                                  color:{MID};margin-bottom:4px\">\
                        <span>{r:.0} {curr} raised</span><span>goal: {g:.0}</span>\
                      </div>\
                      <div style=\"height:4px;border-radius:999px;background:{TAN}\">\
                        <div style=\"height:4px;border-radius:999px;background:{GOLD};width:{pct}%\"></div>\
                      </div>\
                    </div>"
                )
            },
            _ => String::new(),
        };
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-g\">Donate</a>")
        } else { String::new() };
        card(&format!(
            "<div>\
              <div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
                <div style=\"flex:1;min-width:0\">\
                  <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                  <div style=\"font-size:11px;color:{MID};margin-top:2px\">{org}</div>\
                  {desc_h}\
                </div>\
                {link_html}\
              </div>\
              {progress}\
            </div>",
            desc_h = if !desc.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>",
                    desc.chars().take(160).collect::<String>())
            } else { String::new() },
        ))
    }).collect();
    let page_campaigns_title = i18n::t(i18n::translations(), lang, "page.campaigns.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">{page_campaigns_title}</h1>{items}"
    );
    Html(shell("Campaigns", "Community campaigns.", "now", &body, lang)).into_response()
}

// ── ZONE: ICAL EXPORT ─────────────────────────────────────────────────────
async fn zone_ical(lang: &str) -> Response {
    let body = format!(
        "<div style=\"text-align:center;padding:24px 0 16px\">\
          <div style=\"font-size:48px;margin-bottom:8px\">📅</div>\
          <h1 style=\"font-size:20px;font-weight:700;color:{BROWN};margin-bottom:6px\">\
            Subscribe to Bear Events</h1>\
          <p style=\"font-size:13px;color:{MID};max-width:360px;margin:0 auto 20px;\
             line-height:1.6\">\
            Add the full bear events calendar to Apple Calendar, \
            Google Calendar, or Outlook — it auto-updates as new events are added.</p>\
        </div>\
        \
        <div class=\"card\" style=\"margin-bottom:12px\">\
          <div style=\"font-size:11px;font-weight:600;color:{MID};text-transform:uppercase;\
            letter-spacing:.08em;margin-bottom:10px\">All upcoming events</div>\
          <div style=\"display:flex;gap:8px;align-items:center;flex-wrap:wrap\">\
            <code style=\"flex:1;font-size:11px;background:{OFF_WHITE};border:1px solid {TAN};\
              border-radius:8px;padding:8px 10px;word-break:break-all;color:{DARK}\">\
              https://bearings.app/api/events/ical.ics</code>\
            <a href=\"/api/events/ical.ics\" class=\"btn-o\" download=\"bearings-events.ics\">\
              Download</a>\
          </div>\
          <div style=\"font-size:11px;color:{MID};margin-top:8px\">\
            In Apple Calendar: File → New Calendar Subscription → paste the URL above.<br>\
            In Google Calendar: Other calendars (+) → From URL → paste and add.</div>\
        </div>\
        \
        <div class=\"card\" style=\"margin-bottom:12px\">\
          <div style=\"font-size:11px;font-weight:600;color:{MID};text-transform:uppercase;\
            letter-spacing:.08em;margin-bottom:10px\">Filter by country</div>\
          <div style=\"display:flex;flex-direction:column;gap:8px\">\
            {country_rows}\
          </div>\
        </div>\
        \
        <div class=\"card\" style=\"text-align:center\">\
          <div style=\"font-size:12px;color:{MID};margin-bottom:6px\">\
            Calendar feeds update automatically. No account needed.</div>\
          <a href=\"/?zone=coming-up&lang={lang}\" class=\"btn-t\">← Back to events</a>\
        </div>",
        country_rows = [
            ("Worldwide", ""),
            ("Canada",    "Canada"),
            ("USA",       "USA"),
            ("Germany",   "Germany"),
            ("Netherlands","Netherlands"),
            ("UK",        "UK"),
            ("Australia", "Australia"),
            ("Brazil",    "Brazil"),
            ("Spain",     "Spain"),
        ].iter().map(|(label, c)| {
            let qs = if c.is_empty() { String::new() } else { format!("?country={c}") };
            format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:center;\
                  padding:4px 0;border-bottom:1px solid {OFF_WHITE}\">\
                  <span style=\"font-size:12px;color:{DARK}\">{label}</span>\
                  <a href=\"/api/events/ical.ics{qs}\" class=\"btn-t\" style=\"font-size:11px\"\
                    download>ics</a>\
                </div>",
                label = label,
                qs    = qs,
            )
        }).collect::<String>(),
    );
    Html(shell("iCal Export", "Subscribe to bear events in your calendar.", "ical", &body, lang)).into_response()
}

async fn zone_digital(db: SupabaseClient, lang: &str) -> Response {
    let url = format!(
        "{}/rest/v1/digital_spaces?active=eq.true\
         &select=name,space_type,description,url,member_count,\
         instagram,tiktok_handle,bluesky_handle,youtube_handle\
         &order=space_type.asc,name.asc&limit=100",
        db.url
    );
    let spaces: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = spaces.iter().map(|s| {
        let name    = s["name"].as_str().unwrap_or("");
        let stype   = s["space_type"].as_str().unwrap_or("");
        let desc    = s["description"].as_str().unwrap_or("");
        let url_s   = s["url"].as_str().unwrap_or("");
        let members = s["member_count"].as_i64();
        let ig      = s["instagram"].as_str().unwrap_or("");
        let tt      = s["tiktok_handle"].as_str().unwrap_or("");
        let bs      = s["bluesky_handle"].as_str().unwrap_or("");
        let yt      = s["youtube_handle"].as_str().unwrap_or("");
        let mut sc  = Vec::new();
        if !ig.is_empty() { sc.push(format!("<a href=\"https://instagram.com/{ig}\" target=\"_blank\" class=\"badge\" style=\"background:#E1306C;color:#fff\">IG</a>")); }
        if !tt.is_empty() { sc.push(format!("<a href=\"https://tiktok.com/@ {tt}\" target=\"_blank\" class=\"badge\" style=\"background:#000;color:#fff\">TT</a>")); }
        if !bs.is_empty() { sc.push(format!("<a href=\"https://bsky.app/profile/{bs}\" target=\"_blank\" class=\"badge\" style=\"background:#0085ff;color:#fff\">BS</a>")); }
        if !yt.is_empty() { sc.push(format!("<a href=\"https://youtube.com/{yt}\" target=\"_blank\" class=\"badge\" style=\"background:#FF0000;color:#fff\">YT</a>")); }
        let link_html = if !url_s.is_empty() && url_s != "#" {
            format!("<a href=\"{url_s}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Visit</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {stype}</span>\
                  {m_html}\
                </div>\
                {desc_h}\
                <div style=\"margin-top:5px\">{sc_html}</div>\
              </div>\
              {link_html}\
            </div>",
            m_html  = members.map(|m| format!(
                "<span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{m} members</span>"
            )).unwrap_or_default(),
            desc_h  = if !desc.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:3px;line-height:1.5\">{}</div>",
                    desc.chars().take(120).collect::<String>())
            } else { String::new() },
            sc_html = sc.join(" "),
        ))
    }).collect();
    let page_digital_title = i18n::t(i18n::translations(), lang, "page.digital.title");
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">{page_digital_title}</h1>{items}"
    );
    Html(shell("Digital Spaces", "Bear digital spaces.", "now", &body, lang)).into_response()
}

// ── LEGACY WRAPPERS (kept so existing routes in main.rs still compile) ──────
// These delegate to zone functions. Remove once Gaspar confirms ?zone= routing.

pub async fn now_page           (State(db): State<SupabaseClient>) -> Response { zone_now(db, "en").await }
pub async fn coming_up_page     (State(db): State<SupabaseClient>) -> Response { zone_coming_up(db, None, None, None, "en").await }
pub async fn history_page       (State(db): State<SupabaseClient>) -> Response { zone_archive(db, None, None, "en").await }
pub async fn bear_future_page   (State(db): State<SupabaseClient>) -> Response { zone_future(db, "en").await }
pub async fn events_page        (State(db): State<SupabaseClient>) -> Response { zone_events(db, None, "en").await }
pub async fn places_page        (State(db): State<SupabaseClient>) -> Response { zone_places(db, None, None, "en").await }
pub async fn clubs_page         (State(db): State<SupabaseClient>) -> Response { zone_clubs(db, "en").await }
pub async fn titles_page        (State(db): State<SupabaseClient>) -> Response { zone_titles(db, "en").await }
pub async fn creators_page      (State(db): State<SupabaseClient>) -> Response { zone_creators(db, "en").await }
pub async fn campaigns_page     (State(db): State<SupabaseClient>) -> Response { zone_campaigns(db, "en").await }
pub async fn digital_spaces_page(State(db): State<SupabaseClient>) -> Response { zone_digital(db, "en").await }
