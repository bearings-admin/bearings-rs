//! Design system — shared across all SSR zone renderers.
//!
//! Contains:
//! - Colour constants (import with `use crate::ui::*;`)
//! - HTML helper functions: shell, card, sh, flags, timeline_bar, etc.
//!
//! None of these functions call Supabase. They are pure render helpers.
//! Zone functions live in `src/ssr/zones/` and call into this module.

use crate::i18n;

// ── Colour palette (Variant G) ────────────────────────────────────────────
pub(crate) const BROWN: &str = "#5C4033";
pub(crate) const ORANGE: &str = "#D2691E";
pub(crate) const GOLD: &str = "#D4A017";
pub(crate) const TAN: &str = "#C8B89A";
pub(crate) const OFF_WHITE: &str = "#F9F5F0";
pub(crate) const DARK: &str = "#1A1A1A";
pub(crate) const MID: &str = "#777777";

/// HTML-escape a value for safe interpolation into element text or a
/// double-quoted attribute. Escapes `& < > " '` so untrusted DB, submission, or
/// feed content cannot inject markup. Apply to every dynamic value rendered to HTML.
pub(crate) fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

pub(crate) fn shell(
    title: &str,
    description: &str,
    active: &str,
    body: &str,
    lang: &str,
) -> String {
    let i18n = i18n::translations();
    let tl = |key: &str| i18n::t(i18n, lang, key);

    // Language switcher
    let lang_switcher: String = [("en", "EN"), ("es", "ES"), ("fr", "FR"), ("ja", "JA")]
        .iter()
        .map(|(code, label)| {
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
        })
        .collect();

    // Bottom nav — 4 temporal zones, inline SVG icons
    let tnav_svg = |zone: &str, label: &str, svg: &str| {
        let on = zone == active;
        let col = if on { ORANGE } else { BROWN };
        let fw = if on { "700" } else { "400" };
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
        ("places", "🍺", "nav.places"),
        ("clubs", "🏳\u{fe0f}", "nav.clubs"),
        ("creators", "🎨", "nav.creators"),
        ("shops", "\u{1f6cd}\u{fe0f}", "nav.shops"),
        ("titles", "🏆", "nav.titles"),
        ("campaigns", "💚", "nav.campaigns"),
        ("digital-spaces", "📱", "nav.digital"),
    ];
    let dir_links: String = dir_items
        .iter()
        .map(|(zone, icon, key)| {
            let on = zone == &active;
            format!(
                "<a href=\"/?zone={zone}&lang={lang}\" \
               style=\"display:flex;align-items:center;gap:10px;\
                       padding:10px 0;border-bottom:1px solid {TAN};\
                       text-decoration:none;color:{col};font-weight:{fw}\">\
              <span style=\"font-size:18px\">{icon}</span>\
              <span style=\"font-size:14px\">{label}</span>\
            </a>",
                col = if on { ORANGE } else { DARK },
                fw = if on { "700" } else { "400" },
                label = tl(key),
            )
        })
        .collect();

    let _ = tl;
    format!(
        "<!DOCTYPE html>\n\
<html lang=\"{lang}\">\n\
<head>\n\
  <meta charset=\"UTF-8\">\n\
  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n\
  <title>{title} — Bearings</title>\n\
  <meta name=\"description\" content=\"{description}\">\n\
  <meta property=\"og:type\" content=\"website\">\n\
  <meta property=\"og:site_name\" content=\"Bearings\">\n\
  <meta property=\"og:title\" content=\"{title} — Bearings\">\n\
  <meta property=\"og:description\" content=\"{description}\">\n\
  <meta property=\"og:url\" content=\"https://www.bearings.community/\">\n\
  <meta property=\"og:image\" content=\"https://www.bearings.community/og-image.png\">\n\
  <meta name=\"twitter:card\" content=\"summary\">\n\
  <link rel=\"canonical\" href=\"https://www.bearings.community/\">\n\
  <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n\
  <link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap\">\n\
  <script src=\"https://unpkg.com/htmx.org@1.9.12\" integrity=\"sha384-ujb1lZYygJmzgSwoxRggbCHcjc0rB2XoQrxeTUQyRjrOnlCoYta87iKBWq3EsdM2\" crossorigin=\"anonymous\"></script>\n\
  <link rel=\"stylesheet\" href=\"/style.css\">\n\
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
<span style=\"font-size:18px\">📅</span><span style=\"font-size:14px\">iCal Export</span></a><a href=\"/?zone=transparency&lang={lang}\" style=\"display:flex;align-items:center;gap:10px;padding:10px 0;border-top:1px solid {TAN};text-decoration:none;color:{DARK}\"><span style=\"font-size:18px\">💸</span><span style=\"font-size:14px\">Transparency</span></a>\n\
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

/// The shared stylesheet, served once at /style.css and browser-cached,
/// instead of being re-sent inline in every page. Built once via OnceLock.
pub(crate) fn stylesheet() -> &'static str {
    static CSS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CSS.get_or_init(|| {
        format!(
            "    *{{box-sizing:border-box;margin:0;padding:0}}\n\
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
"
        )
    })
}

pub(crate) fn card(c: &str) -> String {
    format!("<div class=\"card\">{c}</div>")
}

/// One row of an entity card: main content on the left (it flexes to fill), a small
/// action or aside pinned to the right. Every zone builds its card body from this, so
/// the row layout is defined in exactly one place.
///
/// Deliberately minimal: it does *not* try to model the whole card. Per-zone specifics
/// (a progress bar, a media list, a bordered "ships worldwide" pill) stay in the zone.
/// The shared seam is only the part that was genuinely identical everywhere.
pub(crate) fn split(content: &str, aside: &str) -> String {
    format!(
        "<div style=\"display:flex;justify-content:space-between;align-items:flex-start;gap:10px\">\
           <div style=\"flex:1;min-width:0\">{content}</div>{aside}\
         </div>"
    )
}

/// A static pill (the `.badge` CSS class). `label` is escaped; `bg`/`fg` are trusted
/// colour constants from this module.
pub(crate) fn badge(label: &str, bg: &str, fg: &str) -> String {
    format!(
        "<span class=\"badge\" style=\"background:{bg};color:{fg}\">{}</span>",
        esc(label)
    )
}

/// A clickable pill that always opens safely in a new tab. `href` must already be
/// escaped/trusted (callers build it from `esc(...)`); `label` is escaped here.
pub(crate) fn link_badge(href: &str, label: &str, bg: &str) -> String {
    format!(
        "<a href=\"{href}\" target=\"_blank\" rel=\"noopener\" class=\"badge\" \
           style=\"background:{bg};color:#fff\">{}</a>",
        esc(label)
    )
}

pub(crate) fn country_region(country: &str) -> &'static str {
    match country {
        "Canada" | "USA" | "Mexico" | "Puerto Rico" => "North America",
        "Belgium" | "Czech Republic" | "Czechia" | "Estonia" | "France" | "Germany" | "Iceland"
        | "Ireland" | "Italy" | "Luxembourg" | "Netherlands" | "Norway" | "Poland" | "Portugal"
        | "Scotland" | "Spain" | "Sweden" | "Switzerland" | "UK" => "Europe",
        "Australia" | "Japan" | "Malaysia" | "New Zealand" | "Philippines" | "Singapore"
        | "South Korea" | "Taiwan" | "Thailand" => "Asia Pacific",
        "Argentina" | "Brazil" | "Chile" | "Colombia" | "Uruguay" => "Latin America",
        "Egypt" | "Israel" | "Morocco" | "South Africa" | "UAE" => "Africa & Middle East",
        _ => "Other",
    }
}

pub(crate) fn sh(label: &str, n: Option<usize>) -> String {
    let pill = n
        .map(|x| format!("<span class=\"cp\">{x}</span>"))
        .unwrap_or_default();
    format!("<div class=\"sh\">{label}{pill}</div>")
}

pub(crate) fn flags(codes: &[String]) -> String {
    codes
        .iter()
        .map(|c| {
            let (lbl, bg, fg) = match c.as_str() {
                "men_only" => ("♂ men only", "#EDE0D4", BROWN),
                "clothing_optional" => ("🌿 clothing opt.", "#F0EAD6", "#5a6f2b"),
                "members_only" => ("🔒 members", "#EDE0D4", BROWN),
                "adults_only" => ("18+", "#FCEBD5", ORANGE),
                "bear_focused" => ("🐻 bear focused", "#FBF0E0", ORANGE),
                _ => (c.as_str(), "#EEE", "#555"),
            };
            format!(
                "<span class=\"badge\" style=\"background:{bg};color:{fg}\">{lbl}</span>",
                lbl = esc(lbl)
            )
        })
        .collect()
}

pub(crate) fn timeline_bar(
    start_dates: &[Option<String>],
    active_month: Option<u32>,
    href_base: &str,
    hx_target: &str,
) -> String {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let bear_col = [
        DARK, MID, BROWN, BROWN, ORANGE, ORANGE, GOLD, GOLD, TAN, TAN, "#AAAAAA", DARK,
    ];
    let mut counts = [0usize; 12];
    for d_opt in start_dates {
        if let Some(d) = d_opt.as_deref() {
            let parts: Vec<&str> = d.splitn(3, '-').collect();
            if let Some(m) = parts.get(1).and_then(|s| s.parse::<usize>().ok()) {
                if (1..=12).contains(&m) {
                    counts[m - 1] += 1;
                }
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
    } else {
        String::new()
    };
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

pub(crate) fn extract_month(date_str: &str) -> Option<u32> {
    let parts: Vec<&str> = date_str.splitn(3, '-').collect();
    parts.get(1).and_then(|s| s.parse::<u32>().ok())
}

// ── ROOT DISPATCHER ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::esc;

    #[test]
    fn esc_neutralises_script_injection() {
        assert_eq!(
            esc("<script>alert('x')</script>"),
            "&lt;script&gt;alert(&#x27;x&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn esc_escapes_attribute_breakout() {
        // A value injected into href="..." must not be able to close the attribute.
        assert_eq!(
            esc("a\" onerror=\"alert(1)"),
            "a&quot; onerror=&quot;alert(1)"
        );
    }

    #[test]
    fn esc_leaves_plain_text_untouched() {
        assert_eq!(esc("Berlin Bear Week 2026"), "Berlin Bear Week 2026");
        assert_eq!(esc("Café Zürich"), "Café Zürich");
    }
}
