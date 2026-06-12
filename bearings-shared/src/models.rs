//! Database models — mirrors the Bearings Supabase schema.
//! Every field name matches the database column name exactly.
//! Reviewed by: Gaspar

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ── EVENTS ────────────────────────────────────────────────────

/// A bear event — run, festival, cruise, or social gathering.
/// Schema source: events table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub month: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub size: Option<String>,
    pub hot: Option<bool>, // featured/hot event flag used by NOW zone
    pub link: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub going: Option<i32>, // interested count
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub charity_name: Option<String>,
    pub charity_link: Option<String>,
    pub host_hotel: Option<String>,
    pub host_hotel_link: Option<String>,
    pub inclusion_flag_codes: Option<Vec<String>>,
    pub inclusion_notes: Option<String>,
    pub status: Option<String>,
    pub archive_notes: Option<String>,
    pub bluesky_handle: Option<String>,
    pub slug: Option<String>,       // URL slug for SEO routes
    pub event_mode: Option<String>, // in-person | hybrid | online
    pub stream_url: Option<String>,
    pub platform: Option<String>,
    pub recurring: Option<bool>,
    pub recurrence_note: Option<String>,
}

// ── PLACES ────────────────────────────────────────────────────

/// A physical bear venue — bar, sauna, campground, leather bar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Place {
    pub id: i64,
    pub name: String,
    pub place_type: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub bear_night_schedule: Option<String>,
    pub has_bear_night: Option<bool>,
    pub bear_popular: Option<bool>,
    pub men_only: Option<bool>,
    pub active: Option<bool>,
    pub closed_year: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: Option<DateTime<Utc>>,
}

// ── CLUBS ─────────────────────────────────────────────────────

/// A bear community club or organising association.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Club {
    pub id: i64,
    pub name: String,
    pub club_type: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub founded_year: Option<i32>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub contact_social: Option<String>,
    pub validator_notes: Option<String>,
    pub outreach_status: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ── TITLE HOLDERS ─────────────────────────────────────────────

/// A bear competition title holder — historical and current.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TitleHolder {
    pub id: i64,
    pub title_name: String,
    pub holder_name: String,
    pub year: Option<i32>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub bio: Option<String>,
    pub photo_link: Option<String>, // DB column is photo_link not photo_url
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

// ── COMPETITIONS ──────────────────────────────────────────────

/// A bear title competition — active or archived.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Competition {
    pub id: i64,
    pub name: String,
    pub competition_type: Option<String>,
    pub scope: Option<String>,
    pub frequency: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub founded_year: Option<i32>,
    pub discontinued_year: Option<i32>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub validator_notes: Option<String>,
    pub outreach_status: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ── BEAR HISTORY ──────────────────────────────────────────────

/// A bear community historical milestone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BearHistory {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub description: Option<String>,
    pub significance: Option<String>,
    pub category: Option<String>,
    pub link: Option<String>,
    pub active: Option<bool>,
    pub featured: Option<bool>,
    pub tags: Option<Vec<String>>,
}

// ── CAMPAIGNS ─────────────────────────────────────────────────

/// A charity campaign — active or archived.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Campaign {
    pub id: i64,
    pub name: String,
    pub org: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub goal: Option<f64>,
    pub raised: Option<f64>,
    pub currency: Option<String>,
    pub active: Option<bool>,
    pub privacy_mode: Option<bool>,
}

// ── TREASURY ──────────────────────────────────────────────────

/// A Bear Future community proposal.
/// Schema source: bear_future_proposals table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BearFutureProposal {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub title: String,       // NOT NULL
    pub description: String, // NOT NULL
    pub cause_category: Option<String>,
    pub target_amount_ada: f64, // NOT NULL
    pub target_amount_usd: Option<f64>,
    pub raised_ada: Option<f64>,
    pub receiving_wallet: Option<String>,
    pub applicant_name: Option<String>,
    pub applicant_email: Option<String>,
    pub applicant_club_id: Option<i64>,
    pub supporting_link: Option<String>,
    pub vote_yes: Option<i32>,
    pub vote_no: Option<i32>,
    pub vote_threshold_pct: Option<i32>, // defaults to 60 (operational) or 75 (constitutional)
    pub vote_min_count: Option<i32>,     // minimum NORTH votes required
    pub voting_opens_at: Option<DateTime<Utc>>,
    pub voting_closes_at: Option<DateTime<Utc>>,
    pub status: Option<String>, // draft | open | passed | failed | funded
    pub funded_at: Option<DateTime<Utc>>,
    pub tx_hash: Option<String>, // on-chain proof when funded
    pub privacy_mode: Option<bool>,
    pub urgent: Option<bool>,
    pub governance_ready: Option<bool>,
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub reviewed_by_steward: Option<bool>,
    pub steward_notes: Option<String>,
}

/// A NORTH token holder — verified bear community contributor.
/// Schema source: governance_token_holders table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceTokenHolder {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub display_name: String,           // NOT NULL in DB
    pub cardano_wallet: Option<String>, // single wallet field (not custodial/self_custody)
    pub user_pref_id: Option<i64>,      // FK to user_preferences
    pub contributor_tier: String, // NOT NULL: anonymous|community|verified_contributor|club_officer|steward
    pub verified_role_description: Option<String>,
    pub title_holder_id: Option<i64>, // FK to title_holders (if verified via competition)
    pub club_id: Option<i64>,         // FK to clubs (if verified via club officer role)
    pub token_balance: Option<i32>,   // NORTH token count = voting weight (default 1)
    pub proposals_voted: Option<i32>,
    pub proposals_passed: Option<i32>,
    pub verified: Option<bool>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<String>, // default: "steward"
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub authorization_phase: Option<i32>,
}

/// An operational ledger entry — every ADA movement logged.
/// Schema source: operational_ledger table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationalLedger {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub tx_hash: Option<String>,
    pub tx_date: Option<chrono::NaiveDate>, // DB type is date, not timestamptz
    pub direction: String,                  // NOT NULL in DB: "in" | "out"
    pub amount_ada: Option<f64>,
    pub amount_usd: Option<f64>,
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub authorized_by: Option<String>,
    pub authorization_phase: Option<i32>, // defaults to 1 in DB
    pub donor_display: Option<String>,    // anonymised donor name if provided
    pub donor_wallet: Option<String>,     // donor wallet (privacy sensitive)
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

// Helper for skip_serializing_if — prevents sending id=0 to PostgREST
// PostgREST rejects inserts that supply a value for GENERATED ALWAYS columns
fn is_zero_id(id: &i64) -> bool {
    *id == 0
}

// ── AGENT INFRASTRUCTURE ──────────────────────────────────────

/// An operating document — directive, research, state, whitepaper.
/// Queried at session start to load agent context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Document {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub version: Option<String>,
    pub content: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<String>,
    pub active: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// A Rust source file — workspace code stored in Supabase.
/// Agents read this to understand and extend the codebase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Code {
    pub id: i64,
    pub crate_name: String, // crate column aliased to avoid Rust keyword
    pub file_path: String,
    pub content: String,
    pub language: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ── IMPL HELPERS ──────────────────────────────────────────────
// Utility methods on the model types.
// These add behaviour without changing the data shape.

impl Event {
    /// Is this event happening in the next N days?
    pub fn is_upcoming(&self, within_days: i64) -> bool {
        match self.start_date {
            Some(date) => {
                let today = chrono::Local::now().date_naive();
                let diff = (date - today).num_days();
                diff >= 0 && diff <= within_days
            }
            None => false,
        }
    }

    /// Short display string: "Bear Frolic 2026 — Ottawa, Canada"
    pub fn display(&self) -> String {
        format!(
            "{} — {}, {}",
            self.name,
            self.city.as_deref().unwrap_or(""),
            self.country.as_deref().unwrap_or(""),
        )
    }
}

impl Place {
    /// Is this place men-only?
    pub fn is_men_only(&self) -> bool {
        self.men_only.unwrap_or(false)
    }

    /// Short display: "The Laird Hotel · Leather Bar · Melbourne, Australia"
    pub fn display(&self) -> String {
        format!(
            "{} · {} · {}, {}",
            self.name,
            self.place_type.as_deref().unwrap_or("venue"),
            self.city.as_deref().unwrap_or(""),
            self.country.as_deref().unwrap_or(""),
        )
    }
}

impl TitleHolder {
    /// Is this the current (most recent, active=true) holder?
    pub fn is_current(&self) -> bool {
        self.active.unwrap_or(false)
    }

    /// Display: "Mr Bear Europe 2026 — TBD (Lisbon)"
    pub fn display(&self) -> String {
        format!(
            "{} {} — {}",
            self.title_name,
            self.year.unwrap_or(0),
            self.holder_name,
        )
    }
}

impl OperationalLedger {
    /// Format the ADA amount with sign: "+2.50 ADA" or "-1.20 ADA"
    pub fn amount_display(&self) -> String {
        let ada = self.amount_ada.unwrap_or(0.0);
        // direction is String (NOT NULL), not Option<String>
        let sign = match self.direction.as_str() {
            "in" => "+",
            "out" => "-",
            _ => "",
        };
        format!("{}{:.2} ADA", sign, ada.abs())
    }
}

impl GovernanceTokenHolder {
    /// Voting weight = token_balance (NORTH tokens held)
    pub fn voting_weight(&self) -> i32 {
        self.token_balance.unwrap_or(0)
    }

    /// Resolved display name — falls back to "Anonymous Contributor"
    pub fn resolved_name(&self) -> &str {
        self.display_name.as_str()
    }
}

// ── BLUESKY / SOCIAL LAYER ────────────────────────────────────
// These tables are live in the DB but not yet exposed via API routes.
// They represent a whole platform capability that's ahead of the Rust code.

/// An inbound social post from Bluesky or other platform.
/// The agent monitors mentions and replies, storing them here for processing.
/// Part of the CONST-10 "inclusion shown not decided" pipeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentInbox {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub platform: String, // "bluesky" | "mastodon"
    pub post_uri: String,
    pub post_cid: Option<String>,
    pub author_handle: Option<String>,
    pub author_did: Option<String>,
    pub post_text: Option<String>,
    pub in_reply_to_uri: Option<String>,
    pub reply_to_post_id: Option<i64>,
    pub intent: Option<String>, // "submission" | "question" | "feedback"
    pub status: Option<String>, // "pending" | "responded" | "escalated"
    pub response_text: Option<String>,
    pub response_uri: Option<String>,
    pub responded_at: Option<DateTime<Utc>>,
    pub escalated_to_steward: Option<bool>,
    pub escalation_reason: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A scheduled or published Bluesky post.
/// The agent drafts posts, steward reviews, then posts go live (CONST-10).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentPost {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub platform: String,
    pub post_type: String, // "event_announce" | "title_holder" | "campaign" | "history"
    pub event_id: Option<i64>,
    pub place_id: Option<i64>,
    pub creator_id: Option<i64>,
    pub club_id: Option<i64>,
    pub campaign_id: Option<i64>,
    pub history_id: Option<i64>,
    pub post_text: String,
    pub post_uri: Option<String>,
    pub post_cid: Option<String>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub status: Option<String>, // "draft" | "scheduled" | "published" | "failed"
    pub like_count: Option<i32>,
    pub repost_count: Option<i32>,
    pub reply_count: Option<i32>,
    pub quote_count: Option<i32>,
    pub generated_by: Option<String>, // "agent" | "steward"
    pub reviewed_by_steward: Option<bool>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A governance vote cast by a NORTH token holder.
/// proposal_id + voter_id must be unique — one vote per holder per proposal.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProposalVote {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub proposal_id: i64,
    pub voter_id: i64,            // FK to governance_token_holders
    pub vote: String,             // "yes" | "no" | "abstain"
    pub vote_weight: Option<i32>, // = voter's token_balance at time of vote
    pub voted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// An inclusion flag — codes used to tag events/places that may exclude certain groups.
/// CONST-10: never remove, always flag with context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InclusionFlag {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub code: String, // e.g. "MASC_ONLY", "AGE_RESTRICTION"
    pub label: String,
    pub description: String,
    pub severity: Option<String>, // "info" | "caution" | "warning"
    pub affected_groups: Option<Vec<String>>,
    pub icon: Option<String>,
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

/// A media item linked to a creator — films, albums, podcasts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Media {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub title: String,
    pub creator_id: Option<i64>,
    pub media_type: Option<String>, // "film" | "album" | "podcast" | "book"
    pub year: Option<i32>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub link: Option<String>,
    pub streaming_link: Option<String>,
    pub trailer_link: Option<String>,
    pub event_id: Option<i64>,
    pub bear_community_subject: Option<bool>,
    pub featured: Option<bool>,
    pub active: Option<bool>,
    pub inclusion_flag_codes: Option<Vec<String>>,
    pub inclusion_notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub timeline_year: Option<i32>,
    pub timeline_month: Option<i32>,
}

// ── CREATOR ZONE ──────────────────────────────────────────────

/// A bear community creator — musician, filmmaker, illustrator, historian.
/// Schema source: creators table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Creator {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub name: String,
    pub creator_type: Option<String>, // musician | filmmaker | illustrator | historian | author | artist
    pub pronouns: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub website: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub spotify_link: Option<String>,
    pub youtube_link: Option<String>,
    pub bandcamp_link: Option<String>,
    pub patreon_link: Option<String>,
    pub booking_link: Option<String>,
    pub bio: Option<String>,
    pub tags: Option<Vec<String>>,
    pub bear_community_member: Option<bool>,
    pub bear_affiliated: Option<bool>,
    pub active: Option<bool>,
    pub verified: Option<bool>,
    pub inclusion_flag_codes: Option<Vec<String>>,
    pub inclusion_notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub bluesky_handle: Option<String>,
    pub linktree_url: Option<String>,
    pub etsy_link: Option<String>,
    pub slug: Option<String>,
}

/// An oral history or community story.
/// Schema source: stories table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Story {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub title: String,
    pub story_type: Option<String>, // oral_history | essay | interview | archive
    pub creator_id: Option<i64>,
    pub event_id: Option<i64>,
    pub club_id: Option<i64>,
    pub year: Option<i32>,
    pub body: Option<String>,
    pub excerpt: Option<String>,
    pub link: Option<String>,
    pub archive_source: Option<String>,
    pub language_code: Option<String>,
    pub tags: Option<Vec<String>>,
    pub active: Option<bool>,
    pub featured: Option<bool>,
    pub privacy_mode: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub timeline_year: Option<i32>,
    pub timeline_month: Option<i32>,
}

/// A digital space — apps, Discord servers, podcasts, Twitch, Reddit communities.
/// Schema source: digital_spaces table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigitalSpace {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub name: String,
    pub space_type: String, // NOT NULL: app | discord | podcast | twitch | reddit | youtube | telegram
    pub platform: Option<String>,
    pub url: Option<String>,
    pub app_store_ios: Option<String>,
    pub app_store_android: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub bear_specific: Option<bool>,
    pub community_types: Option<Vec<String>>,
    pub nsfw_content: Option<bool>,
    pub age_restriction: Option<String>,
    pub member_count: Option<i32>,
    pub member_count_note: Option<String>,
    pub active: Option<bool>,
    pub verified: Option<bool>,
    pub discord_invite: Option<String>,
    pub twitch_handle: Option<String>,
    pub bluesky_handle: Option<String>,
    pub instagram: Option<String>,
    pub tiktok_handle: Option<String>,
    pub reddit_handle: Option<String>,
    pub linked_club_id: Option<i64>,
    pub linked_creator_id: Option<i64>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub inclusion_flag_codes: Option<Vec<String>>,
    pub inclusion_notes: Option<String>,
    pub founded_year: Option<i32>,
    pub closed_year: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub game_title: Option<String>,
    pub game_server: Option<String>,
    pub covid_origin: Option<bool>,
    pub event_linked: Option<bool>,
    pub booking_notes: Option<String>,
}

/// User preferences — inclusion filter settings, session tracking.
/// Schema source: user_preferences table (verified 2026-06-06)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(skip_serializing_if = "is_zero_id")]
    pub id: i64,
    pub session_id: Option<String>,
    pub user_email: Option<String>,
    pub show_all_venues: Option<bool>,
    pub hide_flag_codes: Option<Vec<String>>, // inclusion_flag_codes to hide
    pub warn_flag_codes: Option<Vec<String>>, // inclusion_flag_codes to warn on
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
