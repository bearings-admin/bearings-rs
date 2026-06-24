//! Data-access layer.
//!
//! Each module exposes a repository *trait* (the abstraction the rest of the app
//! depends on) plus a concrete Supabase implementation. Handlers depend on the
//! trait, not on `SupabaseClient` directly — Dependency Inversion, so a handler
//! or service can be unit-tested against a fake repository.
//!
//!   routes  ->  <Resource>Repository (trait)  ->  Supabase<Resource>Repository  ->  PostgREST
//!
//! All PostgREST filter clauses are built through `clause()`, which percent-encodes
//! the value. This is the single place query values meet the URL, so user input
//! can never inject an extra filter.

pub mod bear_future_repo;
pub mod campaign_repo;
pub mod club_repo;
pub mod competition_repo;
pub mod creator_repo;
pub mod digital_space_repo;
pub mod event_repo;
pub mod flag_repo;
pub mod future_idea_repo;
pub mod history_repo;
pub mod place_repo;
pub mod revival_repo;
pub mod story_repo;
pub mod submission_repo;
pub mod title_repo;
pub mod transparency_repo;

/// Build a PostgREST filter clause `&col=op.value` with the value percent-encoded.
///
/// Example: `clause("country", "eq", "Côte d'Ivoire")` →
/// `&country=eq.C%C3%B4te%20d%27Ivoire`. Encoding the value means a string such
/// as `"A&active=eq.false"` becomes inert instead of injecting a second filter.
pub(crate) fn clause(col: &str, op: &str, value: &str) -> String {
    format!("&{}={}.{}", col, op, urlencoding::encode(value))
}

#[cfg(test)]
mod tests {
    use super::clause;

    #[test]
    fn clause_percent_encodes_values() {
        assert_eq!(clause("country", "eq", "Spain"), "&country=eq.Spain");
    }

    /// A value containing `&` must not inject a second PostgREST filter.
    #[test]
    fn clause_neutralises_filter_injection() {
        let injected = clause("city", "eq", "X&active=eq.false");
        assert_eq!(injected, "&city=eq.X%26active%3Deq.false");
        assert!(!injected.contains("&active=eq.false"));
    }
}
