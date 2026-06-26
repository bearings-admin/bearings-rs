# Bearings

**Global bear community hub.** Events, venues, clubs, title holders, and more — worldwide.

Live: `https://srv1744879.hstgr.cloud` (custom domain `bearings.community` not yet assigned)

---

## What this is

Bearings is a Rust monorepo with three active crates and one parked:

| Crate | Role | Status |
|-------|------|--------|
| `bearings-shared` | Typed database models — shared across all crates | ✅ Active |
| `bearings-backend` | Axum HTTP server — SSR + REST API | ✅ Active |
| `bearings-agent` | Data pipeline — RSS/iCal feed ingestion | ✅ Active |
| `bearings-frontend` | Rust→WASM frontend scaffold | ⏸ Parked — framework undecided (see below) |

> The frontend stub is **parked**: excluded from the default build and CI so it doesn't
> gate work or bias the open framework decision. It stays a workspace member, so
> `cargo build -p bearings-frontend` still works. See "Frontend (undecided)" below.

---

## Running locally

```bash
git clone …
cd bearings-rs

# Copy environment config
cp .env.example .env
# Edit .env — needs SUPABASE_URL, SUPABASE_ANON_KEY, SUPABASE_SERVICE_ROLE_KEY, ADMIN_TOKEN

# Run the backend (the parked frontend crate is skipped by default)
cargo run -p bearings-backend

# Visit http://localhost:3000
```

The backend talks directly to Supabase via its REST API. No local database needed.

**Contributing:** `main` is the single source of truth and the VPS is deploy-only — work on a
branch, open a PR, and let CI merge it (no manual approval needed). See **`CONTRIBUTING.md`**.
A `justfile` wraps the common commands (`just backend`, `just test`, `just lint`, `just deploy`).

---

## Architecture

### bearings-shared

The single source of truth for the database schema in Rust. Every crate imports from here.

```
bearings-shared/src/
  models.rs   — Event, Place, Club, TitleHolder, Competition, Campaign, ...
  enums.rs    — EventType, PlaceType, ProposalStatus, ... (defined; models.rs still
                uses raw String fields — wiring the enums in is an open decision)
```

**If you touch the Supabase schema, update `models.rs` first.**

### bearings-backend

An Axum server. Two parallel sets of handlers:

```
src/
  lib.rs             — build_app() (router + middleware) + all module decls
  main.rs            — thin binary: env → config → bind port → serve
  config.rs          — env vars, validated at startup
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

### bearings-frontend (parked — framework undecided)

A Rust→WASM frontend scaffold exists but is **parked**. It was scaffolded with **Leptos**,
but the framework choice is **not settled** — see "Frontend (undecided)" below. It's excluded
from the default build and CI so it doesn't gate work or bias that decision.

```
bearings-frontend/src/
  lib.rs                 — App component, Shell, Nav
  components/
    now.rs               — NowPage + EventCard (calls /api/now)
    coming_up.rs         — ComingUpPage placeholder
```

Whatever framework is chosen, it will **consume the existing typed JSON API in `routes/`** and
**keep SSR-for-crawlers** (the `/llms.txt` + SSR HTML are a core differentiator).

---

## The migration path (current → richer frontend)

```
Phase 1 (done): Axum SSR — zone functions return Html<String>
Phase 2 (done): Type safety — Zone enum, typed models, bearings-shared
Phase 3 (open): Reactive frontend — framework UNDECIDED (Leptos vs Yew vs stay-HTMX)
Phase 4:        WASM hydration — once a framework is chosen
```

Today the live UI is **Axum SSR + HTMX** (no WASM) — fast and crawlable. A richer reactive
frontend is Phase 3, but the framework is an open call (see below). Whatever it is, it consumes
the JSON API in `routes/`.

---

## Database

Supabase (PostgreSQL) via PostgREST REST API. No ORM. Queries are explicit strings in the route handlers.

Key tables: `events`, `places`, `clubs`, `title_holders`, `competitions`, `campaigns`, `watched_feeds`, `candidate_events`, `artifacts` (provenance evidence), `kindred_sources` (credited resources), `agent_actions` (keeper audit log)

Key views: `current_title_holders` (one row per title), `event_predictions`/`event_series` (recurrence forecast), `event_backfill_targets` (keeper backfill worklist), `cause_contributions`/`charity_impact`/`charity_lineage` (unified charity model)

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

Feed types: `rss`, `ical`, `ical-static` (annual URL refresh needed each January). The
nightly run also archives past events and emails a digest (Resend).

A second worker, **the keeper** (`scripts/keeper.py`, weekly `bearings-keeper.timer`), is an
AI agent (Anthropic API) that confirms forecasted event dates from official sites and
auto-applies slam-dunk confirmations (audited, reversible); a `KEEPER_MISSION=backfill` run
web-searches past editions of recurring events. See `RESEARCH_DIRECTIVE.md` (data strategy)
and `AGENT_TEAM.md` (agent roster + roadmap).

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

The VPS (`srv1744879.hstgr.cloud`) is **deploy-only** — never hand-edited. It holds a checkout
at `/opt/bearings-rs` that is kept in sync with GitHub `main`. To ship a merged change, run the
deploy script **on the VPS**:

```bash
ssh root@srv1744879.hstgr.cloud
cd /opt/bearings-rs && ./deploy.sh
```

`deploy.sh` fetches `origin/main`, hard-resets the checkout, `cargo build --release -p
bearings-backend`, restarts the service, and health-checks it. Develop on a branch and let CI
merge the PR first (see `CONTRIBUTING.md`); deploy reflects whatever is on `main`.

Systemd service: `bearings-backend.service`
TLS: **Caddy** reverse proxy + Let's Encrypt → `localhost:3000`.

---

## Frontend (undecided)

The reactive frontend framework is an **open decision** — it has **not** been settled:

- **Stay SSR + HTMX** — what ships today; zero new toolchain, great for crawlers. Ceiling on rich interactivity.
- **Leptos** — what the parked stub was scaffolded with; modern, strong SSR+hydration story.
- **Yew** — a different Rust→WASM framework (the maintainer's day-to-day stack).

The stub currently uses Leptos 0.6 crates, but that is **not** a commitment — it's parked
precisely so it doesn't pre-empt this call. Hard constraints for whatever wins: **keep
SSR-for-crawlers** (`/llms.txt` + SSR HTML), **reuse `bearings-shared` + the REST API**, and
honour the design system / i18n. Full context and the decision brief live in
**`FRONTEND_BRIEF.md`**.
