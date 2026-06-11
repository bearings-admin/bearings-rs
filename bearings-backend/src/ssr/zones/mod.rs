//! Zone submodules — each zone is a separate file.
//!
//! All zones export a single `zone_*` async function that:
//!   1. Fetches data from Supabase via the shared SupabaseClient
//!   2. Builds an HTML response using helpers from `crate::ui`
//!   3. Returns the result wrapped in `crate::ui::shell()`
//!
//! To add a new zone:
//!   1. Create `src/ssr/zones/my_zone.rs` with `pub(crate) async fn zone_my_zone(...)`
//!   2. Declare it here: `pub mod my_zone;`
//!   3. Add a `Zone::MyZone` arm to the `Zone` enum in `src/ssr/mod.rs`
//!   4. Wire it in the `root()` dispatcher match in `src/ssr/mod.rs`

pub mod now;
pub mod coming_up;
pub mod archive;
pub mod future;
pub mod places;
pub mod events;
pub mod clubs;
pub mod titles;
pub mod creators;
pub mod campaigns;
pub mod ical;
pub mod digital;
pub mod admin;
