
# bearings-rs — Design Intent

**Last updated:** 2026-06-07
**Author:** ursasteward@pm.me
**For:** Next Claude session, Gaspar, any future contributor

This document captures every design decision made so far.
Read this before touching any file. Load alongside the Supabase documents table.
For the zoom-out architectural view: see ARCHITECTURE.md in this same crate.

---

## Session start (new Claude chat)

```sql
SELECT content FROM documents WHERE slug = 'directive';
SELECT content FROM documents WHERE slug = 'state';
SELECT content FROM code WHERE crate = 'bearings-rs' AND file_path = 'DESIGN.md';
```

---

## Frontend Status (updated 2026-06-07)

**Lovable is no longer the frontend.** bearings.lovable.app is dead.

The frontend path is now: **Gaspar + Hostinger VPS**.

Gaspar is a senior Rust developer with his own deployed Axum app (confessery.freygas.com).
The frontend decision is his to make. Two options are prepared and documented — he picks one,
no rework happens either way. See ARCHITECTURE.md § Frontend Decision for full detail.

**Option A — HTMX + Tera/Askama (fastest path)**
- ssr.rs serves HTML to all visitors today — it works right now
- Gaspar adds hx-get attributes for interactivity, Tailwind CDN for styling
- No build pipeline changes, no new tooling
- Best if he wants to ship fast and iterate

**Option B — Leptos WASM (bearings-frontend crate)**
- bearings-frontend is already stubbed in the workspace
- ssr.rs reverts to crawler-only; Leptos handles browsers
- Requires cargo-leptos + wasm32-unknown-unknown target
- Best if he wants 100% Rust and is comfortable with the WASM toolchain

**Until he decides:** ssr.rs serves HTML to all browsers. The site is functional.
Do not re-add the LOVABLE_URL redirect or the crawler gate.

---

## What This Is

The Rust backend for Bearings — global bear community infrastructure.
This backend IS the frontend now. No separate React/Lovable layer.

**Database:** Supabase PostgreSQL (mntdhflffhrjjvipxgyl.supabase.co)
**VPS:** Hostinger (steward account — Ubuntu 24.04, ~$5/month) — LIVE at srv750649.hstgr.cloud
**Deployment:** Two systemd services on VPS behind nginx reverse proxy
**Design system:** Variant G — bear flag palette (brown/orange/gold/tan/white/grey/black)
**Frontend:** Inter font, mobile-first, 375px base. Bottom nav 5 items.
**Gaspar demo:** confessery.freygas.com — reference for his style and preferred stack

---

## The Four Zones (information architecture)

```
NOW
  ├── Hero event card (most imminent bear run)
  ├── Timeline bar chart (event density by month, scrollable horizontal)
  ├── Stat row (121 events / 170 places / 49 clubs / 87 title holders)
  ├── Quick card scroller (swipeable: title, this week, campaign, run, cruise)
  └── Treasury block (Base/USDC lights wallet balance, bear flag stripe top)

COMING UP
  ├── Event list (filterable by continent, type, month)
  └── iCal export — /api/events/ical.ics (RFC 5545, subscribable by URL)

BEAR ARCHIVES
  ├── Timeline spine (1987–present, decade markers)
  ├── Title holder archive (87 records, 32 competitions, IBR 1992–2011 complete)
  ├── Club histories (49 clubs, validator contacts stored on records)
  └── Oral histories (stories table — currently empty, needs content)

BEAR FUTURE
  ├── Funding proposals (community causes, progress bar)
  ├── Funded (tx_hash links to a Base block explorer)
  └── Operational ledger (every USDC movement, public on-chain transparency)
```

---

## Crate Architecture

```
bearings-shared    →    bearings-backend   (Axum HTTP server)
                   →    bearings-agent     (Treasury monitor + governance)
                   →    bearings-frontend  (Leptos WASM — Option B, stubbed, not started)
```

**bearings-shared** is the single source of schema truth.
All database structs live here. Every other crate imports from it.
If the Supabase schema changes, change it here and the compiler
tells you everywhere that breaks. All structs have `#[derive(Default)]`.

**bearings-backend** is the Axum HTTP server.
- 34 REST API routes (see main.rs route map)
- 8 SSR HTML pages (ssr.rs — now serves all browsers, not crawler-only)
- iCal export — RFC 5545, filterable by country/month/type
- POST /api/submissions — CONST-9 fallback intake with injection scanning
- llms.txt + llms-full.txt — AI agent discovery
- Privacy middleware (CONST-6) — centralised in middleware.rs
- Config validated at startup in config.rs

**bearings-agent** is the research + Bluesky publishing scaffold.
- The treasury is a single Base/USDC wallet the steward funds and spends manually, so
  there is no autonomous on-chain monitor.
- Bluesky posts always require steward review (CONST-10) — the agent drafts, never posts.
- NEVER holds private keys

---

## Key Design Decisions

### 1. No ORM — direct Supabase REST
`GET /rest/v1/tablename?column=eq.value` via reqwest.
`db.get_json::<T>(url)` is the only abstraction.
`db.post_rpc::<B,T>(rpc_name, body)` for stored functions (places_nearby).
`db.write_json_returning()` for inserts that return the created row.

### 2. Shared reqwest::Client on each struct
`SupabaseClient` (backend) and `SupabaseWriter` (agent) both store the Client.
Creating a new Client per request creates a new TCP connection pool — expensive.

### 3. Route ordering: static before parameterised
Axum 0.7 matches top-to-bottom. `/api/events/by-month` must be
registered BEFORE `/api/events/:id` or the static route is swallowed.
Same for `/api/places/nearby` before `/api/places/:id`.
Same for `/api/title-holders/current` before any `:id` param.

### 4. SSR serves all visitors (Lovable is dead)
Previously: browsers → redirect to bearings.lovable.app. Crawlers → HTML.
Now: everyone → HTML from ssr.rs.
The crawler detection code and LOVABLE_URL constant have been removed.
When Gaspar confirms Option B (Leptos), restore crawler-only for ssr.rs.

### 5. Privacy mode is centralised
CONST-6 (cannot be overridden by any vote or user request).
Full country list lives in middleware.rs `CRIMINALISED_COUNTRIES`.
Source: ILGA World annual report, updated annually.
`country_is_criminalised()` is called from submissions.rs.

### 6. Enums exist but models use String
`enums.rs` defines `EventType`, `PlaceType`, `WalletType`, `ProposalStatus`,
`ContributorTier`. Route handlers use raw String for PostgREST flexibility.
Strategy: use enums for validation in new code, convert models.rs in Phase 2.
Gaspar should weigh in on approach.

## Treasury

A single self-custodial **Base/USDC** wallet keeps the lights on. The steward tops it
up and pays infrastructure invoices manually, one at a time; the agent never holds
keys. Balance, chain, and address live in `platform_settings` (`lights_wallet_*`) and
surface on the Transparency zone. When a second steward joins, the runway can move to a
**Safe** multi-sig (CONST-2). Every movement is recorded in `operational_ledger` (USDC).

---

## Governance (deferred)

Governance is intentionally not built yet. If Bearings ever needs to outlive its
founding steward, or grows large enough to need shared control, control moves to the
community then. A DAO is one plausible model — but the tooling and the wider
crypto/agent landscape will have changed by the time it is needed, so the mechanism is
left open rather than specified now. There is no token. Community voice today is via
submission and discussion.

---

## Competition Archive

| Competition | Years | Status |
|-------------|-------|--------|
| International Mr Bear (IBR, SF) | 1992–2011 | ✅ Complete — 20 years |
| North American Bear (NAB, Lexington) | 2012–2025 | ✅ Mostly — 2026 winner TBC |
| Mr Bear International (Bangkok) | 2024–2026 | ✅ Complete — 3 editions |
| Mr TBRU (Dallas) | 1995–present | ⚠️ Only TBRU 27 confirmed |
| Mr Bear UK | 1990s–present | ❌ Zero records |

**Validators stored on competitions.contact_email:**
- NAB gaps: nabweekend@gmail.com (Adam Rodriguez-Routt)
- TBRU: contest@tbru.org
- Mr Bear UK: mrbearuk.info

---

## What Gaspar Should Review

Priority order:

1. **Frontend decision** — HTMX (Option A) or Leptos (Option B)?
   Both are prepared and documented. See ARCHITECTURE.md § Frontend Decision.
   ssr.rs currently serves HTML to all — functional right now.

2. `bearings-shared/src/models.rs`
   - Are Option<> wrappers appropriate for nullable columns?

4. `bearings-shared/src/enums.rs`
   - Should models use enums or raw String? What is the Rust idiom here?

5. `bearings-backend/src/db.rs`
   - Is storing reqwest::Client on the Axum State correct?

---

## Pending Human Actions (before GitHub)

1. **Create the Base/USDC "lights" wallet** (any self-custodial signer wallet)
   Set lights_wallet_address / lights_wallet_chain / lights_wallet_balance_usd in platform_settings
   Unlocks: treasury display on the Transparency zone

2. **Create bearings-rs GitHub repo** (public, bearings-admin/bearings-rs)
   Paste files from code table into repo
   Unlocks: Gaspar review, CI builds, binary artifacts

3. **Execute user_preferences ALTER TABLE**
   Run deploy/sql/user_preferences_wallet.sql in Supabase SQL editor

4. **Execute submissions table creation**
   Run deploy/sql/submissions_table.sql in Supabase SQL editor

5. **Mr Ottawa Bear 2023 and 2024 winner names** (steward knows personally)

6. **Mr Bear Europe 2026 winner** (competition: Lisbon, July 15–19)
