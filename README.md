# Bearings

**Global bear community hub.** Events, venues, clubs, title holders, and more — worldwide.

Live: `https://www.bearings.community` (currently at `https://srv750649.hstgr.cloud`)

---

## What this is

Bearings is a Rust monorepo with three active crates and one pending:

| Crate | Role | Status |
|-------|------|--------|
| `bearings-shared` | Typed database models — shared across all crates | ✅ Active |
| `bearings-backend` | Axum HTTP server — SSR + REST API | ✅ Active |
| `bearings-agent` | Data pipeline — RSS/iCal feed ingestion | ✅ Active |
| `bearings-frontend` | Leptos SSR + WASM frontend | 🔧 Phase 3 |

---

## Running locally

```bash
git clone …
cd bearings-rs

# Copy environment config
cp .env.example .env
# Edit .env — needs SUPABASE_URL, SUPABASE_ANON_KEY, SUPABASE_SERVICE_ROLE_KEY, ADMIN_TOKEN

# Run the backend
cargo run -p bearings-backend

# Visit http://localhost:3000
```

The backend talks directly to Supabase via its REST API. No local database needed.

---

## Architecture

### bearings-shared

The single source of truth for the database schema in Rust. Every crate imports from here.

```
bearings-shared/src/
  models.rs   — Event, Place, Club, TitleHolder, Competition, Campaign, ...
  enums.rs    — EventType, PlaceType, PlaceType, ProposalStatus, ...
```

**If you touch the Supabase schema, update `models.rs` first.**

### bearings-backend

An Axum server. Two parallel sets of handlers:

```
src/
  main.rs            — router, port binding, middleware
  db.rs              — SupabaseClient (thin wrapper over reqwest)
  ui.rs              — design system: colour constants + HTML helpers
  mcp.rs             read-only MCP server (POST /mcp, JSON-RPC over HTTP)
  ssr/               — server-side rendered HTML (the current web UI)
    mod.rs           — ZoneQuery, Zone enum, dispatcher, legacy wrappers
    zones/           — one file per zone
      now.rs         — hot events + campaigns + title holders
      coming_up.rs   — trip planner (events by country/season)
      archive.rs     — bear history timeline
      future.rs      — BEAR FUTURE treasury + governance
      places.rs      — venues directory
      titles.rs      — competition archive + title holders
      clubs.rs       — bear clubs directory
      creators.rs    — content creators
      campaigns.rs   — community fundraising
      digital.rs     — digital spaces
      ical.rs        — iCal export info
      admin.rs       — feed candidate review (token-gated)
  routes/            — JSON REST API (consumed by mobile/frontend)
    now.rs           — GET /api/now
    coming_up.rs     — GET /api/coming-up
    events.rs        — GET /api/events
    places.rs        — GET /api/places
    titles.rs        — GET /api/title-holders
    ...
```

**Routing model:** `GET /` with `?zone=<name>` dispatches to the correct zone.
Every link in the HTML works without JavaScript. HTMX enhances navigation progressively.

**Design system:** All colour constants and HTML helpers live in `src/ui.rs`.
Each zone file starts with `use crate::ui::*;` to get them in scope.

**MCP server:** `POST /mcp` exposes the directory to AI agents as read-only MCP
tools (JSON-RPC over Streamable HTTP) — `search_events`, `list_places`,
`current_title_holders`, `list_clubs`, `list_creators`, `list_campaigns`,
`list_digital_spaces`. Hand-rolled in `src/mcp.rs`, no SDK dependency. Connect a
client:

```
claude mcp add --transport http bearings https://srv1744879.hstgr.cloud/mcp
```

### bearings-frontend (Phase 3)

The Leptos frontend is stubbed out and ready for activation:

```
bearings-frontend/src/
  lib.rs                 — App component, Shell, Nav
  components/
    now.rs               — NowPage + EventCard (server function wired to /api/now)
    coming_up.rs         — ComingUpPage placeholder
```

The server functions in `now.rs` call the same Supabase REST API that `ssr/zones/now.rs` uses.
Phase 3 is: wire up the remaining server functions → activate hydration → deprecate `ssr/`.

---

## The migration path (SSR → Leptos)

```
Phase 1 (done): Axum SSR — zone functions return Html<String>
Phase 2 (done): Type safety — Zone enum, typed models, bearings-shared
Phase 3 (next): Leptos SSR — replace zone functions with Leptos components
Phase 4:        WASM hydration — add wasm32-unknown-unknown target, cargo-leptos
```

Each `ssr/zones/X.rs` maps to a Leptos component in `bearings-frontend/src/components/X.rs`.
The JSON API in `routes/` is what the Leptos components will call.

---

## Database

Supabase (PostgreSQL) via PostgREST REST API. No ORM. Queries are explicit strings in the route handlers.

Key tables: `events`, `places`, `clubs`, `title_holders`, `competitions`, `campaigns`, `watched_feeds`, `candidate_events`

Key views: `current_title_holders` (deduplicated, one row per title, scope pre-joined)

The Supabase project is at `mntdhflffhrjjvipxgyl.supabase.co`.

---

## Data pipeline

A Python script (`scripts/feed_reader.py`) runs nightly via systemd:

```
bearings-feeds.timer — fires at 02:04 UTC
→ feed_reader.py    — fetches all active watched_feeds
→ candidate_events  — new events land here with status='pending'
→ /?zone=admin      — steward reviews and approves
```

Feed types: `rss`, `ical`, `ical-static` (annual URL refresh needed each January).
See `RESEARCH_DIRECTIVE.md` for the full tiered data collection strategy.

---

## Adding a zone

1. Create `src/ssr/zones/my_zone.rs`:
   ```rust
   use axum::response::{Html, IntoResponse, Response};
   use crate::{db::SupabaseClient, ui::*};
   
   pub(crate) async fn zone_my_zone(db: SupabaseClient, lang: &str) -> Response {
       // fetch from Supabase, build HTML, call shell()
       Html(shell("My Zone", "Description.", "my-zone", &body, lang)).into_response()
   }
   ```

2. Add `pub mod my_zone;` to `src/ssr/zones/mod.rs`

3. Add `Zone::MyZone` to the `Zone` enum in `src/ssr/mod.rs` and wire it in `root()`

4. Add `use zones::my_zone::zone_my_zone;` to `src/ssr/mod.rs`

---

## Environment variables

| Variable | Description |
|----------|-------------|
| `SUPABASE_URL` | Supabase project URL |
| `SUPABASE_ANON_KEY` | Public read key |
| `SUPABASE_SERVICE_ROLE_KEY` | Write key (feed ingestion, admin) |
| `ADMIN_TOKEN` | Token for `/?zone=admin` |
| `PORT` | Server port (default: 3000) |
| `RUST_LOG` | Log level (e.g. `bearings_backend=debug`) |

---

## Deployment

Single binary on a Hostinger VPS (`srv750649.hstgr.cloud`):

```bash
# Build release binary
cargo build -p bearings-backend --release

# Copy to VPS
scp target/release/bearings-backend user@vps:/opt/bearings-rs/bearings-backend.bin

# Restart service
ssh user@vps systemctl restart bearings-backend
```

Systemd service: `bearings-backend.service`  
TLS: Let's Encrypt via Hostinger auto-provisioning (expires Aug 2026)

---

## Frontend decision

The frontend crate uses **Leptos 0.6** (SSR feature, no WASM compilation required initially).

Leptos was chosen because it follows the same component model as Yew but with:
- Fine-grained signals instead of virtual DOM diffing
- `leptos_axum` for first-class Axum integration
- Better server function support (typed, async, serialized via serde)

To activate the full WASM hydration pipeline:
```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos
cargo leptos build
```
