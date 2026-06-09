
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
  └── Treasury block (community ADA + operational ADA, bear flag stripe top)

COMING UP
  ├── Event list (filterable by continent, type, month)
  └── iCal export — /api/events/ical.ics (RFC 5545, subscribable by URL)

BEAR ARCHIVES
  ├── Timeline spine (1987–present, decade markers)
  ├── Title holder archive (87 records, 32 competitions, IBR 1992–2011 complete)
  ├── Club histories (49 clubs, validator contacts stored on records)
  └── Oral histories (stories table — currently empty, needs content)

BEAR FUTURE
  ├── The Pot (treasury_balance_ada, operational_balance_ada, NORTH count)
  ├── Active proposals (vote_yes / vote_no / progress bar)
  ├── Funded (tx_hash links to Cardanoscan)
  └── Operational ledger (every ADA movement, public on-chain transparency)
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

**bearings-agent** is the treasury monitor + governance agent.
- Hourly: check both Cardano wallets for new inbound transactions
- Monday 03:00 UTC: update treasury balance snapshot in platform_settings
- NORTH token minting — Phase 1 manual, Phase 4 autonomous
- Embedded wallet onboarding — email-only, custodial → self-custody exit
- x402 payments — Phase 4 ONLY, guarded by check_phase_before_payment()
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

### 7. Blockfrost via REST not crate
`blockfrost-rs` requires nightly. We call the REST API directly.
`net_received()` handles change outputs correctly.
`headers()` rebuilds HeaderMap per call — cheap, acceptable.

### 8. x402 payments are Phase 4 ONLY
`check_phase_before_payment()` reads `treasury_phase` from platform_settings
and returns Err if < 4. Called at top of every payment function.
Community treasury NEVER touched by autonomous agent — constitutional.

---

## NORTH Token

**Name:** NORTH
**Metaphor:** Compass bearings measured relative to magnetic north.
More NORTH = more sway. Follow the NORTH.
**Chain:** Cardano native asset
**Distribution:** 1 per verified role (title holder, club officer, etc.)
**Non-transferable** during bootstrapping
**DAO threshold:** 100 verified holders unlocks full governance
**Current holders:** 0

**Minting flow (Phase 1):**
1. Bear submits claimed role via web form
2. `log_mint_request()` creates unverified row in governance_token_holders
3. Steward reviews against public records (BWM, competition archive)
4. `confirm_mint()` sets verified=true, token_balance=1
5. Bear receives email: you can now vote

**Wallet onboarding:**
- Email only — no crypto knowledge needed
- Custodial wallet placeholder created in Phase 1
- Bear can connect Eternl/Lace for self-custody exit
- `wallet_type`: "custodial" | "self-custody" | "both"

---

## Treasury Phases

| Phase | Model | Status |
|-------|-------|--------|
| 1 | Steward holds keys, manual | **Current** |
| 2 | Multi-sig 2-of-3 elected keyholders | Next milestone |
| 3 | NORTH vote triggers release, steward executes | Post-bootstrapping |
| 4 | Agent autonomous via x402 Protocol | Long-term |

`treasury_phase` in platform_settings controls which phase is active.
Advancing requires a governance vote.

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
   - Should ADA amounts be f64 or u64 lovelace with conversion?

3. `bearings-agent/src/blockfrost.rs`
   - Is net_received() correct for complex transactions?

4. `bearings-shared/src/enums.rs`
   - Should models use enums or raw String? What is the Rust idiom here?

5. `bearings-backend/src/db.rs`
   - Is storing reqwest::Client on the Axum State correct?

6. `bearings-agent/src/treasury.rs`
   - Any concerns about the hourly loop structure?

---

## Pending Human Actions (before GitHub)

1. **Create TWO Cardano wallets** (Eternl or Lace)
   Set treasury_wallet_ada and operational_wallet_ada in platform_settings
   Unlocks: Bear Future zone, treasury display

2. **Create bearings-rs GitHub repo** (public, bearings-admin/bearings-rs)
   Paste files from code table into repo
   Unlocks: Gaspar review, CI builds, binary artifacts

3. **Execute user_preferences ALTER TABLE**
   Run deploy/sql/user_preferences_wallet.sql in Supabase SQL editor

4. **Execute submissions table creation**
   Run deploy/sql/submissions_table.sql in Supabase SQL editor

5. **Mr Ottawa Bear 2023 and 2024 winner names** (steward knows personally)

6. **Mr Bear Europe 2026 winner** (competition: Lisbon, July 15–19)
