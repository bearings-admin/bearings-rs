//! Typed query result structs for Supabase REST API responses.
//!
//! Each struct mirrors a SELECT from one Supabase table or view.
//! Use `db.get_json::<Vec<XxxRow>>(&url)` to deserialize directly.
//!
//! Date columns use `Option<String>` rather than `NaiveDate` because
//! the Supabase REST API returns ISO 8601 strings, not native date objects.
//! Convert to `NaiveDate` at the call site if date arithmetic is needed.

// These structs mirror full Supabase table shapes for documentation; not
// every column is rendered yet, so unread fields are expected.
#![allow(dead_code)]

use serde::Deserialize;

// ── Current title holders view ────────────────────────────────────────────────

/// Row from the `current_title_holders` Supabase view.
/// The view returns one row per `title_name` for the most recent competition,
/// resolves holdovers, and pre-joins competition scope.
/// Rust-side dedup in zone_now guards against the view returning duplicates.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct CurrentHolder {
    pub title_name: String,
    pub holder_name: String,
    pub holder_status: Option<String>,
    pub display_status: Option<String>,
    pub year: Option<i32>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub competition_scope: Option<String>,
    pub competition_name: Option<String>,
}

// ── Events ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EventRow {
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub hot: Option<bool>,
    pub link: Option<String>,
    pub description: Option<String>,
    pub event_mode: Option<String>,
    pub inclusion_flag_codes: Option<Vec<String>>,
}

// ── Campaigns ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CampaignRow {
    pub name: String,
    pub org: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub goal: Option<f64>,
    pub raised: Option<f64>,
    pub currency: Option<String>,
    pub urgent: Option<bool>,
    pub ends_at: Option<String>,
    pub cause: Option<String>,
    pub donate_url: Option<String>,
    pub usdc_accepted: Option<bool>,
}

// ── Places ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PlaceRow {
    pub name: String,
    pub place_type: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub hours_open: Option<String>,
    pub website: Option<String>,
    pub booking_link: Option<String>,
    pub bear_night_schedule: Option<String>,
    pub bear_popular: Option<bool>,
    pub inclusion_flag_codes: Option<Vec<String>>,
}

// ── Clubs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ClubRow {
    pub id: Option<i64>,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub club_type: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub founded_year: Option<i32>,
}

// ── Competitions ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CompetitionRow {
    pub id: i64,
    pub name: String,
    pub scope: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub website: Option<String>,
    pub founded_year: Option<i32>,
    pub owning_club_id: Option<i64>,
}

// ── Title Holders ─────────────────────────────────────────────────────────────

/// Row from the `title_holders` table.
/// `competition_id` is set when querying from `titles.rs` (join key).
/// `title_name` is set when querying from `archive.rs` (denormalised label).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TitleHolderRow {
    pub competition_id: Option<i64>,
    pub title_name: Option<String>,
    pub holder_name: String,
    pub year: Option<i32>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub inclusion_flag_codes: Option<Vec<String>>,
    pub holder_status: Option<String>,
    pub bio: Option<String>,
    pub charity_name: Option<String>,
    pub charity_link: Option<String>,
}

// ── Creators ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreatorRow {
    pub id: i64,
    pub name: String,
    pub creator_type: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub spotify_link: Option<String>,
    pub youtube_link: Option<String>,
    pub bandcamp_link: Option<String>,
    pub etsy_link: Option<String>,
    pub instagram: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MediaRow {
    pub title: Option<String>,
    pub creator_id: Option<i64>,
    pub media_type: Option<String>,
    pub year: Option<i32>,
    pub link: Option<String>,
    pub streaming_link: Option<String>,
    pub affiliate_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StoreRow {
    pub name: String,
    #[serde(rename = "type")]
    pub store_type: Option<String>,
    pub link: Option<String>,
    pub description: Option<String>,
    pub bear_owned: Option<bool>,
    pub size_inclusive: Option<bool>,
    pub ships_global: Option<bool>,
    pub affiliate_link: Option<String>,
    pub affiliate_pct: Option<f64>,
}

// ── Digital Spaces ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DigitalSpaceRow {
    pub name: String,
    pub space_type: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub member_count: Option<i64>,
    pub instagram: Option<String>,
    pub tiktok_handle: Option<String>,
    pub bluesky_handle: Option<String>,
    pub youtube_handle: Option<String>,
    pub id: Option<i64>,
    pub country: Option<String>,
}

// ── Bear History ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BearHistoryRow {
    pub year: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub significance: Option<String>,
    pub link: Option<String>,
    pub id: Option<i64>,
    pub month: Option<i32>,
    pub featured: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CommunityStoryRow {
    pub title: Option<String>,
    pub story_type: Option<String>,
    pub year: Option<i32>,
    pub excerpt: Option<String>,
    pub bear_history_id: Option<i64>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ClosedVenueRow {
    pub id: Option<i64>,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub closed_year: Option<i32>,
    pub revival_votes: Option<i64>,
    #[serde(alias = "place_type", alias = "club_type")]
    pub kind_type: Option<String>,
}

// ── Admin ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CandidateEventRow {
    pub id: Option<i64>,
    pub raw_title: Option<String>,
    pub raw_description: Option<String>,
    pub raw_date: Option<String>,
    pub parsed_country: Option<String>,
    pub source_url: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WatchedFeedRow {
    pub id: Option<i64>,
    pub org_name: Option<String>,
    pub feed_type: Option<String>,
    pub last_fetched: Option<String>,
    pub fetch_errors: Option<i64>,
}

// ── Future Ideas ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FutureIdeaRow {
    pub id: Option<i64>,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub upvotes: Option<i64>,
    pub source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the EventRow struct deserializes from the JSON shape
    /// that Supabase PostgREST actually returns. This catches field-name
    /// mismatches between the SELECT clause and the struct definition.
    ///
    /// If this test fails after a Supabase schema change, update the struct
    /// and the SELECT clause in the zone file together.
    #[test]
    fn event_row_deserializes_full() {
        let json = r#"{
            "name": "Grizzly Run",
            "city": "Montreal",
            "country": "Canada",
            "start_date": "2026-08-15",
            "end_date": "2026-08-18",
            "type": "run",
            "hot": true,
            "link": "https://example.com",
            "description": "Annual bear run.",
            "event_mode": "in-person",
            "inclusion_flag_codes": ["bear_focused", "adults_only"]
        }"#;
        let row: EventRow = serde_json::from_str(json).expect("EventRow failed to deserialize");
        assert_eq!(row.name, "Grizzly Run");
        assert_eq!(row.city.as_deref(), Some("Montreal"));
        assert_eq!(row.event_type.as_deref(), Some("run")); // renamed from "type"
        assert_eq!(row.hot, Some(true));
        let flags = row.inclusion_flag_codes.unwrap();
        assert_eq!(flags.len(), 2);
        assert!(flags.contains(&"bear_focused".to_string()));
    }

    /// A partial SELECT (not all columns) must still deserialize — optional
    /// fields missing from the response should be None, not an error.
    #[test]
    fn event_row_deserializes_partial_select() {
        // Simulates: SELECT name, city, country, start_date, type, hot, link
        let json = r#"{
            "name": "Bear Week Sitges",
            "city": "Sitges",
            "country": "Spain",
            "start_date": "2026-10-05",
            "type": "run",
            "hot": false,
            "link": "https://bearweeksitges.com"
        }"#;
        let row: EventRow =
            serde_json::from_str(json).expect("partial EventRow should deserialize");
        assert_eq!(row.name, "Bear Week Sitges");
        assert!(row.end_date.is_none());
        assert!(row.inclusion_flag_codes.is_none());
    }

    /// A missing required field (name is String, not Option<String>) must return
    /// an error — this is the key improvement over serde_json::Value.
    /// Previously: ev["nme"].as_str() silently returned "". Now: compile/parse error.
    #[test]
    fn event_row_rejects_missing_required_name() {
        let json = r#"{"city": "Berlin", "country": "Germany"}"#;
        let result: Result<EventRow, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "EventRow with missing 'name' should fail, not silently default"
        );
    }

    #[test]
    fn campaign_row_deserializes() {
        let json = r#"{
            "name": "Bear Welfare Fund",
            "org": "IBR",
            "link": "https://ibr.org/fund",
            "raised": 4200.0,
            "goal": 10000.0,
            "currency": "USD",
            "urgent": false
        }"#;
        let row: CampaignRow = serde_json::from_str(json).expect("CampaignRow failed");
        assert_eq!(row.name, "Bear Welfare Fund");
        assert_eq!(row.goal, Some(10000.0));
    }

    #[test]
    fn current_holder_dedup_keeps_latest_year() {
        use super::CurrentHolder;
        use std::collections::HashMap;

        // Simulate the view returning two rows for "Mr Bear UK" (2024 and 2025)
        let raw = vec![
            CurrentHolder {
                title_name: "Mr Bear UK".into(),
                holder_name: "Jordan Smith".into(),
                year: Some(2024),
                holder_status: Some("holdover".into()),
                display_status: None,
                city: None,
                country: Some("UK".into()),
                competition_scope: Some("national".into()),
                competition_name: Some("Mr Bear UK".into()),
            },
            CurrentHolder {
                title_name: "Mr Bear UK".into(),
                holder_name: "Alex Turner".into(),
                year: Some(2025),
                holder_status: Some("current".into()),
                display_status: None,
                city: None,
                country: Some("UK".into()),
                competition_scope: Some("national".into()),
                competition_name: Some("Mr Bear UK".into()),
            },
        ];

        // This is the dedup logic from zone_now
        let mut seen: HashMap<String, CurrentHolder> = HashMap::new();
        for t in raw {
            let year = t.year.unwrap_or(0);
            let existing = seen.get(&t.title_name).and_then(|v| v.year).unwrap_or(0);
            if year >= existing {
                seen.insert(t.title_name.clone(), t);
            }
        }

        let result = seen.get("Mr Bear UK").expect("Mr Bear UK should be in map");
        assert_eq!(
            result.holder_name, "Alex Turner",
            "dedup should keep 2025 holder, not 2024"
        );
        assert_eq!(result.year, Some(2025));
    }

    #[test]
    fn place_row_deserializes() {
        let json = r#"{
            "name": "The Eagle",
            "place_type": "bar",
            "city": "London",
            "country": "UK",
            "website": "https://eaglelondon.com",
            "bear_popular": true,
            "inclusion_flag_codes": ["bear_focused"]
        }"#;
        let row: PlaceRow = serde_json::from_str(json).expect("PlaceRow failed");
        assert_eq!(row.name, "The Eagle");
        assert_eq!(row.bear_popular, Some(true));
    }

    #[test]
    fn competition_row_id_is_required() {
        // id is i64 (not Option<i64>) — missing id must fail
        let json = r#"{"name": "Mr Bear Germany", "scope": "national"}"#;
        let result: Result<CompetitionRow, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "CompetitionRow with missing 'id' should fail"
        );
    }
}


/// A candidate duplicate pair from the `event_dupe_candidates` view.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DupePairRow {
    pub id_a: i64,
    pub name_a: String,
    pub city_a: Option<String>,
    pub date_a: Option<String>,
    pub id_b: i64,
    pub name_b: String,
    pub city_b: Option<String>,
    pub date_b: Option<String>,
    pub sim: Option<String>,
}


/// A provenance-bearing artifact (photo/document) attached to an entity.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ArtifactRow {
    pub id: i64,
    pub entity_id: Option<i64>,
    pub kind: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub transcription: Option<String>,
    pub contributor: Option<String>,
    pub provenance: Option<String>,
    pub captured_on: Option<String>,
    pub image_url: Option<String>,
}
