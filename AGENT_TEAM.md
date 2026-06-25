# Bearings — Agent Team

*Status: 2026-06-25. The team went from concept to running infrastructure today —
the first AI agent (the keeper) is live, scheduled, and feeding one-click work into
the admin panel. This is the canonical doc for the agent roster, model assignments,
and roadmap. Operational detail lives in `RESEARCH_DIRECTIVE.md` and `directives/`.*

---

## The operating principles

1. **Propose, never insert.** No agent writes to the public directory unreviewed.
   Agents turn messy input/research into a *structured, reviewable diff* that the
   steward (or, later, an automated check) approves. This is constitutional
   (CONST-9: conversational intake; never raw UGC). The forecast keeper embodies it:
   it queues confirmations into `candidate_events` (pending) → steward clicks Approve.
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
| **Keeper — forecast confirmation** | Read `event_predictions`, fetch each series' official site, verify next-edition dates, queue confirmations | **Haiku** (`KEEPER_MODEL`, default opus, set to haiku) | **live** (`bearings-keeper.timer`, weekly Mon 03:30 UTC) |
| **Keeper — historical backfill** | Find thin (1-year) series, research past editions, queue them | Haiku → Sonnet | planned (same engine, new mission) |
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
- `KEEPER_MODEL` — in `.env`; `claude-haiku-4-5` (set today). Switch per-agent later.
- Billed pay-per-token on the Anthropic API (separate from any Claude Pro sub).

---

## Roadmap

1. **Auto-apply gate** (next) — for slam-dunk confirmations (date on the *official* site,
   verbatim quote, inside the predicted window), let the keeper create the event and just
   *notify* the steward, reserving the admin queue for ambiguous cases. A deliberate step
   beyond "always review" — needs a confidence threshold + an audit log. *Steward chose to
   keep everything on the admin page until trust is established.*
2. **Historical backfill mission** — point the keeper at thin series to deepen the Archive
   AND the forecast (both research, both reviewed).
3. **Surfaces beyond the admin page** — a **Telegram bot**, or have another agent read the
   keeper's output to escalate further (steward's stated direction).
4. **Conversational intake** — grow from the narrow research mission into the full
   intake chat (the 4 roles), with triage (Haiku) → drafting (Sonnet) → steward review.

---

## Design notes / guardrails

- Agents run as systemd oneshots on the VPS (sibling to `bearings-feeds`), reading keys
  from `.env`. They are **not** persistent daemons that can summon Claude Code; "passing
  actions to an agent" today means the admin queue + the steward, or Claude Code in a
  live session. A persistent apply-worker would be a separate build.
- Everything an agent proposes is sourced and reviewable; the constitutional line
  (no unreviewed UGC, privacy for criminalised regions) is never crossed by automation.
