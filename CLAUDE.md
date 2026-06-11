# CLAUDE.md — Bearings Agent Directive

**Project:** Bearings — global gay bear community infrastructure  
**Steward:** ursasteward@pm.me  
**Repo:** github.com/bearings-admin/bearings-rs  
**Updated:** 2026-06-11

This file is loaded automatically by Claude Code at session start. It replaces the `documents` table in Supabase as the canonical agent directive.

---

## What Bearings Is

A verified living directory of bear events, places, clubs, title holders, creators, digital spaces, campaigns, and community history. Unincorporated association. Not a dating app, social network, or event promoter.

**Four-zone information architecture:**
- **ARCHIVE** — community memory since 1987. Decade tabs. Title holder lineage. Oral histories.
- **NOW** — hot events today, current title holders, active campaigns.
- **COMING UP** — default landing. "When & Where do you want to meet?" Trip planner + iCal.
- **FUTURE** — active campaigns, milestones, new bear regions, governance direction.

---

## Stack

| Layer | Detail |
|-------|--------|
| Database | Supabase PostgreSQL — project `mntdhflffhrjjvipxgyl` (ca-central-1) |
| Backend | Rust/Axum — bearings-rs workspace on `srv1744879.hstgr.cloud` (Ubuntu 24.04, Hostinger) |
| Live URL | https://srv1744879.hstgr.cloud — Caddy reverse proxy + Let's Encrypt → localhost:3000 |
| VPS path | `/opt/bearings-rs/` — workspace root |
| SSH | `ssh root@2.25.191.141` (key: `~/.ssh/id_ed25519`) |
| Deploy | `systemctl restart bearings-backend` after `cargo build --release` |
| GitHub | `bearings-admin/bearings-rs` — `gh` CLI authenticated as `bearings-admin` |

---

## Rust Workspace

```
bearings-rs/
  bearings-backend/     — Axum SSR + REST API (the live server)
  bearings-shared/      — shared typed models (used by backend + agent)
  bearings-agent/       — treasury monitor, Blockfrost, Bluesky stub
  bearings-frontend/    — Leptos 0.6 skeleton (Phase 3, not yet active)
```

### bearings-backend source layout

```
src/
  lib.rs           — owns all modules + build_app(); the library crate
  main.rs          — thin binary: env -> config -> serve
  db.rs            — SupabaseClient: get_json<T> (TTL-cached), post_rpc, write_json
  cache.rs         — 30s TTL cache fronting get_json (url -> raw JSON body)
  config.rs        — all env vars validated at startup
  error.rs         — AppError -> HTTP response
  middleware.rs    — privacy enforcement (CONST-6: criminalised-country list)
  i18n.rs          — 842 EN/ES/FR keys baked at startup via OnceLock
  ui.rs            — design system + esc() HTML-escape; stylesheet() served at /style.css
  llms.rs          — /llms.txt, /llms-full.txt, /robots.txt
  ssr/
    mod.rs         — ZoneQuery, Zone enum + Zone::parse(), root() dispatcher
    query.rs       — typed DB row structs (EventRow, PlaceRow, CurrentHolder, ...)
    zones/         — 13 zone renderers (now, coming_up, archive, future, places,
                     events, clubs, titles, creators, campaigns, ical, digital, admin)
  repositories/    — data access: trait + Supabase impl per resource (DIP);
                     clause() percent-encodes filter values. 14 repos.
  services/        — business logic; vote_service orchestrates voting
  routes/          — thin JSON REST handlers; delegate to repositories
tests/
  api_tests.rs     — HTTP integration tests (axum-test) against build_app()
```

**Key patterns:**
- Layering: routes -> services -> repositories -> db (PostgREST). Handlers stay thin.
- Typed reads via `db.get_json::<Vec<XxxRow>>(&url)` (no `serde_json::Value`); 30s TTL cache fronts it.
- Zone dispatch: `match Zone::parse(...) { Zone::Now => ... }`
- Security: `ui::esc()` on every rendered value (XSS); `repositories::clause()` encodes filter values (injection).
- Tests: `cargo test -p bearings-backend --lib` (unit, no network) and `--test api_tests` (HTTP, needs `SUPABASE_URL`).
- Architecture & decisions: `bearings-backend/ARCHITECTURE.md`. DB schema/portability: `supabase/`.

**Design constants (all in ui.rs):**
`BROWN #5C4033`, `ORANGE #D2691E`, `GOLD #D4A017`, `TAN #C8B89A`, `OFF_WHITE #F9F5F0`, `DARK #1A1A1A`, `MID #777777`

---

## Database Quick Reference

**Session health check:**
```sql
SELECT
  (SELECT COUNT(*) FROM events WHERE active = true) as events,
  (SELECT COUNT(*) FROM places WHERE active = true) as places,
  (SELECT COUNT(*) FROM clubs WHERE active = true) as clubs,
  (SELECT COUNT(*) FROM title_holders) as title_holders;
```

**Current data (June 2026):** ~88 events, 173 places, 49 clubs, 32 competitions, ~89 title holders, 49 bear history entries, 35 creators, 12 campaigns, 32 digital spaces, 842 translation rows.

**Known schema notes:**
| Table | Note |
|-------|------|
| `creators` | No unique constraint on `name` — use `INSERT ... WHERE NOT EXISTS` |
| `creators` | Column is `bio` not `description` |
| `digital_spaces` | Column is `url` not `link` |
| `places` | `place_type` uses hyphens: `sauna-bathhouse`, `leather-bar`, `party-venue` |
| `title_holders` | `holder_status` enum: `active` / `holdover` / `vacant` / `unknown` |

**Never delete rows.** Set `active = false` to archive. The Archive Principle is constitutional (CONST-8).

**Write permissions:** Only `submissions` and `newsletter_subscribers` accept public writes. All other tables require steward approval.

---

## Constitutional Values (CONST-1–10)

These require a 75% supermajority to amend. Operational directives require 60%.

- **CONST-1** Community memory preservation is the primary purpose.
- **CONST-2** No single point of human failure — all state recoverable by any authorised steward.
- **CONST-3** Governance must remain lightweight.
- **CONST-4** Seven languages are operational. EN/ES/FR baked in; DE/PT/AR/JA translated live.
- **CONST-5** Revenue serves community — no individual extraction, all affiliates disclosed.
- **CONST-6** Privacy protection is non-negotiable. Submissions from criminalised countries activate `privacy_mode`. Cannot be overridden.
- **CONST-7** Do not compete with partners — Bearings is infrastructure, not a destination.
- **CONST-8** Content freshness is a constitutional obligation. Stale content must be flagged or archived.
- **CONST-9** Conversational intake is the primary submission mechanism — the chatbot is never replaced by a cold form.
- **CONST-10** Inclusion is shown, not decided — never remove a listing for being exclusionary; flag it with `inclusion_flag_codes` and provide the inclusive alternative.

---

## Key Collaborator

**Gaspar** (gasparteixeira on GitHub) — senior Rust/CS developer, uses Rocket + Yew stack.  
Reviewing and extending the codebase. The architecture was shaped to be comfortable to him:
- Zone enum (not string dispatch)
- Typed DB row structs (not `serde_json::Value`)
- Tests with clear comments explaining what each test guards
- `build_app()` extracted from `main()` for testability
- `db.rs` documents PostgREST vs SQLx tradeoff honestly

Gaspar's primary work will be the Leptos frontend (`bearings-frontend`). The Axum SSR (`bearings-backend`) is the bridge until Leptos takes over rendering.

---

## Pending Items

| Item | Notes |
|------|-------|
| Mr Bear Europe 2026 winner | Competition: Lisbon, July 15–19 2026 — update `title_holders` after crowning |
| Mr Bear UK lineage | Zero records — full lineage unknown. Contact: mrbearuk.info |
| Mr TBRU 1995–2026 | Outreach: contest@tbru.org |
| NAB Weekend gaps | 2012–2014 names, 2017 name — contact: nabweekend@gmail.com |
| Governance model | For-profit vs community infrastructure — Gaspar + steward decision pending |
| Cardano treasury / NORTH token | Deferred until governance model decided |
| `bearings-frontend` Phase 3 | Wire Supabase client into Leptos server functions |
| Nightly research cron | Replace manual inserts with systemd timer + Eventbrite/iCal APIs |

---

## Research Principles

- **Official APIs first:** Eventbrite API, Meetup API, iCal feed parsing (nightly cron on VPS via systemd timer)
- **Never scrape in real time** during page renders — caused IP blocks
- **Dedup before every insert** — check by name + date/location
- **Gap records over guesses** — `holder_status = 'unknown'`, `holder_name = 'Unknown — name not in public record'`
- **Source required** — note source URL in bio/description/notes on every insert
- **Privacy mode** — any data linked to a criminalised country: `privacy_mode = true`, location no more specific than country

---

## Workflow Notes

- **Edit files:** SSH to VPS → Python patch script → `cargo check` → `cargo build --release` → `systemctl restart bearings-backend`
- **Test before deploying:** `~/.cargo/bin/cargo test -p bearings-backend --bin bearings-backend`
- **Commit pattern:** `git add -p` specific files, never `git add -A` (secrets risk)
- **Bluesky publishing:** All posts require steward review. 4-hour cooldown. Agent never posts without review.
