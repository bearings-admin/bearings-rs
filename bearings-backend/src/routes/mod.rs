//! Route modules — each file handles one resource.
//! Adding a new resource: create the file, declare it here, add routes in main.rs.

pub mod bear_future;
pub mod campaigns;
pub mod clubs;
pub mod coming_up; // COMING UP zone — trip planner composite endpoint
pub mod competitions;
pub mod creators; // BEAR ARCHIVES — musicians, filmmakers, illustrators
pub mod digital_spaces; // NOW — apps, Discord, podcasts, Twitch
pub mod events;
pub mod flags; // CONST-10 — inclusion flags reference + flagged events
pub mod future_ideas; // BEAR FUTURE — idea upvotes
pub mod history;
pub mod ical; // COMING UP — RFC 5545 iCal export
pub mod now; // NOW zone — composite here-and-now endpoint
pub mod places;
pub mod stories; // BEAR ARCHIVES — oral histories
pub mod submissions; // Public write — CONST-9 fallback intake
pub mod titles;
