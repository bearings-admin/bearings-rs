# Bearings White Paper

**Version:** 0.9  **Date:** June 2026  **Status:** Current — governance model under active discussion

Bearings is global infrastructure for the gay bear community: a verified, living
directory and coordination layer for events, places, clubs, title holders,
creators, digital spaces, campaigns, and community history. It is an
unincorporated association; the steward's identity is held privately.

Bearings is **not** a dating app, a social network, a magazine, or an event
promoter. Its most valuable asset is temporal + geographic navigation —
*"when and where do you want to meet?"* — and everything is organised around that.

---

## §1 Purpose and Identity

Community memory preservation is the primary purpose: keep the record accurate and
current, prioritise it over growth. Privacy is non-negotiable, especially for
members in countries where being gay is criminalised. Governance is lightweight by
default. If revenue is ever generated, it serves the community — no individual
extraction, all affiliate relationships disclosed.

---

## §2 Information Architecture

Four zones on a temporal spine:

- **Archive** — community memory since 1987. Decade tabs, title-holder lineage by
  competition, oral histories.
- **Now** — what's happening today: hot events, current title holders, live campaigns.
- **Coming Up** — the default landing. A 6–24 month trip planner; monthly bar chart
  filters by click; country dropdown; iCal export.
- **Future** — active campaigns (progress bars), recent milestones (Breaking Ground),
  new bear territories (with ILGA safety context), and community-upvoted ideas.

**Navigation:** four-icon bottom nav (pure CSS, no JS), a hamburger directory drawer,
and an EN/ES/FR language switcher carried through every link. Interface text is
translated at startup; community data uses the browser's `lang` attribute for
on-the-fly translation.

**Data as of June 2026:** 88 events, 173 places (saunas, bars, campgrounds, leather
bars), 49 clubs, 33 competitions, 93 title holders, 58 history entries, 35 creators,
33 digital spaces, 11 campaigns, ~1,190 translation rows.

---

## §3 Governance (under discussion)

**The open question (steward + Gaspar):** does Bearings become a for-profit company,
or remain free community infrastructure with separately-incubated commercial ventures?
This decision gates the governance architecture.

- **Path A — free community infrastructure.** Funded by voluntary contributions,
  grants, and minimal affiliate income. Lightweight governance: the steward makes
  operational decisions; the community has voice via submission and discussion. No
  token required.
- **Path B — community-owned platform.** A NORTH token on Cardano supports community
  voting — one per verified role (former title holder, club officer, organiser),
  non-transferable during bootstrapping, with a 100-verified-holder threshold to
  unlock DAO governance (60% operational / 75% constitutional).

**Current default is Path A** until decided. The token/treasury machinery described
below is exploratory architecture, not a commitment — it can be activated under Path B
without disrupting operations.

---

## §4 Treasury and Finance

Today the steward holds all operational credentials; there is no on-chain treasury.
An `operational_ledger` table tracks income and costs for transparency. Under Path B,
a two-wallet Cardano model (Community Treasury + Operational) would phase in from
manual steward control through multi-sig to governance-triggered and eventually
autonomous (x402) payments — with the agent never holding private keys until the
final phase. Affiliate links are disclosed and logged under either path.

---

## §5 Content Model

Verified data only; a source is required for every record. Gaps are documented, not
guessed (`holder_status = 'unknown'`). The **Archive Principle** is constitutional:
nothing is deleted — `active = false` preserves community memory permanently.
Submissions from criminalised countries activate `privacy_mode` (cannot be overridden
by vote or directive).

Competition archive completeness: IBR (San Francisco, 1992–2011) complete; NAB Weekend
mostly complete; Mr Bear International (2024–) complete; Mr Bear UK and Mr TBRU partial,
outreach pending.

---

## §6 Research and Data Collection

Real-time scraping during page renders has been retired (it caused IP blocks). The
approach now: official APIs first (Eventbrite, Meetup, iCal feeds), a nightly cron on
the VPS (systemd timer), local validation before insert, and community submission via
the chatbot intake and the `submissions` table. Dedup before every insert; never
insert a record without a source.

---

## §7 Technical Stack

**Phase 1 (live):** a Lovable React frontend (`bearings.lovable.app`) over Supabase
PostgreSQL — the original rapid-start UI.

**Phase 2 (live):** the `bearings-rs` Rust workspace on a Hostinger VPS, served over
HTTPS at `https://srv1744879.hstgr.cloud/` (Caddy reverse proxy + automatic Let's
Encrypt TLS in front of the app on port 3000). Source:
`github.com/bearings-admin/bearings-rs`.

The backend (`bearings-backend`, Axum) is built in clean layers:

```
routes / ssr  →  services  →  repositories  →  db (Supabase PostgREST)
```

- **Routes & SSR zones** are thin: parse the request, render or return JSON.
- **Repositories** own all data access — one trait + one implementation per resource,
  so the database is swappable without touching the rest of the app. User-supplied
  filter values are percent-encoded in a single place (injection-safe).
- **Services** hold business logic (e.g. governance voting), unit-tested against fake
  repositories with no database.
- **Server-rendered HTML + HTMX** (not a WASM SPA): immediate first paint, no bundle
  to download or hydrate. A 30-second in-memory cache fronts every read, so repeat
  page loads skip the network round-trip entirely (≈45 ms → ≈0.5 ms).
- **Security:** every rendered value is HTML-escaped (XSS-safe), including
  public-submitted content.

Supporting crates: `bearings-shared` (typed models), `bearings-agent` (treasury
monitor and Bluesky publishing stubs), and `bearings-frontend` (a Leptos skeleton for
the Phase 3 interactive frontend).

Design decisions and benchmarks — including Axum vs Rocket, PostgREST vs a direct
`sqlx` connection, and HTMX vs a WASM SPA — are documented in
`bearings-backend/ARCHITECTURE.md`.

---

## §8 Bear Future Zone

Four live sections: **Bears Taking Action** (campaigns with progress bars),
**Breaking Ground** (recent title holders, including historic firsts), **New Bear
Territories** (regional safety reference via ILGA; notes on Malaysia, the Middle East,
Eastern Europe), and **What Could Be** (community-upvoted ideas). A `bear_regions`
table for per-region safety commentary is planned, pending steward content.

---

## §9 Portability

No vendor lock-in. The database is standard PostgreSQL — "Supabase" is a managed
Postgres plus a REST layer (PostgREST), and the backend uses none of Supabase's
proprietary features (no Auth, Storage, or Realtime). The full schema (tables,
constraints, functions, triggers, views, RLS policies) is captured in the repository
at `supabase/schema.sql`, with a README documenting an authoritative `pg_dump` and the
steps to move to any Postgres host.

Because data access is isolated in the repository layer, porting the database is a
connection-string change. A future move to `sqlx` (a direct, compile-time-checked SQL
connection) would also remove the REST hop — an option weighed in `ARCHITECTURE.md`.
The data itself is kept out of the repository by design.

---

## §10 Constitutional Values

Ten values (CONST-1…10) govern the project and require a 75% supermajority to amend:
community-memory-first, no single point of human failure, lightweight governance,
multilingual operation, revenue-serves-community, non-negotiable privacy, do-not-
compete-with-partners, content-freshness-as-obligation, conversational-intake-first,
and *inclusion is shown, not decided* (listings are never removed for being
exclusionary — they are flagged with context and an inclusive alternative). Full text
lives in the agent directive (`CLAUDE.md`).
