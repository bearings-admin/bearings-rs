//! Unit tests for bearings-shared model helpers.
//! These run without any network access — pure logic tests.

use bearings_shared::models::{Event, OperationalLedger};

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
        direction: "in".to_string(),
        amount_usdc: Some(50.0),
        ..Default::default()
    };
    assert_eq!(entry.amount_display(), "+50.00 USDC");
}

#[test]
fn ledger_entry_formats_outbound() {
    let entry = OperationalLedger {
        id: 0,
        direction: "out".to_string(),
        amount_usdc: Some(1.234567),
        ..Default::default()
    };
    assert_eq!(entry.amount_display(), "-1.23 USDC");
}
