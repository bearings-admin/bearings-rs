
# bearings-rs — Architectural Alignment

**Last reviewed:** 2026-06-07
**Purpose:** This document maps the white paper's stated architecture against
what is actually in the database and the Rust code. It is the zoom-out view.
Read DESIGN.md for implementation detail. Read this to understand coherence.

---

## The Three Layers

```
WHITE PAPER (intent)
      ↕  alignment gaps documented below
DATABASE (Supabase — 32 tables, 10 views, 12 AI summary views)
      ↕  schema drift fixed 2026-06-06, monitored going forward
RUST CODE (bearings-rs — 57 files, 4 crates)
```

The database is more advanced than the Rust code in several areas.
The Rust code has more deployment infrastructure than the database has security.
The white paper is broadly accurate but predates some DB evolution.

---

## Frontend Decision (added 2026-06-07)

**Lovable is dead.** The frontend is now Gaspar + Hostinger VPS.

Gaspar is a senior Rust developer. His existing work (confessery.freygas.com)
is a deployed Rust-backed web app — minimal, functional, real.
The decision below is his to make. Both options are fully prepared.

### Option A — Axum + HTMX + Templates (fastest path, no new tooling)

**How it works:**
- `ssr.rs` is already serving HTML to all browsers (as of 2026-06-07)
- Gaspar adds a template engine: Tera (Jinja2-style .html files) or Askama (compiled .rs)
- HTMX CDN script enables fragment swapping without page reloads
- Tailwind CDN or build step handles styling
- The existing API routes are called directly via hx-get attributes

**What changes in bearings-rs:**
- Add `tera` or `askama` to Cargo.toml
- Move html_page() in ssr.rs to actual template files
- Add HTMX attributes to the HTML for the two-lens filters (WHERE + WHEN)
- bearings-frontend crate stays stubbed — not activated

**Tradeoffs:**
- Ships fastest — probably days not weeks
- No WASM, no new Rust toolchain targets
- Interactivity is limited to HTMX's fragment model (enough for Bearings)
- No JS needed beyond one CDN script tag

### Option B — Leptos WASM (bearings-frontend crate)

**How it works:**
- The `bearings-frontend` crate in the workspace is activated
- Gaspar builds reactive Leptos components that call the Axum API
- Compiles to WASM, runs in the browser
- `ssr.rs` reverts to crawler-only detection (AI/search bots get HTML, browsers get WASM)

**What changes in bearings-rs:**
- Add `cargo-leptos` tooling, `wasm32-unknown-unknown` target
- Build out bearings-frontend/src/ with Leptos components
- Restore `is_crawler()` and redirect browsers to the Leptos SPA URL
- bearings-backend stays as pure API (no HTML responsibilities)

**Tradeoffs:**
- 100% Rust — Gaspar's natural habitat
- More setup cost: WASM build pipeline, cargo-leptos, wasm-pack or trunk
- Reactive components feel more natural for the two-lens trip planner UI
- Higher ceiling for interactivity

### Current state while decision is pending

`ssr.rs` serves HTML to ALL visitors (removed crawler gate and LOVABLE_URL).
The site is functional and navigable right now:
- /events, /places, /clubs, /titles, /creators, /history, /digital-spaces all serve real data
- Full nav bar across all pages
- Social handles and hours included in places page
- Hot badges on events

When Gaspar decides:
- Option A: build templates on top of what's there
- Option B: restore crawler gate, activate bearings-frontend

---

## White Paper → Database Alignment

### ✅ Fully aligned

| White paper concept | Database table | Notes |
|---------------------|---------------|-------|
| Events (NOW + COMING UP) | events (36 cols) | DB has additional fields: hot, slug, event_mode, stream_url, inclusion_flag_codes |
| Places (bars, saunas, campgrounds) | places (57 cols) | Most complete table in the schema. All 170 records now have addresses. |
| Clubs | clubs (31 cols) | validator_notes, outreach_status implemented |
| Title holders | title_holders (20 cols) | IBR 1992-2011 complete. 87 total records. |
| Competitions | competitions (25 cols) | Active + archived support |
| Bear history | bear_history (15 cols) | 1987-present |
| Campaigns | campaigns (16 cols) | privacy_mode implemented |
| NORTH token holders | governance_token_holders (17 cols) | cardano_wallet, verified, authorization_phase |
| Community proposals | bear_future_proposals (29 cols) | Full voting scaffold: threshold, window, steward review |
| Proposal votes | proposal_votes (7 cols) | vote_weight = NORTH balance at vote time |
| Treasury ledger | operational_ledger (16 cols) | authorization_phase, donor_display, donor_wallet |
| Agent directives | documents + document_archive | v0.7 live, v0.1-v0.6 archived |
| Agent code storage | code (10 cols) | Full bearings-rs workspace |
| Platform config | platform_settings | treasury_phase, bear_future_active, NORTH metadata |

### ✅ In database, NOT in white paper (DB ahead of plan)

| Table | What it is | Implication |
|-------|-----------|-------------|
| agent_inbox | Inbound Bluesky/social mentions | Bluesky pipeline is further designed than white paper describes |
| agent_posts | Outbound posts with steward review gate | CONST-10 circuit breaker is implemented at DB level |
| inclusion_flags | Reference table for CONST-10 flag codes | MASC_ONLY, AGE_RESTRICTION etc — more sophisticated than white paper implies |
| media | Films, albums, podcasts linked to creators | Creator zone is richer than described |
| stores | Bear merchandise stores | White paper doesn't mention stores — legacy or planned feature |
| sponsors | Event sponsors | Affiliate/revenue model at DB layer |
| translations | i18n key-value store | Seven-language support (CONST-4) has DB infrastructure |
| creator_event_links | M-M creators ↔ events | Relational links the white paper describes but doesn't detail |
| digital_space_event_links | M-M digital spaces ↔ events | Same |
| event_place_links | M-M events ↔ venues | Same |

### ⚠️ In white paper, NOT yet in database

| White paper concept | Status | Notes |
|--------------------|--------|-------|
| iCal personal subscriptions | Rust planned, no DB token table | Needs a `calendar_subscriptions` table |
| Bluesky handle per user | On events table only | Needs user-level social handle |
| Newsletter | newsletter_subscribers exists | But no campaign/send table yet |

---

## Database → Rust Code Alignment

### ✅ Tables with Rust models (updated 2026-06-06)

Event, Place, Club, TitleHolder, Competition, BearHistory, Campaign,
BearFutureProposal, GovernanceTokenHolder, OperationalLedger,
Document, Code — all synced with verified DB schema.

**New models added 2026-06-06:** AgentInbox, AgentPost, ProposalVote,
InclusionFlag, Media.

### ⚠️ Tables with NO Rust models yet

| Table | Priority | Used by |
|-------|----------|---------|
| stores | Low | No current API route — legacy |
| stories | Medium | BEAR ARCHIVES oral history zone |
| sponsors | Low | Revenue model — future |
| translations | Medium | CONST-4 seven languages |
| newsletter_subscribers | Low | Marketing layer |
| user_preferences | High | Embedded wallet onboarding, contributor tiers |
| creator_event_links | Medium | Relational joins |
| digital_space_event_links | Low | Relational joins |
| event_place_links | Low | Relational joins |
| sponsor_event_links | Low | Revenue model |

### ⚠️ API routes with NO Rust implementation yet

| Route | DB table | Four-zone |
|-------|---------|-----------|
| GET /api/stories | stories | BEAR ARCHIVES oral histories |
| GET /api/stores | stores | Utility |
| GET /api/media | media | BEAR ARCHIVES creator zone |
| POST /api/votes | proposal_votes | BEAR FUTURE |
| GET /api/events/with-flags | events + inclusion_flags | CONST-10 |
| GET /api/agent/posts | agent_posts | Bluesky pipeline |
| GET /api/agent/inbox | agent_inbox | Bluesky pipeline |

---

## Database Views the Rust Code Ignores

The DB has 10 pre-built views the Rust code doesn't use at all.

| View | What it does | Route it could power |
|------|-------------|---------------------|
| current_title_holders | Most recent holder per competition | /api/title-holders/current (replace Rust dedupe) |
| competition_history | Full winner lineage per competition | /api/competitions/:id/history |
| events_with_flags | Events with inclusion_flag details joined | /api/events (add flag data) |
| places_with_flags | Places with inclusion_flag details | /api/places (add flag data) |
| places_near_events | Venues near upcoming events | /api/events/:id/nearby-places |
| ai_event_summary | Flattened plain text for AI consumption | /llms-full.txt (replace current approach) |
| ai_place_summary | Same for places | Same |
| ai_title_summary | Same for title holders | Same |
| ai_campaign_summary | Same for campaigns | Same |
| ai_history_summary | Same for history | Same |
| ai_creator_summary | Same for creators | Same |

**The ai_* views are important.** The `llms-full.txt` route currently fetches
raw events and formats them in Rust. The DB already has `ai_event_summary`
doing this better. The Rust route should query the view, not the raw table.

---

## Security Posture (fixed 2026-06-06)

### Fixed this session
- RLS enabled on: operational_ledger, bear_future_proposals,
  governance_token_holders, proposal_votes, documents, code, document_archive
- Public read policy on all — service role writes only
- bear_history RLS policy gap filled

### Remaining security notes (informational, not urgent)
- 11 SECURITY DEFINER views — this is Supabase's default for views,
  not a misconfiguration. The linter flags it but it's the correct pattern
  for read-only public views. No action needed.
- newsletter_subscribers / submissions / user_preferences INSERT policies
  have WITH CHECK (true) — this is intentional for public write tables.
  The linter flags it; it's correct. No action needed.

---

## The Bluesky Pipeline — A Whole Feature Layer

The database has a complete Bluesky publishing architecture that is NOT
described in the white paper and has NO Rust implementation:

```
DB:
  agent_posts          — draft/scheduled/published posts, steward review gate
  agent_inbox          — inbound mentions, intent classification
  events.bluesky_handle — per-event Bluesky handles

White paper §10 (CONST-10):
  "Inclusion is shown, not decided" — posts with flag context

Agent behaviour described in directive:
  "All posts require steward review during bootstrapping"
  "Circuit breakers: 4-hour post cooldown, reviewed_by_steward must be true"
```

When bearings-agent is extended to handle Bluesky, the models and DB are ready.
The Rust work needed: a `bluesky.rs` module in bearings-agent (stub already exists).

---

## Four-Zone Coverage Map

How much of each zone is covered by current Rust routes:

| Zone | Routes | DB tables | Coverage |
|------|--------|-----------|----------|
| NOW | events/list, events/by-month, treasury | events, platform_settings | 60% — missing digital_spaces, creators, hot flag query |
| COMING UP | events/list + upcoming_only, ical.ics | events | 80% — missing personal calendar subscriptions |
| BEAR ARCHIVES | history/list, titles/list, competitions/list, clubs/list | all | 70% — missing stories (oral histories), media, creator zone |
| BEAR FUTURE | proposals, funded, token_holders, ledger, treasury | all | 85% — missing POST /votes, proposal submission |

---

## Priority Work (in order)

0. **Frontend decision (Gaspar)** — HTMX or Leptos?
   See § Frontend Decision above. Both are ready. Site works today either way.

1. **llms.txt upgrade** — Replace raw event fetch with ai_event_summary view.
   Add ai_place_summary, ai_title_summary, ai_campaign_summary.
   This makes Bearings dramatically more useful to AI assistants.

2. **POST /api/votes** — The voting infrastructure exists (proposal_votes table,
   vote_weight = token_balance) but the Rust endpoint doesn't. Bear Future
   is described as governance-capable in the white paper. Without this route
   it's read-only.

3. **stories route** — GET /api/stories. Oral histories. Empty table but the
   white paper explicitly promises this. Stories table schema is ready.

4. **inclusion_flags route** — GET /api/inclusion-flags + events_with_flags view.
   CONST-10 ("inclusion shown, not decided") is described as a constitutional
   requirement but has no API exposure.

5. **user_preferences route** — The wallet onboarding flow needs this.
   user_preferences exists (8 cols) but has no Rust model or route.
   NOTE: user_preferences_wallet.sql ALTER TABLE has not been run yet —
   run this in Supabase SQL editor first.

6. **Bluesky module** — bearings-agent/src/bluesky.rs. The DB infrastructure
   is complete. The agent directive describes it. Stub exists. Only implementation missing.

7. **Proximity sorting for /api/events** — UX.md describes this as the key
   improvement over Lovable. When lat/lng provided, sort by distance not date.
   The places_nearby RPC pattern is the model.
