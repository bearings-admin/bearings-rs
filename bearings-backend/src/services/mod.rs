//! Service layer — business logic that orchestrates one or more repositories.
//!
//! Services depend on repository *traits*, not concrete database types, so the
//! logic here is unit-testable against fakes (see `vote_service` tests).
//!
//!   routes  ->  services  ->  repositories  ->  db

pub mod vote_service;
