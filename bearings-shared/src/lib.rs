//! bearings-shared — types shared across the bearings-rs workspace.
//!
//! Every crate in the workspace imports from here.
//! This is the single source of truth for the database schema in Rust.
//!
//! Usage:
//! ```toml
//! # In your Cargo.toml
//! bearings-shared = { path = "../bearings-shared" }
//! ```
//!
//! ```rust
//! use bearings_shared::models::Event;
//! use bearings_shared::enums::PlaceType;
//! ```

pub mod enums;
pub mod models;
