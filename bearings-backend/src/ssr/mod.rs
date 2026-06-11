//! SSR dispatcher — receives GET / requests and routes them to zone renderers.
//!
//! Zone renderers are in `src/ssr/zones/`. Each zone is a separate file.
//! Design system (colours + helpers) is in `src/ui.rs`.
//!
//! Routing model: a single `?zone=<name>` query param dispatches to the correct
//! handler. HTMX enhances navigation in the browser; every link works without JS.

mod query;
mod zones;

// ── Zone function imports ─────────────────────────────────────────────────────
use zones::now::zone_now;
use zones::coming_up::zone_coming_up;
use zones::archive::zone_archive;
use zones::future::zone_future;
use zones::places::zone_places;
use zones::events::zone_events;
use zones::clubs::zone_clubs;
use zones::titles::zone_titles;
use zones::creators::zone_creators;
use zones::campaigns::zone_campaigns;
use zones::ical::zone_ical;
use zones::digital::zone_digital;
use zones::admin::zone_admin;


use axum::{
    extract::{Query, State},
    response::Response,
};
use crate::db::SupabaseClient;
use serde::Deserialize;


// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct ZoneQuery {
    pub zone:          Option<String>,
    pub decade:        Option<String>,
    pub month:         Option<u32>,
    pub fragment:      Option<String>,
    pub place_type:    Option<String>,
    pub place_country: Option<String>,
    pub lang:          Option<String>,
    pub months_ahead:  Option<u32>,
    pub event_country: Option<String>,
    pub token:         Option<String>,
}

// ── Zone discriminant ─────────────────────────────────────────────────────────

/// Typed zone discriminant — prevents silent fallthrough on typos in `?zone=` param.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Zone {
    Now, ComingUp, Archive, Future,
    Places, Events, Clubs, Titles, Creators,
    Campaigns, DigitalSpaces, Ical, Admin, History,
    Unknown,
}

impl Zone {
    pub fn parse(s: &str) -> Self {
        match s {
            "now"            => Self::Now,
            "coming-up"      => Self::ComingUp,
            "archive"        => Self::Archive,
            "future"         => Self::Future,
            "places"         => Self::Places,
            "events"         => Self::Events,
            "clubs"          => Self::Clubs,
            "titles"         => Self::Titles,
            "creators"       => Self::Creators,
            "campaigns"      => Self::Campaigns,
            "digital-spaces" => Self::DigitalSpaces,
            "ical"           => Self::Ical,
            "admin"          => Self::Admin,
            "history"        => Self::History,
            _                => Self::Unknown,
        }
    }
}

// ── Root dispatcher ───────────────────────────────────────────────────────────

pub async fn root(
    State(db): State<SupabaseClient>,
    Query(q): Query<ZoneQuery>,
) -> Response {
    let lang_owned = match q.lang.as_deref() { Some("es") => "es", Some("fr") => "fr", _ => "en" };
    match Zone::parse(q.zone.as_deref().unwrap_or("now")) {
        Zone::Now           => zone_now(db, lang_owned).await,
        Zone::ComingUp      => zone_coming_up(db, q.months_ahead, q.event_country.clone(), q.month, lang_owned).await,
        Zone::Archive       => zone_archive(db, q.decade, q.fragment.clone(), lang_owned).await,
        Zone::Future        => zone_future(db, lang_owned).await,
        Zone::Places        => zone_places(db, q.place_type.clone(), q.place_country.clone(), lang_owned).await,
        Zone::Events        => zone_events(db, q.month, lang_owned).await,
        Zone::Clubs         => zone_clubs(db, lang_owned).await,
        Zone::Titles        => zone_titles(db, lang_owned).await,
        Zone::Creators      => zone_creators(db, lang_owned).await,
        Zone::Campaigns     => zone_campaigns(db, lang_owned).await,
        Zone::DigitalSpaces => zone_digital(db, lang_owned).await,
        Zone::Ical          => zone_ical(lang_owned).await,
        Zone::Admin         => zone_admin(db, q.token.clone(), lang_owned).await,
        Zone::History | Zone::Unknown
                            => zone_coming_up(db, None, None, None, lang_owned).await,
    }
}

// ── Legacy named-path wrappers ────────────────────────────────────────────────
// These exist so /coming-up, /history etc. keep working without redirects.
// They delegate to the same zone functions as the dispatcher.

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

#[cfg(test)]
mod tests {
    use super::Zone;

    /// Every ?zone= string that appears in the UI resolves to the correct variant.
    /// If you add a new zone, add it here — this test acts as the registry.
    #[test]
    fn zone_parse_all_known() {
        let cases = [
            ("now",            Zone::Now),
            ("coming-up",      Zone::ComingUp),
            ("archive",        Zone::Archive),
            ("future",         Zone::Future),
            ("places",         Zone::Places),
            ("events",         Zone::Events),
            ("clubs",          Zone::Clubs),
            ("titles",         Zone::Titles),
            ("creators",       Zone::Creators),
            ("campaigns",      Zone::Campaigns),
            ("digital-spaces", Zone::DigitalSpaces),
            ("ical",           Zone::Ical),
            ("admin",          Zone::Admin),
            ("history",        Zone::History),
        ];
        for (input, expected) in cases {
            assert_eq!(Zone::parse(input), expected, "?zone={input} did not parse correctly");
        }
    }

    /// A typo or an unknown value must produce Zone::Unknown, not a panic.
    /// The dispatcher must handle unknown zones gracefully (fallback to Now).
    #[test]
    fn zone_parse_unknown_does_not_panic() {
        assert_eq!(Zone::parse("typo"),          Zone::Unknown);
        assert_eq!(Zone::parse(""),              Zone::Unknown);
        assert_eq!(Zone::parse("NOW"),           Zone::Unknown); // case-sensitive
        assert_eq!(Zone::parse("coming_up"),     Zone::Unknown); // underscore ≠ hyphen
        assert_eq!(Zone::parse("digitalspaces"), Zone::Unknown); // missing hyphen
    }

    /// Zone::parse is the single source of truth — verify it round-trips with
    /// the string representations used in HTML href attributes.
    #[test]
    fn zone_parse_matches_href_strings() {
        // These are the strings used in zone nav links throughout ui.rs shell()
        let nav_zones = ["now", "coming-up", "archive", "future"];
        for z in nav_zones {
            assert_ne!(Zone::parse(z), Zone::Unknown,
                "nav zone {z:?} is not registered in Zone::parse — nav link is broken");
        }
    }
}
