#!/usr/bin/env python3
"""Bearings keeper — v1: forecast confirmation (+ optional auto-apply gate).

Reads predicted recurrences (the `event_predictions` view), fetches each series'
official website, and asks Claude whether the *next* edition's dates have actually
been announced. A positive result is queued as a REVIEWABLE proposal into the admin
review queue (`candidate_events`, status=pending) — the steward approves in the admin
panel, one click.

AUTO-APPLY GATE (off by default): when KEEPER_AUTO_APPLY is set, a *slam-dunk*
confirmation — official-site source, a dated verbatim quote, and a start date inside
the predicted window — is promoted straight to a live `events` row (source
`keeper-auto-applied`), the candidate is marked `auto_applied` with its `event_id`,
and the action is written to the `agent_actions` audit log. Everything ambiguous still
waits for human review. This is the deliberate "remove the steward from the routine
loop" step, so it is opt-in.

Zero third-party deps (urllib, matching feed_reader.py): talks to Supabase
PostgREST and the Anthropic Messages API over raw HTTP. Keys come from
/opt/bearings-rs/.env (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY, ANTHROPIC_API_KEY).
Model via KEEPER_MODEL (default claude-opus-4-8; claude-haiku-4-5 for cheap runs).
"""
import os
import re
import json
from datetime import date, datetime, timezone
from urllib.request import urlopen, Request
from urllib.error import HTTPError


def _load_env(p="/opt/bearings-rs/.env"):
    if os.path.exists(p):
        for line in open(p):
            line = line.strip()
            if line and "=" in line and not line.startswith("#"):
                k, v = line.split("=", 1)
                os.environ.setdefault(k.strip(), v.strip())


_load_env()
SUPABASE_URL = os.environ["SUPABASE_URL"]
SUPABASE_KEY = os.environ["SUPABASE_SERVICE_ROLE_KEY"]
ANTHROPIC_KEY = os.environ["ANTHROPIC_API_KEY"]
MODEL = os.environ.get("KEEPER_MODEL", "claude-opus-4-8")
# Auto-apply is OFF unless explicitly enabled — a deliberate trust step.
AUTO_APPLY = os.environ.get("KEEPER_AUTO_APPLY", "").strip().lower() in (
    "1",
    "true",
    "yes",
    "on",
)
# A confirmed start date must fall within this many days of the forecast to auto-apply.
WINDOW_DAYS = 45
UA = "Bearings-Keeper/1.0 (+https://bearings.community)"

# The keeper's prompt is a repo file, not hardcoded — edit directives/keeper.md to tune
# the agent (the doc is the behaviour). Everything after the first '---' is the template
# sent to Claude, with per-event tokens substituted in check().
_DIRECTIVE_PATH = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "directives", "keeper.md"
)
PROMPT_TEMPLATE = (
    open(_DIRECTIVE_PATH, encoding="utf-8").read().split("\n---\n", 1)[-1].strip()
)


def supa_get(path):
    req = Request(
        f"{SUPABASE_URL}/rest/v1/{path}",
        headers={
            "apikey": SUPABASE_KEY,
            "Authorization": f"Bearer {SUPABASE_KEY}",
            "Accept": "application/json",
        },
    )
    with urlopen(req, timeout=20) as r:
        return json.loads(r.read())


def supa_post(path, data, prefer="resolution=ignore-duplicates,return=minimal"):
    body = json.dumps(data).encode()
    req = Request(
        f"{SUPABASE_URL}/rest/v1/{path}",
        data=body,
        method="POST",
        headers={
            "apikey": SUPABASE_KEY,
            "Authorization": f"Bearer {SUPABASE_KEY}",
            "Content-Type": "application/json",
            "Prefer": prefer,
        },
    )
    try:
        with urlopen(req, timeout=20) as r:
            # return=representation yields the created row(s); minimal yields no body.
            if "return=representation" in prefer:
                return json.loads(r.read())
            return r.status
    except HTTPError as e:
        return e.code


def supa_patch(path, data):
    body = json.dumps(data).encode()
    req = Request(
        f"{SUPABASE_URL}/rest/v1/{path}",
        data=body,
        method="PATCH",
        headers={
            "apikey": SUPABASE_KEY,
            "Authorization": f"Bearer {SUPABASE_KEY}",
            "Content-Type": "application/json",
            "Prefer": "return=minimal",
        },
    )
    try:
        with urlopen(req, timeout=20) as r:
            return r.status
    except HTTPError as e:
        return e.code


def fetch_text(url):
    req = Request(url, headers={"User-Agent": UA})
    with urlopen(req, timeout=25) as r:
        html = r.read().decode("utf-8", "replace")
    html = re.sub(r"(?is)<script.*?</script>|<style.*?</style>", " ", html)
    text = re.sub(r"<[^>]+>", " ", html)
    return re.sub(r"\s+", " ", text).strip()[:9000]


def claude(prompt):
    body = json.dumps(
        {
            "model": MODEL,
            "max_tokens": 700,
            "messages": [{"role": "user", "content": prompt}],
        }
    ).encode()
    req = Request(
        "https://api.anthropic.com/v1/messages",
        data=body,
        method="POST",
        headers={
            "x-api-key": ANTHROPIC_KEY,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        },
    )
    with urlopen(req, timeout=90) as r:
        data = json.loads(r.read())
    return "".join(
        b.get("text", "") for b in data.get("content", []) if b.get("type") == "text"
    )


def parse_json(s):
    try:
        return json.loads(s)
    except Exception:
        m = re.search(r"\{.*\}", s, re.S)
        return json.loads(m.group(0)) if m else {}


def next_edition_name(name, year):
    if re.search(r"\b(19|20)\d{2}\b", name):
        return re.sub(r"\b(19|20)\d{2}\b", year, name)
    return f"{name} {year}"


def check(pred):
    year = (pred.get("predicted_date") or "")[:4]
    name = pred.get("sample_name", "")
    city = pred.get("city", "")
    try:
        page = fetch_text(pred["website"])
    except Exception as e:
        return {"error": str(e)}
    prompt = (
        PROMPT_TEMPLATE.replace("{{NAME}}", name)
        .replace("{{CITY}}", city)
        .replace("{{YEAR}}", year)
        .replace("{{PAGE}}", page)
    )
    return parse_json(claude(prompt))


def build_candidate(pred, year, found):
    start = found["start_date"]
    end = found.get("end_date") or ""
    base = re.sub(r"\s*\b(19|20)\d{2}\b\s*$", "", pred["sample_name"]).strip()
    cand = {
        "raw_title": next_edition_name(pred["sample_name"], year),
        "raw_description": (
            f"Keeper-confirmed from the official site ({MODEL}). "
            f"{base} {year}: {start}" + (f" to {end}" if end else "")
            + f". Evidence: {found.get('evidence', '')[:200]}"
        ),
        "raw_date": start.replace("-", ""),
        "parsed_start": start,
        "parsed_end": (end or None),
        "parsed_city": pred.get("city") or None,
        "parsed_country": pred.get("country") or None,
        "parsed_type": "bear-run",
        "source_url": f"{pred['website']}#keeper-{year}",
        "status": "pending",
    }
    return {k: v for k, v in cand.items() if v is not None}


def _parse_date(s):
    try:
        return date.fromisoformat((s or "")[:10])
    except Exception:
        return None


def is_slam_dunk(pred, year, found):
    """Strict gate for auto-apply: official source + dated quote + in-window."""
    start = _parse_date(found.get("start_date"))
    if not (found.get("announced") and start):
        return False, "not announced / no start date"
    if str(start.year) != str(year):
        return False, f"start year {start.year} != forecast {year}"
    pdate = _parse_date(pred.get("predicted_date"))
    if pdate and abs((start - pdate).days) > WINDOW_DAYS:
        return False, f"out of window ({abs((start - pdate).days)}d > {WINDOW_DAYS})"
    ev = (found.get("evidence") or "").strip()
    if len(ev) < 25 or not any(ch.isdigit() for ch in ev):
        return False, "weak/undated evidence"
    return True, "official site + dated quote + in-window"


def audit(action, cid, eid, pred, found, detail):
    row = {
        "agent": "keeper",
        "action": action,
        "candidate_id": cid,
        "event_id": eid,
        "series_name": pred.get("sample_name"),
        "detail": (detail + " | " + (found.get("evidence") or ""))[:500],
        "model": MODEL,
    }
    supa_post("agent_actions", {k: v for k, v in row.items() if v is not None})


def promote_to_event(cand):
    """Create the live event from a candidate, stamped as keeper-auto-applied."""
    event = {
        "name": cand.get("raw_title"),
        "description": cand.get("raw_description"),
        "country": cand.get("parsed_country"),
        "city": cand.get("parsed_city"),
        "start_date": cand.get("parsed_start"),
        "end_date": cand.get("parsed_end"),
        "link": cand["source_url"].split("#")[0],
        "type": cand.get("parsed_type") or "bear-run",
        "active": True,
        "source": "keeper-auto-applied",
    }
    event = {k: v for k, v in event.items() if v is not None}
    rows = supa_post(
        "events", event, prefer="return=representation"
    )
    return rows[0]["id"] if isinstance(rows, list) and rows else None


def main():
    preds = supa_get(
        "event_predictions?select=sample_name,city,country,predicted_date,confidence,website"
        "&order=predicted_date"
    )
    mode = "AUTO-APPLY ON" if AUTO_APPLY else "review-only"
    print(
        f"[keeper] checking {len(preds)} forecasted series for announced dates "
        f"(model={MODEL}, {mode})\n"
    )
    queued = 0
    applied = 0
    for p in preds:
        site = p.get("website")
        if not site:
            print(f"  - {p['sample_name']}: no website on file — skip")
            continue
        r = check(p)
        year = (p.get("predicted_date") or "")[:4]
        if r.get("error"):
            print(f"  ! {p['sample_name']}: fetch/check failed — {r['error']}")
            continue
        if not (r.get("announced") and r.get("start_date")):
            print(f"  ·  {p['sample_name']}: {year} not announced yet on {site}")
            continue

        cand = build_candidate(p, year, r)
        created = supa_post(
            "candidate_events", cand, prefer="return=representation"
        )
        cid = created[0]["id"] if isinstance(created, list) and created else None
        ok, reason = is_slam_dunk(p, year, r)
        print(
            f"  CONFIRM  {p['sample_name']} {year}: {r['start_date']} -> "
            f"{r.get('end_date', '')} (forecast ~{p['predicted_date']})\n"
            f"           source:   {site}\n"
            f"           evidence: {r.get('evidence', '')[:160]}\n"
            f"           gate:     {'PASS' if ok else 'hold'} — {reason}"
        )

        if AUTO_APPLY and ok and cid:
            eid = promote_to_event(cand)
            supa_patch(
                f"candidate_events?id=eq.{cid}",
                {
                    "status": "auto_applied",
                    "event_id": eid,
                    "reviewed_at": datetime.now(timezone.utc).isoformat(),
                },
            )
            audit("auto_apply", cid, eid, p, r, reason)
            applied += 1
            print(f"           -> AUTO-APPLIED to live events (event #{eid})")
        else:
            note = reason if not ok else "auto-apply disabled (set KEEPER_AUTO_APPLY)"
            audit("propose", cid, None, p, r, note)
            queued += 1
            print(f"           -> queued for steward review ({note})")

    print(
        f"\n[keeper] {applied} auto-applied, {queued} queued for review "
        f"(mode={mode})."
    )


if __name__ == "__main__":
    main()
