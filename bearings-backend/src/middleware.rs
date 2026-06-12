//! Middleware and request utilities.
//!
//! Privacy mode (CONST-6) is a constitutional requirement — it cannot be
//! overridden by any vote, user request, or agent directive.
//! This module centralises the enforcement so it is not scattered across
//! individual route handlers.

/// The complete list of countries where homosexuality is criminalised.
/// Source: ILGA World annual report — updated annually.
/// Last updated: 2026
///
/// These countries activate privacy_mode automatically for any submission,
/// user record, or financial transaction.
pub const CRIMINALISED_COUNTRIES: &[&str] = &[
    // Africa
    "algeria",
    "burundi",
    "cameroon",
    "chad",
    "comoros",
    "egypt",
    "eritrea",
    "ethiopia",
    "gambia",
    "ghana",
    "guinea",
    "kenya",
    "lesotho",
    "liberia",
    "libya",
    "malawi",
    "mauritania",
    "mauritius",
    "morocco",
    "mozambique",
    "namibia",
    "nigeria",
    "senegal",
    "sierra leone",
    "somalia",
    "south sudan",
    "sudan",
    "tanzania",
    "togo",
    "tunisia",
    "uganda",
    "zambia",
    "zimbabwe",
    // Asia
    "afghanistan",
    "bangladesh",
    "brunei",
    "indonesia",
    "iran",
    "iraq",
    "kuwait",
    "lebanon",
    "malaysia",
    "maldives",
    "myanmar",
    "oman",
    "pakistan",
    "palestine",
    "qatar",
    "saudi arabia",
    "singapore",
    "sri lanka",
    "syria",
    "turkmenistan",
    "uae",
    "united arab emirates",
    "uzbekistan",
    "yemen",
    // Oceania / other
    "papua new guinea",
    "solomon islands",
    "tonga",
    "samoa",
    "cook islands",
];

/// Check whether a country name is on the criminalised list.
/// Case-insensitive. Uses a HashSet for O(1) lookup.
/// The set is built once via std::sync::OnceLock — not rebuilt per request.
pub fn is_criminalised(country: &str) -> bool {
    use std::collections::HashSet;
    use std::sync::OnceLock;

    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    let set = SET.get_or_init(|| CRIMINALISED_COUNTRIES.iter().copied().collect());

    set.contains(country.to_lowercase().as_str())
}

/// Check an optional country field — safe to call on None.
pub fn country_is_criminalised(country: Option<&str>) -> bool {
    country.map(is_criminalised).unwrap_or(false)
}

/// Redact a wallet address for public display.
/// Shows only the first 8 and last 6 characters.
/// Example: "addr1xyz...abc456"
pub fn redact_wallet(address: &str) -> String {
    if address.len() < 20 {
        return "addr1...redacted".to_string();
    }
    format!("{}...{}", &address[..8], &address[address.len() - 6..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn criminalised_countries_detected() {
        assert!(is_criminalised("Iran"));
        assert!(is_criminalised("SAUDI ARABIA"));
        assert!(is_criminalised("nigeria"));
        assert!(is_criminalised("United Arab Emirates"));
    }

    #[test]
    fn safe_countries_not_flagged() {
        assert!(!is_criminalised("Canada"));
        assert!(!is_criminalised("Germany"));
        assert!(!is_criminalised("Australia"));
        assert!(!is_criminalised("Portugal"));
    }

    #[test]
    fn wallet_redaction() {
        let addr = "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3jcu5d8ps7zex2y9usm9y5e5rce";
        let redacted = redact_wallet(addr);
        assert!(redacted.starts_with("addr1qx2"));
        assert!(redacted.contains("..."));
    }
}
