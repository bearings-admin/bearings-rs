
//! Unit tests for bearings-shared model helpers.
//! These run without any network access — pure logic tests.

use bearings_shared::models::{Event, GovernanceTokenHolder, OperationalLedger};

#[test]
fn event_display_formats_correctly() {
    let event = Event {
        id: 1,
        name: "Bear Frolic 2026".to_string(),
        city: Some("Ottawa".to_string()),
        country: Some("Canada".to_string()),
        ..Default::default()
    };
    assert_eq!(event.display(), "Bear Frolic 2026 — Ottawa, Canada");
}

#[test]
fn ledger_entry_formats_inbound() {
    let entry = OperationalLedger {
        id: 0,
        direction: Some("in".to_string()),
        amount_ada: Some(50.0),
        ..Default::default()
    };
    assert_eq!(entry.amount_display(), "+50.00 ADA");
}

#[test]
fn ledger_entry_formats_outbound() {
    let entry = OperationalLedger {
        id: 0,
        direction: Some("out".to_string()),
        amount_ada: Some(1.234567),
        ..Default::default()
    };
    assert_eq!(entry.amount_display(), "-1.23 ADA");
}

#[test]
fn governance_holder_voting_weight() {
    let holder = GovernanceTokenHolder {
        id: 1,
        token_balance: Some(3),
        ..Default::default()
    };
    assert_eq!(holder.voting_weight(), 3);
}
