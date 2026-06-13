# Bearings White Paper

**Version:** 0.10  **Date:** June 2026  **Status:** Current — governance model under active discussion

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
and an EN/ES/FR/JA language switcher carried through every link. Interface text is
translated at startup; community data uses the browser's `lang` attribute for
on-the-fly translation.

**Directory zones** (via the drawer): Places, Clubs, Title Holders, Campaigns and
Digital Spaces, plus two creator-facing zones — **Creators**, grouped by craft
(musicians, DJs, authors, illustrators, filmmakers, performers), where authors list
their books; and **Shops**, bear-owned shops and community gear, kept a separate zone
so commerce never crowds the directory.

**Data as of June 2026:** 88 events, 173 places (saunas, bars, campgrounds, leather
bars), 49 clubs, 28 competitions, 154 title holders, 52 history entries, 49 creators
(incl. international DJs and authors), 21 bear shops, 11 books, 35 digital spaces,
13 campaigns, 10 in-language bear-region profiles, ~1,190 translation rows across
four baked languages (EN/ES/FR/JA).

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

Today the steward holds all operational credentials and there is no on-chain
treasury; an `operational_ledger` records income and costs, and a **Transparency**
zone publishes operating costs, runway, and (once funded) the wallet balance.

**Near-term operational wallet (decided, pending setup with a second steward).** A
self-custodial wallet on **Base** (Coinbase's Ethereum L2) holding USDC, used to pay
infrastructure invoices (e.g. Hostinger) **manually via CoinGate**. The shape follows
from CONST-2 (no single point of human failure):

- A standard signer wallet (EOA) holds a small operating balance and settles invoices
  one at a time. A solo signer is the only sensible start — a single-owner multi-sig
  is all friction and no benefit.
- When a second steward joins (Gaspar or a backup), the treasury proper becomes a
  **Safe** smart-account **multi-sig** (e.g. 2-of-3) holding the runway. A Safe is a
  vault controlled by the stewards' signer wallets — recoverable by any authorised
  steward and auditable on-chain, which is exactly what CONST-2 asks for.
- **Affiliate and contribution income** is recorded in `operational_ledger`
  (`direction = in`) and shown against costs on the Transparency zone. Affiliates pay
  in fiat, so income reaches the wallet only as an explicit, logged conversion step —
  the ledger is the record, the wallet is the proof.
- **The agent never holds private keys.** Payments are manual and steward-authorised.

The Cardano two-wallet model and the NORTH governance token remain the **Path B**
option (§3), activatable later without disrupting this setup. Auto-renewing crypto
payments would need a crypto debit card and are out of scope for now.

---

## §5 Content Model

Verified data only; a source is required for every record. Gaps are documented, not
guessed (`holder_status = 'unknown'`). The **Archive Principle** is constitutional:
nothing is deleted — `active = false` preserves community memory permanently.
Submissions from criminalised countries activate `privacy_mode` (cannot be overridden
by vote or directive).

Competition archive completeness: IBR (San Francisco, 1992–2011) complete; NAB Weekend
mostly complete; Mr Bear International (2024–) complete; Mr Bear UK (2022–) complete; Mr TBRU partial, outreach pending. Belgium, Luxembourg
and Poland national lineages were added in-language with sources, and Mr. Bear Canada
(national), Mr Bear Japan, Mr Bear Norway and Mr Bear Portugal archived — no national
title exists (a duplicate, or club/event-anchored scenes).

**Commerce and affiliates (CONST-5).** Two link types, always disclosed. Bear-owned
shops and direct sellers are linked **directly** — no skim. Marketplace products
(currently books on Amazon) carry **affiliate links** shown with a plain-language
disclosure that a small percentage funds Bearings at no extra cost to the reader;
books are listed under their author in the Creators zone. Affiliate links only earn
once the steward's marketplace account (e.g. Amazon Associates) is connected — until
then they are ordinary product links.

---

## §6 Research and Data Collection

Real-time scraping during page renders has been retired (it caused IP blocks). The
approach now: official APIs first (Eventbrite, Meetup, iCal feeds), a nightly cron on
the VPS (systemd timer), local validation before insert, and community submission via
the chatbot intake and the `submissions` table. Dedup before every insert; never
insert a record without a source.

---

## §7 Technical Stack

**Phase 1 (retired):** a Lovable React prototype over Supabase PostgreSQL — the
original rapid-start UI, since superseded by Phase 2.

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
- **Interface i18n & view layer:** EN, ES, FR and JA are compiled in at startup
  (OnceLock, with English fallback); a small set of view helpers (`esc`/`card`/`split`/
  `badge`/`link_badge`) in `ui.rs` is the single source of HTML layout, composed by
  every zone rather than re-hand-rolled.
- **Agent access (MCP):** a read-only Model Context Protocol server at `POST /mcp`
  exposes the directory to AI agents as JSON-RPC tools, alongside `llms.txt` — so the
  data is consumable by agents, not just humans.

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
Eastern Europe), and **What Could Be** (community-upvoted ideas). A `bear_regions` table
now carries per-region organising, payment, platform and safety notes in ten
languages — researched in-language, each with a source.

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
