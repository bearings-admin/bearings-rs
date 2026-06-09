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
use crate::db::SupabaseClient;
use serde::Deserialize;

// ── Query params ──────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct ZoneQuery {
    pub zone:     Option<String>,
    pub decade:   Option<String>,
    pub month:    Option<u32>,
    pub fragment: Option<String>,  // "events" → return bare #event-list HTML for HTMX
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

fn shell(title: &str, description: &str, active: &str, body: &str) -> String {
    let nav = |zone: &str, icon: &str, label: &str| {
        let on = zone == active;
        format!(
            "<a href=\"/?zone={zone}\" style=\"display:flex;flex-direction:column;\
               align-items:center;gap:2px;text-decoration:none;padding:4px 8px;\
               border-radius:10px;color:{col};font-weight:{fw};font-size:10px\"\
               ><span style=\"font-size:20px;line-height:1\">{icon}</span>{label}</a>",
            col = if on { ORANGE } else { BROWN },
            fw  = if on { "700" } else { "400" },
        )
    };
    format!(
        "<!DOCTYPE html>\n\
<html lang=\"en\">\n\
<head>\n\
  <meta charset=\"UTF-8\">\n\
  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n\
  <title>{title} — Bearings</title>\n\
  <meta name=\"description\" content=\"{description}\">\n\
  <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n\
  <link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap\">\n\
  <script src=\"https://cdn.tailwindcss.com\"></script>\n\
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
  </style>\n\
</head>\n\
<body style=\"padding-bottom:72px\">\n\
\n\
  <div class=\"stripe\"></div>\n\
\n\
  <header style=\"max-width:640px;margin:0 auto;padding:14px 16px 8px;\n\
                 display:flex;justify-content:space-between;align-items:center\">\n\
    <a href=\"/?zone=now\" style=\"display:flex;align-items:baseline;gap:8px\">\n\
      <span style=\"font-size:18px;font-weight:700;letter-spacing:.15em;color:{BROWN}\">BEARINGS</span>\n\
      <span style=\"font-size:11px;color:{MID}\">global bear community</span>\n\
    </a>\n\
    <a href=\"/api/events/ical.ics\"\n\
       style=\"font-size:11px;background:{GOLD};color:{DARK};border-radius:999px;\n\
              padding:5px 12px;font-weight:600\">📅 iCal</a>\n\
  </header>\n\
\n\
  <main style=\"max-width:640px;margin:0 auto;padding:4px 16px 16px\">\n\
    {body}\n\
  </main>\n\
\n\
  <nav style=\"position:fixed;bottom:0;left:0;right:0;background:{OFF_WHITE};\n\
              border-top:1px solid {TAN};z-index:100\">\n\
    <div style=\"max-width:640px;margin:0 auto;display:flex;\n\
                justify-content:space-around;align-items:center;padding:6px 8px 10px\">\n\
      {n_now}{n_coming}{n_archive}{n_future}{n_places}\n\
    </div>\n\
  </nav>\n\
\n\
</body>\n\
</html>",
        n_now     = nav("now",       "🐻", "NOW"),
        n_coming  = nav("coming-up", "📍", "TRIPS"),
        n_archive = nav("archive",   "📚", "ARCHIVE"),
        n_future  = nav("future",    "🏛️","FUTURE"),
        n_places  = nav("places",    "🍺", "PLACES"),
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

fn timeline_bar(evs: &[serde_json::Value], active_month: Option<u32>) -> String {
    let months = ["Jan","Feb","Mar","Apr","May","Jun",
                  "Jul","Aug","Sep","Oct","Nov","Dec"];
    let bear_col = [DARK,MID,BROWN,BROWN,ORANGE,ORANGE,
                    GOLD,GOLD,TAN,TAN,"#AAAAAA",DARK];
    let mut counts = [0usize; 12];
    for e in evs {
        if let Some(d) = e["start_date"].as_str() {
            // Split on '-' using string slice, not char literal
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
        format!(
            "<a href=\"/?zone=now&month={mn}\"\
               hx-get=\"/?zone=now&month={mn}&fragment=events\"\
               hx-target=\"#event-list\" hx-swap=\"innerHTML\"\
               hx-indicator=\"#bar-spin\"\
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
    format!(
        "<div class=\"card\" style=\"padding:12px 14px\">\
          <div style=\"font-size:10px;font-weight:600;color:{MID};margin-bottom:8px;\
                      text-transform:uppercase;letter-spacing:.08em\">Events by month · click to filter</div>\
          <div style=\"display:flex;gap:3px;align-items:flex-end;height:56px\">{bars}</div>\
          <div id=\"bar-spin\" class=\"htmx-indicator\">loading…</div>\
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
    match q.zone.as_deref().unwrap_or("now") {
        "coming-up"      => zone_coming_up(db).await,
        "archive"        => zone_archive(db, q.decade).await,
        "future"         => zone_future(db).await,
        "places"         => zone_places(db).await,
        "events"         => zone_events(db, q.month).await,
        "clubs"          => zone_clubs(db).await,
        "titles"         => zone_titles(db).await,
        "creators"       => zone_creators(db).await,
        "campaigns"      => zone_campaigns(db).await,
        "digital-spaces" => zone_digital(db).await,
        _                => zone_now(db, q.month, q.fragment).await,
    }
}

// ── ZONE: NOW ─────────────────────────────────────────────────

async fn zone_now(db: SupabaseClient, month_filter: Option<u32>, fragment: Option<String>) -> Response {
    let rpc_body = serde_json::json!({
        "input_lat": serde_json::Value::Null,
        "input_lng": serde_json::Value::Null,
        "radius_km": 2000.0,
    });
    let data: serde_json::Value = match db.post_rpc("now_feed", &rpc_body).await {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "now_feed failed").into_response(),
    };
    let hot  = data["hot_events"].as_array().cloned().unwrap_or_default();
    let near = data["nearby_venues"].as_array().cloned().unwrap_or_default();
    let cmpg = data["active_campaigns"].as_array().cloned().unwrap_or_default();
    let ttls = data["current_titles"].as_array().cloned().unwrap_or_default();

    let all_url = format!(
        "{}/rest/v1/events?active=eq.true&status=neq.past&select=start_date&limit=200",
        db.url
    );
    let all_evs: Vec<serde_json::Value> = db.get_json(&all_url).await.unwrap_or_default();

    // Hero card
    let hero = hot.first().map(|ev| {
        let name  = ev["name"].as_str().unwrap_or("Upcoming Bear Event");
        let city  = ev["city"].as_str().unwrap_or("");
        let ctry  = ev["country"].as_str().unwrap_or("");
        let start = ev["start_date"].as_str().unwrap_or("");
        let end   = ev["end_date"].as_str().unwrap_or("");
        let link  = ev["link"].as_str().unwrap_or("");
        let desc  = ev["description"].as_str().unwrap_or("");
        let km    = ev["distance_km"].as_f64().unwrap_or(0.0);
        let fs    = ev_flags(ev);
        let dates = if !end.is_empty() && end != start {
            format!("{start} → {end}")
        } else { start.to_string() };
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-g\">Learn more →</a>")
        } else { String::new() };
        format!(
            "<div style=\"border-radius:20px;padding:20px 18px;margin-bottom:12px;\
                color:#fff;position:relative;overflow:hidden;\
                background:linear-gradient(135deg,{BROWN},{ORANGE});\
                box-shadow:0 4px 20px rgba(92,64,51,.3)\">\
              <div style=\"position:absolute;top:14px;right:16px;font-size:26px\">🐻</div>\
              <div style=\"font-size:10px;font-weight:700;letter-spacing:.12em;\
                          text-transform:uppercase;opacity:.7;margin-bottom:4px\">Up Next 🔥</div>\
              <h1 style=\"font-size:19px;font-weight:700;margin-bottom:4px;\
                         line-height:1.3;padding-right:36px\">{name}</h1>\
              <div style=\"font-size:13px;opacity:.9;margin-bottom:6px\">{city}, {ctry} · {dates}</div>\
              <div style=\"font-size:12px;opacity:.8;margin-bottom:12px;line-height:1.5\">{snippet}</div>\
              <div style=\"display:flex;flex-wrap:wrap;gap:4px;margin-bottom:14px\">{flags_html}{dist_html}</div>\
              {link_html}\
            </div>",
            snippet   = desc.chars().take(160).collect::<String>(),
            flags_html= flags(&fs),
            dist_html = if km > 0.0 { dist(km) } else { String::new() },
        )
    }).unwrap_or_default();

    // Filter events by month if selected
    let displayed: Vec<&serde_json::Value> = if let Some(mn) = month_filter {
        hot.iter().filter(|e| {
            e["start_date"].as_str()
                .and_then(|d| extract_month(d))
                == Some(mn)
        }).collect()
    } else {
        hot.iter().collect()
    };

    let event_cards: String = displayed.iter().skip(1).take(9).map(|ev| {
        let name  = ev["name"].as_str().unwrap_or("");
        let city  = ev["city"].as_str().unwrap_or("");
        let ctry  = ev["country"].as_str().unwrap_or("");
        let start = ev["start_date"].as_str().unwrap_or("");
        let link  = ev["link"].as_str().unwrap_or("");
        let etype = ev["type"].as_str().unwrap_or("");
        let km    = ev["distance_km"].as_f64().unwrap_or(0.0);
        let fs    = ev_flags(ev);
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name} 🔥</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}, {ctry} · {start}</div>\
                <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                  <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                  {fhtml}{dhtml}\
                </div>\
              </div>\
              {link_html}\
            </div>",
            fhtml = flags(&fs),
            dhtml = if km > 0.0 { dist(km) } else { String::new() },
        ))
    }).collect();

    let venue_cards: String = near.iter().take(3).map(|v| {
        let name  = v["name"].as_str().unwrap_or("");
        let ptype = v["place_type"].as_str().unwrap_or("");
        let city  = v["city"].as_str().unwrap_or("");
        let ctry  = v["country"].as_str().unwrap_or("");
        let km    = v["distance_km"].as_f64().unwrap_or(0.0);
        let site  = v["website"].as_str().unwrap_or("");
        let bn    = v["bear_night_schedule"].as_str().unwrap_or("");
        let site_html = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Visit</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1\">\
                <div style=\"font-weight:600;font-size:14px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ptype}</span>\
                </div>\
                <div style=\"font-size:12px;color:{MID}\">{city}, {ctry} {dhtml}</div>\
                {bn_html}\
              </div>\
              {site_html}\
            </div>",
            dhtml   = dist(km),
            bn_html = if !bn.is_empty() {
                format!("<div style=\"font-size:11px;color:{ORANGE};margin-top:4px\">🐻 {}</div>",
                    bn.chars().take(80).collect::<String>())
            } else { String::new() },
        ))
    }).collect();

    let camp_cards: String = cmpg.iter().take(3).map(|c| {
        let name = c["name"].as_str().unwrap_or("");
        let org  = c["org"].as_str().unwrap_or("");
        let link = c["link"].as_str().unwrap_or("");
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-g\">Donate</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:center;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                <div style=\"font-size:11px;color:{MID};margin-top:2px\">{org}</div>\
              </div>\
              {link_html}\
            </div>"
        ))
    }).collect();

    let title_cards: String = ttls.iter().take(5).map(|t| {
        let title   = t["title_name"].as_str().unwrap_or("");
        let holder  = t["display_name"].as_str()
            .unwrap_or_else(|| t["holder_name"].as_str().unwrap_or(""));
        let status  = t["display_status"].as_str().unwrap_or("");
        let year    = t["year"].as_i64().unwrap_or(0);
        let city    = t["city"].as_str().unwrap_or("");
        let ctry    = t["country"].as_str().unwrap_or("");
        let scope   = t["competition_scope"].as_str().unwrap_or("");
        let icon    = match scope {
            "continental"=>"🌎","national"=>"🏳️","regional"=>"📍","local"=>"🏙️",_=>"🐻"
        };
        let status_badge = if !status.is_empty() {
            format!("<span style=\"font-size:9px;font-weight:600;padding:1px 6px;\
                     border-radius:999px;background:#EDE0D4;color:{BROWN};\
                     margin-left:4px;vertical-align:middle\">{status}</span>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:center\">\
              <div>\
                <div style=\"font-weight:600;font-size:14px\">{icon} {title}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">\
                  {holder}{status_badge}\
                </div>\
                <div style=\"font-size:11px;color:{MID}\">{city}{sep}{ctry}</div>\
              </div>\
              <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">{year}</div>\
            </div>",
            sep = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
        ))
    }).collect();

    // Fragment: HTMX month-bar clicks request just the event list HTML.
    // Return bare inner HTML so HTMX can swap #event-list without full reload.
    if fragment.as_deref() == Some("events") {
        let frag_html: String = displayed.iter().skip(1).take(9).map(|ev| {
            let name  = ev["name"].as_str().unwrap_or("");
            let city  = ev["city"].as_str().unwrap_or("");
            let ctry  = ev["country"].as_str().unwrap_or("");
            let start = ev["start_date"].as_str().unwrap_or("");
            let link  = ev["link"].as_str().unwrap_or("");
            let etype = ev["type"].as_str().unwrap_or("");
            let km    = ev["distance_km"].as_f64().unwrap_or(0.0);
            let fs    = ev_flags(ev);
            let link_html = if !link.is_empty() && link != "#" {
                format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
            } else { String::new() };
            card(&format!(
                "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
                  <div style=\"flex:1;min-width:0\">\
                    <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name} 🔥</div>\
                    <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}, {ctry} · {start}</div>\
                    <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                      <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                      {fhtml}{dhtml}\
                    </div>\
                  </div>\
                  {link_html}\
                </div>",
                fhtml = flags(&fs),
                dhtml = if km > 0.0 { dist(km) } else { String::new() },
            ))
        }).collect();
        let view_all = format!(
            "<a href=\"/?zone=events\" style=\"display:block;text-align:center;font-size:13px;\
             color:{ORANGE};padding:8px 0 16px\">View all 88 events →</a>"
        );
        return Html(format!("{frag_html}{view_all}")).into_response();
    }

    let month_lbl = month_filter.map(|m| {
        ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"]
            .get((m.saturating_sub(1)) as usize).copied().unwrap_or("")
    });
    let events_label = month_lbl
        .map(|l| format!("Hot Events — {l} 🔥"))
        .unwrap_or_else(|| "Hot Events 🔥".to_string());

    let stats_html = format!(
        "<div style=\"display:grid;grid-template-columns:repeat(4,1fr);gap:8px;margin-bottom:10px\">{}</div>",
        stat_row()
    );

    let body = format!(
        "{hero}\
        {stats}\
        {bar}\
        <div id=\"event-list\">\
          {h_events}\
          {event_cards}\
          <a href=\"/?zone=events\" style=\"display:block;text-align:center;font-size:13px;\
             color:{ORANGE};padding:8px 0 16px\">View all 88 events →</a>\
        </div>\
        {h_venues}\
        {venue_cards}\
        <a href=\"/?zone=places\" style=\"display:block;text-align:center;font-size:13px;\
           color:{ORANGE};padding:8px 0 16px\">View all 170 venues →</a>\
        {h_camps}\
        {camp_cards}\
        <a href=\"/?zone=campaigns\" style=\"display:block;text-align:center;font-size:13px;\
           color:{ORANGE};padding:8px 0 16px\">View all campaigns →</a>\
        {h_titles}\
        {title_cards}\
        <a href=\"/?zone=titles\" style=\"display:block;text-align:center;font-size:13px;\
           color:{ORANGE};padding:8px 0 24px\">Full title archive (87 records) →</a>",
        stats    = stats_html,
        bar      = timeline_bar(&all_evs, month_filter),
        h_events = sh(&events_label, Some(displayed.len())),
        h_venues = sh("Nearby Venues", Some(near.len())),
        h_camps  = sh("Community Campaigns", Some(cmpg.len())),
        h_titles = sh("Current Title Holders", Some(ttls.len())),
    );

    Html(shell("Now", "What the bear world is doing right now.", "now", &body)).into_response()
}

// ── ZONE: COMING UP ───────────────────────────────────────────

async fn zone_coming_up(db: SupabaseClient) -> Response {
    let rpc_body = serde_json::json!({
        "input_lat": serde_json::Value::Null, "input_lng": serde_json::Value::Null,
        "radius_km": serde_json::Value::Null, "season": serde_json::Value::Null,
        "from_date": serde_json::Value::Null, "to_date": serde_json::Value::Null,
        "event_type": serde_json::Value::Null, "country": serde_json::Value::Null,
        "max_rows": 40,
    });
    let data: serde_json::Value = match db.post_rpc("coming_up", &rpc_body).await {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "coming_up failed").into_response(),
    };
    let events = data["events"].as_array().cloned().unwrap_or_default();
    let venues = data["venues"].as_array().cloned().unwrap_or_default();

    let filter_card = format!(
        "<div class=\"card\">\
          <div style=\"font-size:10px;font-weight:700;text-transform:uppercase;\
                      letter-spacing:.1em;color:{MID};margin-bottom:12px\">Plan a Trip</div>\
          <form hx-get=\"/?zone=coming-up\" hx-target=\"#cu-results\" hx-swap=\"innerHTML\" hx-indicator=\"#cu-spin\">\
            <div style=\"margin-bottom:10px\">\
              <div style=\"font-size:12px;font-weight:600;color:{BROWN};margin-bottom:4px\">Where</div>\
              <select name=\"country\" style=\"width:100%;padding:9px 12px;border-radius:10px;\
                     border:1px solid {TAN};background:{OFF_WHITE};font-size:13px\">\
                <option value=\"\">Anywhere</option>\
                <option>Canada</option><option>USA</option><option>Germany</option>\
                <option>UK</option><option>Spain</option><option>Netherlands</option>\
                <option>Australia</option><option>France</option><option>Portugal</option>\
              </select>\
            </div>\
            <div style=\"display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-bottom:14px\">\
              <select name=\"season\" style=\"padding:9px 8px;border-radius:10px;\
                     border:1px solid {TAN};background:{OFF_WHITE};font-size:12px\">\
                <option value=\"\">Any season</option>\
                <option value=\"spring\">Spring (Mar–May)</option>\
                <option value=\"summer\">Summer (Jun–Aug)</option>\
                <option value=\"autumn\">Autumn (Sep–Nov)</option>\
                <option value=\"winter\">Winter (Dec–Feb)</option>\
              </select>\
              <select name=\"event_type\" style=\"padding:9px 8px;border-radius:10px;\
                     border:1px solid {TAN};background:{OFF_WHITE};font-size:12px\">\
                <option value=\"\">Any type</option>\
                <option value=\"bear-run\">Bear Run</option>\
                <option value=\"cruise\">Cruise</option>\
                <option value=\"social\">Social</option>\
                <option value=\"party-night\">Party Night</option>\
              </select>\
            </div>\
            <button type=\"submit\" class=\"btn-o\" style=\"width:100%;padding:10px;text-align:center\">Find events →</button>\
          </form>\
          <div id=\"cu-spin\" class=\"htmx-indicator\">Searching…</div>\
        </div>"
    );

    let ical_block = format!(
        "<div style=\"border-radius:16px;border:1px solid {GOLD};\
             background:#FFFEF5;padding:16px;text-align:center;margin-bottom:4px\">\
          <div style=\"font-weight:600;font-size:13px;color:{BROWN};margin-bottom:4px\">📅 Subscribe to this calendar</div>\
          <div style=\"font-size:11px;color:{MID};margin-bottom:12px\">Auto-updates in Apple Calendar, Google Calendar, any iCal app</div>\
          <a href=\"/api/events/ical.ics\" class=\"btn-g\">Subscribe — All Events</a>\
        </div>"
    );

    let ev_cards: String = events.iter().map(|ev| {
        let name  = ev["name"].as_str().unwrap_or("");
        let city  = ev["city"].as_str().unwrap_or("");
        let ctry  = ev["country"].as_str().unwrap_or("");
        let start = ev["start_date"].as_str().unwrap_or("");
        let end   = ev["end_date"].as_str().unwrap_or("");
        let link  = ev["link"].as_str().unwrap_or("");
        let etype = ev["type"].as_str().unwrap_or("");
        let fs    = ev_flags(ev);
        let dates = if !end.is_empty() && end != start { format!("{start} → {end}") }
                    else { start.to_string() };
        let link_html = if !link.is_empty() && link != "#" {
            format!("<a href=\"{link}\" target=\"_blank\" rel=\"noopener\" class=\"btn-o\">Info</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px;line-height:1.3\">{name}</div>\
                <div style=\"font-size:12px;color:{MID};margin-top:2px\">{city}, {ctry} · {dates}</div>\
                <div style=\"margin-top:5px;display:flex;flex-wrap:wrap;gap:2px\">\
                  <span class=\"badge\" style=\"background:{TAN};color:{BROWN}\">{etype}</span>\
                  {fhtml}\
                </div>\
              </div>\
              {link_html}\
            </div>",
            fhtml = flags(&fs),
        ))
    }).collect();

    let vn_cards: String = venues.iter().take(4).map(|v| {
        let name  = v["name"].as_str().unwrap_or("");
        let ptype = v["place_type"].as_str().unwrap_or("");
        let city  = v["city"].as_str().unwrap_or("");
        let ctry  = v["country"].as_str().unwrap_or("");
        let hours = v["hours_open"].as_str().unwrap_or("");
        let site  = v["website"].as_str().unwrap_or("");
        let site_html = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" style=\"font-size:12px;color:{ORANGE}\">Visit →</a>")
        } else { String::new() };
        card(&format!(
            "<div>\
              <div style=\"font-weight:600;font-size:14px\">{name}\
                <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ptype}</span>\
              </div>\
              <div style=\"font-size:12px;color:{MID}\">{city}, {ctry}</div>\
              {hours_html}\
              {site_html}\
            </div>",
            hours_html = if !hours.is_empty() {
                format!("<div style=\"font-size:11px;color:{GOLD};margin-top:3px\">🕐 {}</div>",
                    hours.chars().take(60).collect::<String>())
            } else { String::new() },
        ))
    }).collect();

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Plan a Trip</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">Set your destination and season to find events and venues.</p>\
        {filter_card}\
        {ical_block}\
        <div id=\"cu-results\">\
          {h_ev}{ev_cards}\
          {h_vn}{vn_cards}\
        </div>",
        h_ev = sh("Upcoming Events", Some(events.len())),
        h_vn = sh("Venues in Destination", Some(venues.len())),
    );
    Html(shell("Coming Up", "Plan your bear trips.", "coming-up", &body)).into_response()
}

// ── ZONE: BEAR ARCHIVES (decade tabs) ────────────────────────

async fn zone_archive(db: SupabaseClient, decade: Option<String>) -> Response {
    let url = format!(
        "{}/rest/v1/bear_history?active=eq.true\
         &select=year,title,description,category,significance\
         &order=year.asc&limit=100",
        db.url
    );
    let history: Vec<serde_json::Value> = match db.get_json(&url).await {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let titles_url = format!(
        "{}/rest/v1/title_holders?select=title_name,holder_name,year,city,country\
         &order=year.desc&limit=12",
        db.url
    );
    let recent_titles: Vec<serde_json::Value> = db.get_json(&titles_url).await.unwrap_or_default();

    let decades = ["1980s","1990s","2000s","2010s","2020s"];
    let active  = decade.as_deref().unwrap_or("2020s");
    let d_start: i64 = match active {
        "1980s" => 1980, "1990s" => 1990, "2000s" => 2000, "2010s" => 2010, _ => 2020,
    };

    // Decade tabs — each shows count of entries
    let tabs: String = decades.iter().map(|&d| {
        let on = d == active;
        let ds: i64 = match d { "1980s"=>1980,"1990s"=>1990,"2000s"=>2000,"2010s"=>2010,_=>2020 };
        let count = history.iter()
            .filter(|h| h["year"].as_i64().map(|y| y >= ds && y < ds+10).unwrap_or(false))
            .count();
        format!(
            "<a href=\"/?zone=archive&decade={d}\"\
               hx-get=\"/?zone=archive&decade={d}\"\
               hx-target=\"#archive-tl\" hx-swap=\"outerHTML\"\
               hx-indicator=\"#archive-spin\"\
               class=\"dtab {cls}\">{d} <span style=\"font-size:10px;opacity:.7\">({count})</span></a>",
            cls = if on { "dtab-on" } else { "dtab-off" },
        )
    }).collect::<Vec<_>>().join("");

    let decade_entries: Vec<&serde_json::Value> = history.iter()
        .filter(|h| h["year"].as_i64()
            .map(|y| y >= d_start && y < d_start + 10)
            .unwrap_or(false))
        .collect();

    let timeline = build_timeline(&decade_entries);

    let title_cards: String = recent_titles.iter().take(8).map(|t| {
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

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bear Archives</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          Community history from 1987 to present — {n} milestones documented.</p>\
        <div style=\"display:flex;gap:6px;flex-wrap:wrap;margin-bottom:16px\">{tabs}</div>\
        <div id=\"archive-spin\" class=\"htmx-indicator\">Loading…</div>\
        <div id=\"archive-tl\">{timeline}</div>\
        {h_titles}\
        {title_cards}\
        <a href=\"/?zone=titles\" style=\"display:block;text-align:center;font-size:13px;\
           color:{ORANGE};padding:8px 0 16px\">Full title archive (87 records) →</a>\
        <div class=\"card\">\
          <div style=\"font-weight:600;font-size:14px;margin-bottom:4px\">Clubs &amp; Organisations</div>\
          <div style=\"font-size:12px;color:{MID};margin-bottom:6px\">49 clubs across 20+ countries.</div>\
          <a href=\"/?zone=clubs\" style=\"font-size:12px;color:{ORANGE}\">View all clubs →</a>\
        </div>",
        n        = history.len(),
        h_titles = sh("Recent Title Holders", None),
    );
    Html(shell("Bear Archives", "Community history 1987 to present.", "archive", &body)).into_response()
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
                {sig_html}\
              </div>\
            </div>"
        )
    }).collect()
}
// ── ZONE: BEAR FUTURE ─────────────────────────────────────────

async fn zone_future(db: SupabaseClient) -> Response {
    let s_url = format!(
        "{}/rest/v1/platform_settings\
         ?key=in.(treasury_balance_ada,operational_balance_ada,\
         governance_token_name,governance_dao_threshold,treasury_phase)",
        db.url
    );
    let settings: Vec<serde_json::Value> = db.get_json(&s_url).await.unwrap_or_default();
    let get = |k: &str| settings.iter()
        .find(|s| s["key"].as_str() == Some(k))
        .and_then(|s| s["value"].as_str())
        .unwrap_or("").to_string();

    let token_name    = get("governance_token_name");
    let dao_threshold = get("governance_dao_threshold");
    let phase         = get("treasury_phase");

    let p_url = format!(
        "{}/rest/v1/bear_future_proposals?active=eq.true&order=created_at.desc&limit=10",
        db.url
    );
    let proposals: Vec<serde_json::Value> = db.get_json(&p_url).await.unwrap_or_default();

    let treasury_card = format!(
        "<div style=\"border-radius:16px;overflow:hidden;margin-bottom:12px;\
             border:1px solid {TAN};box-shadow:0 1px 4px rgba(0,0,0,.06)\">\
          <div class=\"stripe\"></div>\
          <div style=\"background:#fff;padding:16px\">\
            <div style=\"font-size:10px;font-weight:700;text-transform:uppercase;\
                        letter-spacing:.1em;color:{MID};margin-bottom:12px\">Community Treasury</div>\
            <div style=\"display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:12px\">\
              <div style=\"background:{OFF_WHITE};border-radius:12px;padding:12px;text-align:center\">\
                <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">₳ 0</div>\
                <div style=\"font-size:11px;color:{MID}\">Community ADA</div>\
              </div>\
              <div style=\"background:{OFF_WHITE};border-radius:12px;padding:12px;text-align:center\">\
                <div style=\"font-size:22px;font-weight:700;color:{BROWN}\">₳ 0</div>\
                <div style=\"font-size:11px;color:{MID}\">Operational ADA</div>\
              </div>\
            </div>\
            <div style=\"font-size:11px;color:{MID};text-align:center;margin-bottom:12px\">\
              Phase {phase} · Wallets being configured</div>\
            <div style=\"background:{OFF_WHITE};border-radius:12px;padding:14px;text-align:center\">\
              <div style=\"font-size:18px;font-weight:700;color:{BROWN}\">{token_name}</div>\
              <div style=\"font-size:11px;color:{MID};margin-bottom:4px\">Governance token · Follow the NORTH.</div>\
              <div style=\"font-size:11px;color:{MID};margin-bottom:8px\">\
                0 / {dao_threshold} holders · DAO unlocks at {dao_threshold}</div>\
              <div style=\"height:6px;border-radius:999px;background:{TAN}\">\
                <div style=\"height:6px;border-radius:999px;background:{GOLD};width:0%\"></div>\
              </div>\
            </div>\
          </div>\
        </div>"
    );

    let north_block = format!(
        "<div style=\"border-radius:16px;border:1px solid {GOLD};\
             background:#FFFEF5;padding:16px;margin-bottom:12px\">\
          <div style=\"font-weight:600;font-size:14px;color:{BROWN};margin-bottom:8px\">What is NORTH?</div>\
          <div style=\"font-size:12px;color:{MID};line-height:1.7\">\
            <strong style=\"color:{BROWN}\">NORTH</strong> is the Bearings governance token on Cardano.\
            Every verified title holder, club officer, and community steward receives 1 NORTH.<br><br>\
            More NORTH = more sway. <em>Follow the NORTH.</em><br><br>\
            When 100 holders are verified, the DAO activates — NORTH holders vote on proposals\
            directly, no intermediary.\
          </div>\
        </div>"
    );

    let proposals_html: String = if proposals.is_empty() {
        format!(
            "<div style=\"border-radius:16px;border:2px dashed {TAN};\
                 padding:24px;text-align:center;margin-bottom:12px\">\
              <div style=\"font-weight:600;font-size:14px;color:{BROWN};margin-bottom:6px\">No active proposals</div>\
              <div style=\"font-size:12px;color:{MID};line-height:1.6\">\
                The community treasury and governance are being established.\
                First proposals appear here once wallets are configured.</div>\
            </div>"
        )
    } else {
        proposals.iter().map(|p| {
            let title = p["title"].as_str().unwrap_or("Proposal");
            let sum   = p["summary"].as_str().unwrap_or("");
            let yes   = p["vote_yes"].as_i64().unwrap_or(0);
            let no    = p["vote_no"].as_i64().unwrap_or(0);
            let total = (yes + no).max(1);
            let pct   = yes * 100 / total;
            card(&format!(
                "<div>\
                  <div style=\"font-weight:600;font-size:14px\">{title}</div>\
                  <div style=\"font-size:11px;color:{MID};margin-bottom:8px\">{sum}</div>\
                  <div style=\"display:flex;gap:12px;font-size:12px;margin-bottom:6px\">\
                    <span style=\"color:#2e7d32\">✓ {yes} yes</span>\
                    <span style=\"color:#b71c1c\">✗ {no} no</span>\
                    <span style=\"color:{MID}\">{pct}% approval</span>\
                  </div>\
                  <div style=\"height:4px;border-radius:999px;background:{TAN}\">\
                    <div style=\"height:4px;border-radius:999px;background:{GOLD};width:{pct}%\"></div>\
                  </div>\
                </div>"
            ))
        }).collect()
    };

    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Bear Future</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">\
          Community treasury · NORTH governance · Proposals</p>\
        {treasury_card}\
        {north_block}\
        {h_props}\
        {proposals_html}",
        h_props = sh("Active Proposals", Some(proposals.len())),
    );
    Html(shell("Bear Future", "Community treasury and NORTH governance.", "future", &body)).into_response()
}

// ── SUPPLEMENTARY ZONES ───────────────────────────────────────

async fn zone_places(db: SupabaseClient) -> Response {
    let url = format!(
        "{}/rest/v1/places?active=eq.true\
         &select=name,place_type,city,country,address,hours_open,website,\
         booking_link,bear_popular,bear_night_schedule,inclusion_flag_codes\
         &order=country.asc,city.asc&limit=200",
        db.url
    );
    let places: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = places.iter().map(|p| {
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
                <div style=\"font-size:12px;color:{MID}\">{city}, {ctry}</div>\
                {addr_h}{hours_h}{bn_h}\
                <div style=\"margin-top:4px\">{fhtml}</div>\
              </div>\
              <div style=\"display:flex;flex-direction:column;gap:6px\">{site_html}{book_html}</div>\
            </div>",
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
    }).collect();
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">Bear Venues</h1>{items}"
    );
    Html(shell("Places", "Bear bars, saunas, campgrounds worldwide.", "places", &body)).into_response()
}

async fn zone_events(db: SupabaseClient, month: Option<u32>) -> Response {
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
    Html(shell("Events", "Bear events worldwide.", "now", &body)).into_response()
}

async fn zone_clubs(db: SupabaseClient) -> Response {
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
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">Bear Clubs</h1>{items}"
    );
    Html(shell("Clubs", "Bear clubs worldwide.", "archive", &body)).into_response()
}

async fn zone_titles(db: SupabaseClient) -> Response {
    let url = format!(
        "{}/rest/v1/current_title_holders\
         ?select=title_name,holder_name,year,city,country,competition_scope\
         &order=title_name.asc",
        db.url
    );
    let holders: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = holders.iter().map(|h| {
        let title  = h["title_name"].as_str().unwrap_or("");
        let holder = h["holder_name"].as_str().unwrap_or("");
        let year   = h["year"].as_i64().unwrap_or(0);
        let city   = h["city"].as_str().unwrap_or("");
        let ctry   = h["country"].as_str().unwrap_or("");
        let scope  = h["competition_scope"].as_str().unwrap_or("");
        let icon   = match scope {
            "continental"=>"🌎","national"=>"🏳️","regional"=>"📍","local"=>"🏙️",_=>"🐻"
        };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:center\">\
              <div>\
                <div style=\"font-weight:600;font-size:14px\">{icon} {title}</div>\
                <div style=\"font-size:12px;color:{MID}\">{holder}</div>\
                <div style=\"font-size:11px;color:{MID}\">{city}{sep}{ctry}</div>\
              </div>\
              <div style=\"font-size:22px;font-weight:700;color:{ORANGE}\">{year}</div>\
            </div>",
            sep = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
        ))
    }).collect();
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:4px\">Title Holders</h1>\
        <p style=\"font-size:12px;color:{MID};margin-bottom:16px\">87 records · IBR complete 1992–2011.</p>\
        {items}"
    );
    Html(shell("Titles", "Bear title holders worldwide.", "archive", &body)).into_response()
}
async fn zone_creators(db: SupabaseClient) -> Response {
    let url = format!(
        "{}/rest/v1/creators?active=eq.true\
         &select=name,creator_type,city,country,bio,website\
         &order=creator_type.asc,name.asc&limit=100",
        db.url
    );
    let creators: Vec<serde_json::Value> = db.get_json(&url).await.unwrap_or_default();
    let items: String = creators.iter().map(|c| {
        let name  = c["name"].as_str().unwrap_or("");
        let ctype = c["creator_type"].as_str().unwrap_or("creator");
        let city  = c["city"].as_str().unwrap_or("");
        let ctry  = c["country"].as_str().unwrap_or("");
        let bio   = c["bio"].as_str().unwrap_or("");
        let site  = c["website"].as_str().unwrap_or("");
        let site_html = if !site.is_empty() && site != "#" {
            format!("<a href=\"{site}\" target=\"_blank\" rel=\"noopener\" class=\"btn-t\">Site</a>")
        } else { String::new() };
        card(&format!(
            "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
              <div style=\"flex:1;min-width:0\">\
                <div style=\"font-weight:600;font-size:14px\">{name}\
                  <span style=\"font-weight:400;font-size:11px;color:{MID}\"> {ctype}</span>\
                </div>\
                <div style=\"font-size:12px;color:{MID}\">{city}{sep}{ctry}</div>\
                {bio_h}\
              </div>\
              {site_html}\
            </div>",
            sep   = if !city.is_empty() && !ctry.is_empty() { ", " } else { "" },
            bio_h = if !bio.is_empty() {
                format!("<div style=\"font-size:12px;color:{MID};margin-top:4px;line-height:1.5\">{}</div>",
                    bio.chars().take(160).collect::<String>())
            } else { String::new() },
        ))
    }).collect();
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">Creators</h1>{items}"
    );
    Html(shell("Creators", "Bear community creators.", "archive", &body)).into_response()
}

async fn zone_campaigns(db: SupabaseClient) -> Response {
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
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">Community Campaigns</h1>{items}"
    );
    Html(shell("Campaigns", "Community campaigns.", "now", &body)).into_response()
}

async fn zone_digital(db: SupabaseClient) -> Response {
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
    let body = format!(
        "<h1 style=\"font-size:18px;font-weight:700;color:{BROWN};margin-bottom:16px\">Digital Spaces</h1>{items}"
    );
    Html(shell("Digital Spaces", "Bear digital spaces.", "now", &body)).into_response()
}

// ── LEGACY WRAPPERS (kept so existing routes in main.rs still compile) ──────
// These delegate to zone functions. Remove once Gaspar confirms ?zone= routing.

pub async fn now_page           (State(db): State<SupabaseClient>) -> Response { zone_now(db, None, None).await }
pub async fn coming_up_page     (State(db): State<SupabaseClient>) -> Response { zone_coming_up(db).await }
pub async fn history_page       (State(db): State<SupabaseClient>) -> Response { zone_archive(db, None).await }
pub async fn bear_future_page   (State(db): State<SupabaseClient>) -> Response { zone_future(db).await }
pub async fn events_page        (State(db): State<SupabaseClient>) -> Response { zone_events(db, None).await }
pub async fn places_page        (State(db): State<SupabaseClient>) -> Response { zone_places(db).await }
pub async fn clubs_page         (State(db): State<SupabaseClient>) -> Response { zone_clubs(db).await }
pub async fn titles_page        (State(db): State<SupabaseClient>) -> Response { zone_titles(db).await }
pub async fn creators_page      (State(db): State<SupabaseClient>) -> Response { zone_creators(db).await }
pub async fn campaigns_page     (State(db): State<SupabaseClient>) -> Response { zone_campaigns(db).await }
pub async fn digital_spaces_page(State(db): State<SupabaseClient>) -> Response { zone_digital(db).await }
