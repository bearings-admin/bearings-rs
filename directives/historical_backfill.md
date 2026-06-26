# Keeper directive — historical backfill

The keeper's **runtime prompt** for the historical-backfill mission
(`KEEPER_MISSION=backfill`). `scripts/keeper.py` loads everything after the `---`
separator and substitutes `{{NAME}}`, `{{CITY}}`, `{{SITE}}` for each single-edition
series (from the `event_backfill_targets` view) before calling Claude **with the
web_search tool enabled**.

Goal: deepen the Archive (CONST-1) **and** the recurrence forecast. The forecast engine
only fires for series seen in ≥2 years, so recovering an annual event's *past* editions
both enriches community memory and lets the engine start predicting future repeats.

Why web search: an event's homepage almost always shows only the current year. Past
editions live in press coverage, listings, archives, and Wikipedia — so the keeper
searches the web for them rather than reading a single page.

The keeper **proposes, never inserts**: every past edition found becomes a reviewable
`candidate_events` row (status=pending) for one-click steward approval. There is **no
auto-apply for this mission** — historical research is more error-prone than confirming an
official announcement, so a human always reviews. Sourcing is required; never guess dates.

---

You research the history of a recurring LGBTQ+ "bear" community event for an archive.
Use the web_search tool to find **past editions** (previous years) of the event and their
dates, from reliable sources — the event's own site or archive, community press, and
event listings. Do not guess: only include an edition you can date from a source you found.

Event: "{{NAME}}" in {{CITY}}. Official site (if any): {{SITE}}.

Search for the event's previous years / past dates / history / hall of fame. Then respond
with ONLY a JSON array (no prose, no markdown fences). One object per past edition you can
date:
[{"year": "YYYY", "start_date": "YYYY-MM-DD or empty", "end_date": "YYYY-MM-DD or empty", "evidence": "short note including the source"}]

Rules:
- Only include an edition for which you found at least a start date in a real source.
- Prefer the event's official/archive pages and reputable community sources.
- Include the source in the "evidence" field.
- If you cannot find any datable past editions, respond with exactly: []
