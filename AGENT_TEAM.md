# Bearings — Agent Team

*Status: 2026-06-26. The keeper is live and scheduled, now with **two missions**
(forecast confirmation + web-search historical backfill), an **auto-apply gate**
(enabled) for slam-dunk confirmations with a one-click admin **Undo**, and the nightly
**email digest** reaching the steward via Resend. This is the canonical doc for the agent
roster, model assignments, and roadmap. Operational detail lives in `RESEARCH_DIRECTIVE.md`
and `directives/`.*

---

## The operating principles

1. **Propose, never insert — with one audited exception.** Agents turn messy
   input/research into a *structured, reviewable diff* the steward approves (CONST-9:
   never raw UGC). The keeper queues to `candidate_events` (pending) → steward clicks
   Approve. The **one deliberate exception** is the auto-apply gate: a *slam-dunk*
   forecast confirmation (official site + dated verbatim quote + in-window) is promoted
   automatically, but it is **audited** (`agent_actions`), **reversible** (admin Undo
   archives the event), **opt-in** (`KEEPER_AUTO_APPLY`), and never crosses the
   constitutional line. Research that isn't a slam-dunk (e.g. the backfill mission) is
   always reviewed.
2. **Tiered escalation — gate cheap, escalate rarely.** Don't run one model for the
   whole pipeline. Cheap models handle the high-volume, low-judgment work and escalate
   to a more capable model only for the hard / high-stakes / hard-to-reverse cases
   (~5–10% of calls). Most volume hits Haiku; the wallet feels the difference.
3. **Deterministic where possible.** If a step can be plain code (feed parsing,
   applying an already-approved diff), it is — no LLM at all.
4. **Reuse the existing review surfaces.** Agents feed the same admin queue
   (`candidate_events`) + Approve action the nightly feed reader already uses.

---

## The roster & model assignment

| Agent / role | Job | Model | Status |
|---|---|---|---|
| **Feed reader** | Nightly RSS/iCal parse, dedup, queue candidates | none (deterministic) | **live** (`bearings-feeds.timer`) |
| **Lifecycle sweep** | Archive past events (retain = foresight fuel), log foresight | none | **live** (in feed cron) |
| **Keeper — forecast confirmation** | Read `event_predictions`, verify next-edition dates from official sites, queue confirmations; **auto-apply slam-dunks** (audited, reversible) | **Haiku** (`KEEPER_MODEL`) | **live** (`bearings-keeper.timer`, weekly Mon 03:30 UTC) + **auto-apply enabled** |
| **Keeper — historical backfill** | Research PAST editions of single-edition series via **web search**, queue them (reviewed) | **Haiku** + web_search tool | **live** (`KEEPER_MISSION=backfill`, on demand) |
| **Keeper — lineage harvest** | Fill titleholder lineage gaps from club Hall-of-Fame pages | Haiku | directive written (`lineage_harvest.md`); needs a proposals queue + run-mode |
| **Triage / router** | Classify intake: correction vs add vs organizer vs travel vs spam/abuse | Haiku | planned |
| **Conversational intake** | Talk to humans; the 4 roles (history correction, titleholder adds, organizer "why isn't my event here", invisible-community surfacing) | Sonnet | planned |
| **Proposal drafting** | Turn input + research into a precise, schema-valid DB diff w/ sourcing + constitutional rules | Sonnet; **Opus** for high-stakes | planned |
| **Commit / apply** | Apply an *approved* data diff → deterministic/Haiku. Author *code* (migrations, PRs) → **Opus** | split | partial (code authoring is Claude Code in-session today) |
| **Forage** (sibling) | Bear AI travel agent; reads Bearings read-only | its own | separate project; intake just routes travel Qs to it |

**Which roles actually need Opus?** Only two: **authoring code/PRs** (auto-merges; a
bad diff matters) and **high-stakes proposal judgment** (privacy CONST-6, irreversible
merges). Everything else is Haiku or Sonnet. Sonnet is the default brain; Haiku does
the volume. On Sonnet/Opus, also dial `effort` down for routine steps before jumping a
whole model tier.

---

## What's live today (the significant move)

**The keeper exists and runs.** `scripts/keeper.py` (zero-dep: urllib + raw Anthropic
Messages API), weekly systemd timer, on Haiku. The full cycle:

```
forecast (event_predictions)  →  keeper fetches official site  →  Claude verifies
next-year dates  →  queues a confirmation into candidate_events (pending)
→  steward clicks Approve in admin  →  live event (enriched: start+end+city+link)
→  the forecast resolves itself (a confirmed edition removes the shadow)
```

- Validated end-to-end (queued → rendered with clean date → approved → enriched event
  → cleaned up). Positive control: extracted BeefDip 2027 (Jan 24–31) from beefdip.com
  with a source quote.
- The forecast (rolling-12mo bar, recurrence engine, shadow bars, tentative cards) is
  the keeper's **prioritized worklist** — what to look for, and where.
- Shipped via the proper branch → CI → auto-merge → deploy pipeline (PRs #14 verify-link,
  #15 keeper v1, #16 keeper→admin queue).

---

## Credentials & config

- `ANTHROPIC_API_KEY` — in `/opt/bearings-rs/.env` only (rotated, validated). Never in
  git, never in chat, never in agent memory.
- `KEEPER_MODEL` — in `.env`; `claude-haiku-4-5`. Switch per-agent later.
- `KEEPER_AUTO_APPLY` — `1` enables the auto-apply gate (currently on). Set `0` to pause.
- `KEEPER_MISSION` — `forecast` (default, the weekly timer) or `backfill` (web-search past
  editions, on demand). `KEEPER_BACKFILL_LIMIT` caps per-run work.
- `RESEND_API_KEY` (+ `DIGEST_FROM`/`DIGEST_TO`) — the nightly email digest transport.
- Billed pay-per-token on the Anthropic API; the backfill mission also incurs web-search fees.

---

## Roadmap

1. ✅ **Auto-apply gate** (shipped + enabled, 2026-06-26) — slam-dunk confirmations create the
   event and just notify, with an `agent_actions` audit log + admin "Auto-applied (Undo)" view.
2. ✅ **Historical backfill mission** (shipped, 2026-06-26) — `KEEPER_MISSION=backfill` uses
   Claude's web_search tool to find past editions (0 → 11 sourced proposals on the first run).
3. **Lineage harvest run-mode** (next, directive written) — wire `lineage_harvest.md` to a
   keeper run-mode; needs a `candidate_title_holders` proposals queue + admin surface.
4. **Surfaces beyond the admin page** — a **Telegram bot**, or another agent reading the
   keeper's output to escalate (steward's stated direction).
5. **Conversational intake** — grow into the full intake chat (the 4 roles), with triage
   (Haiku) → drafting (Sonnet) → steward review.

---

## Design notes / guardrails

- Agents run as systemd oneshots on the VPS (sibling to `bearings-feeds`), reading keys
  from `.env`. They are **not** persistent daemons that can summon Claude Code; "passing
  actions to an agent" today means the admin queue + the steward, or Claude Code in a
  live session. A persistent apply-worker would be a separate build.
- Everything an agent proposes is sourced and reviewable; the constitutional line
  (no unreviewed UGC, privacy for criminalised regions) is never crossed by automation.
