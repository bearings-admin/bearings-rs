# Keeper Mission — Titleholder Lineage Harvest

*Directive for the keeper agent. Lives in the repo so it deploys to the VPS and the
cron can read it at runtime (see `CLAUDE.md` → Agents & Automation). Status as of
2026-06-25: **directive written; wiring (proposals queue + keeper run-mode) still to
build** — see "Wiring status" at the bottom.*

---

## Mission

Deepen the Archive (CONST-1) by filling **gaps in titleholder lineages** from official
club / competition sources. The keeper **proposes, never inserts** — every find lands in
a review queue for one-click steward approval, exactly like the forecast-confirmation
mission.

## Why this exists (the Belgium lesson)

A lineage can look *complete* in our data and still be missing a decade. We held only
Mr. Bear Belgium 2024–2026 (3 contiguous rows — looks done), but the club's "Hall of
Fame" tab lists 2012–2026. One tab over = 11 missing names. The gap was invisible to a
simple interior-gap scan because we never scraped anything *before* the current holder.

## Two kinds of gap — look for both

1. **Interior gaps** — rows scattered across a wide span with holes
   (e.g. Mr. Bear Poland: 6 rows over a 19-year span). A SQL scan finds these.
2. **Backward truncation** — we hold only the last 1–3 years, but the source goes back
   further. **Invisible to the gap scan** (the rows we have are contiguous, so
   `missing_yrs = 0`). Belgium, Netherlands, Germany, Milano, and most single-row 2025
   titles are suspect. **Always check the source's earliest year, not just our holes.**

## Candidate-selection query (starting worklist)

```sql
-- titles worth a source check: wide span with holes, OR recent-only (likely truncated)
SELECT title_name,
       count(*) AS rows, min(year) AS first_yr, max(year) AS last_yr,
       (max(year)-min(year)+1) - count(DISTINCT year) AS interior_gaps,
       max(club_id) AS club_id, max(competition_id) AS competition_id
FROM title_holders
WHERE year IS NOT NULL
GROUP BY title_name
HAVING ((max(year)-min(year)+1) - count(DISTINCT year)) > 0   -- interior gaps
    OR (max(year) >= 2024 AND count(*) <= 3)                  -- recent-only => suspect truncation
ORDER BY interior_gaps DESC, rows ASC;
```

## Harvest method

1. **Find the official source.** Prefer, in order: the club's own site (`clubs.website`),
   the competition's site, then a reputable aggregator (Bear World Magazine titleholders
   archive, Wikipedia). Note the source URL — it is required on every proposal.
2. **Look for a lineage page.** Tab names vary by language:
   `Hall of Fame` · `Laureaci` (PL) · `Anciens / Palmarès` (FR) · `Albo d'oro` (IT) ·
   `Vorherige Gewinner` (DE) · `Past winners / Titleholders`. Euro club titles usually
   have one; North-American titles are often only in press coverage (multi-source).
3. **Extract** `year` + `holder_name` (+ city/runner-up if shown).
4. **Cross-check** against what we already hold for that `title_name`; only the **missing**
   years become proposals. Dedup by `(title_name, year)`.

## Quality & constitutional rules (do not skip)

- **Source required** — every proposal carries the source URL in `bio`/notes.
- **First-name-only** is common on older Hall-of-Fame entries — queue it anyway, but set a
  `needs_surname` flag so the steward knows a second source is needed. Don't invent surnames.
- **Gap records over guesses** — if a year is genuinely blank at the source, propose
  `holder_status='unknown'`, `holder_name='Unknown — name not in public record'`. Never guess.
- **Privacy (CONST-6)** — a holder tied to a criminalised country: `privacy_mode`, no detail
  finer than country. When in doubt, hold it for the steward.
- **Never write `title_holders` directly.** Proposals only.
- **Don't delete or overwrite** an existing holder row — propose additions/corrections as a
  reviewable diff (extends CONST-9: bot-fed, never raw UGC).

## Proven yield (hand-check, 2026-06-25)

| Title | We held | Source | Find | Mode |
|-------|---------|--------|------|------|
| Mr. Bear Belgium | 2024–26 (3) | belgiumbearpride.be `/mister-bears/` | 2012–2026 Hall of Fame, ~11 missing | **easy** |
| Mr. Bear Poland | 6 rows/19yr | mrbearpoland.eu Hall of Fame | 2010–2026 + a full Vice lineage we don't track | **easy** |
| Mr. Bear Montréal | 7 rows/18yr | bearitmtl.com + press | scattered across press, no single page | medium |
| Mr Bear France | 3 rows/7yr | Fierté Ours Paris + press | scattered across press | medium |

## Wiring status (what's left to build before this can run)

This directive is the *behaviour spec*. To activate it as a keeper run-mode:
- **Proposals queue:** `candidate_events` is event-shaped. Titleholders need their own
  queue — a `candidate_title_holders` table (or a generic `proposals` table) mirroring the
  one-click-approve admin flow.
- **Admin surface:** a "Pending Titleholder Proposals" section, Approve → insert into
  `title_holders` with the source preserved.
- **keeper.py mode:** add a `--mission lineage` path that loads this directive, runs the
  candidate query, fetches sources, and writes proposals. Keep it on the cheap model
  (Haiku) — this is routine extraction, same as forecast confirmation.
- Possible schema extension: a **Vice/runner-up** lineage field, since Poland (and others)
  publish it. Out of scope for v1; note for later.
