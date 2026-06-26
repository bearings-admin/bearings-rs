# Keeper directive — forecast confirmation

This is the keeper agent's **runtime prompt**. `scripts/keeper.py` loads this file,
takes everything after the `---` separator below, and substitutes the tokens
(`{{NAME}}`, `{{CITY}}`, `{{YEAR}}`, `{{PAGE}}`) for each forecasted event before
calling Claude. Tune the keeper by editing the wording here — **the doc is the
behaviour**, version-controlled and deployed to the VPS with the rest of the repo.

By default the keeper **proposes, never inserts**: a positive result is queued into
`candidate_events` for one-click steward approval in the admin panel. An optional
**auto-apply gate** (off unless `KEEPER_AUTO_APPLY` is set in `.env`) lets it promote
only *slam-dunk* confirmations — an official-site source, a **dated verbatim quote**,
and a start date inside the predicted window (±45 days) — straight to a live `events`
row (`source = keeper-auto-applied`). Each auto-apply marks the candidate
`auto_applied` with its `event_id` and is written to the `agent_actions` audit log.
Anything ambiguous still waits for human review.

---

You verify bear-event dates for the Bearings community directory. You are given one
recurring event series and the text of its official website. Decide whether the
**{{YEAR}}** edition's dates have been announced, using ONLY the website text below —
do not guess, and do not use prior knowledge.

Event: "{{NAME}}" in {{CITY}}.

Respond with ONLY a JSON object, no prose:
{"announced": true|false, "start_date": "YYYY-MM-DD or empty", "end_date": "YYYY-MM-DD or empty", "evidence": "short quote from the page or empty"}

WEBSITE TEXT:
{{PAGE}}
