//! Enum types for the Bearings database schema.
//!
//! NOTE: These enums are defined but route handlers currently use raw String
//! fields for flexibility with PostgREST deserialization. Two approaches to
//! use them:
//!
//! 1. Keep String fields in models.rs, use enums for validation:
//!    `let place_type: PlaceType = serde_json::from_str(&place.place_type)?;`
//!
//! 2. Change models.rs fields from String to enum (Gaspar recommendation needed):
//!    Requires `#[serde(rename_all = "snake_case")]` to match DB values.
//!    Risk: deserialization fails if DB has unexpected values.
//!
//! Strategy: use enums in new code, convert models.rs in Phase 2 after Gaspar review.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    BearRun,
    Social,
    Cruise,
    Event,
    Party,
    Fundraiser,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSize {
    Local,
    Regional,
    National,
    Major,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceType {
    Bar,
    LeatherBar,
    SaunaBathhouse,
    Campground,
    Club,
    PartyVenue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletType {
    Custodial,
    SelfCustody,
    Both,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Open,
    Passed,
    Failed,
    Funded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributorTier {
    Anonymous,
    Community,
    VerifiedContributor,
    ClubOfficer,
    Steward,
}
