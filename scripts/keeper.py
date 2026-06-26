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

# Mission selector: "forecast" (default, weekly date-confirmation) or "backfill"
# (research PAST editions of single-edition series to deepen Archive + forecast).
MISSION = os.environ.get("KEEPER_MISSION", "forecast").strip().lower()
# Cap per-run work for the backfill mission (API cost + politeness to sites).
BACKFILL_LIMIT = int(os.environ.get("KEEPER_BACKFILL_LIMIT", "8"))


def load_directive(fname):
    p = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "directives", fname
    )
    return open(p, encoding="utf-8").read().split("\n---\n", 1)[-1].strip()


# Loaded only when actually running the backfill mission.
BACKFILL_PROMPT = (
    load_directive("historical_backfill.md") if MISSION == "backfill" else ""
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


def claude(prompt, tools=None, max_tokens=700):
    payload = {
        "model": MODEL,
        "max_tokens": max_tokens,
        "messages": [{"role": "user", "content": prompt}],
    }
    if tools:
        payload["tools"] = tools
    body = json.dumps(payload).encode()
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
    # Web search can take a while (multiple round trips), so allow more time.
    with urlopen(req, timeout=180 if tools else 90) as r:
        data = json.loads(r.read())
    return "".join(
        b.get("text", "") for b in data.get("content", []) if b.get("type") == "text"
    )


# Server-side web search tool — lets the backfill mission find PAST editions from
# real sources (press, archives, Wikipedia) instead of only the event's homepage.
WEB_SEARCH_TOOL = [{"type": "web_search_20250305", "name": "web_search", "max_uses": 4}]


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


def run_forecast():
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


def parse_json_list(s):
    try:
        v = json.loads(s)
        return v if isinstance(v, list) else []
    except Exception:
        m = re.search(r"\[.*\]", s, re.S)
        if m:
            try:
                return json.loads(m.group(0))
            except Exception:
                return []
        return []


def find_past_editions(target):
    name = target.get("sample_name", "")
    city = target.get("city") or ""
    site = target.get("website") or ""
    prompt = (
        BACKFILL_PROMPT.replace("{{NAME}}", name)
        .replace("{{CITY}}", city)
        .replace("{{SITE}}", site)
    )
    try:
        out = claude(prompt, tools=WEB_SEARCH_TOOL, max_tokens=2000)
    except Exception as e:
        return {"error": str(e)}
    return {"editions": parse_json_list(out)}


def queue_backfill_proposal(target, ed):
    year = str(ed.get("year") or "")[:4]
    start = ed.get("start_date") or ""
    end = ed.get("end_date") or ""
    base = re.sub(r"\s*\b(19|20)\d{2}\b\s*$", "", target["sample_name"]).strip()
    cand = {
        "raw_title": f"{base} {year}".strip(),
        "raw_description": (
            f"Keeper historical backfill ({MODEL}): past edition of {base}. "
            f"{year}: {start}" + (f" to {end}" if end else "")
            + f". Evidence: {(ed.get('evidence') or '')[:200]}"
        ),
        "raw_date": (start.replace("-", "") if start else None),
        "parsed_start": (start or None),
        "parsed_end": (end or None),
        "parsed_city": target.get("city") or None,
        "parsed_country": target.get("country") or None,
        "parsed_type": "bear-run",
        "source_url": f"{target['website']}#backfill-{year}",
        "status": "pending",
    }
    cand = {k: v for k, v in cand.items() if v is not None}
    created = supa_post("candidate_events", cand, prefer="return=representation")
    return created[0]["id"] if isinstance(created, list) and created else None


def run_backfill():
    targets = supa_get(
        "event_backfill_targets?select=sample_name,city,country,website,known_date"
        f"&order=sample_name&limit={BACKFILL_LIMIT}"
    )
    this_year = date.today().year
    print(
        f"[keeper] historical backfill: scanning {len(targets)} single-edition "
        f"series for past editions (model={MODEL})\n"
    )
    queued = 0
    for t in targets:
        site = t.get("website")
        if not site or site == "#":
            continue
        known_year = str(t.get("known_date") or "")[:4]
        r = find_past_editions(t)
        if r.get("error"):
            print(f"  ! {t['sample_name']}: fetch failed — {r['error']}")
            continue
        # Keep only genuine PAST editions (with dates) we don't already hold.
        past = [
            e
            for e in r.get("editions", [])
            if e.get("start_date")
            and str(e.get("year") or "")[:4].isdigit()
            and int(str(e["year"])[:4]) < this_year
            and str(e.get("year") or "")[:4] != known_year
        ]
        if not past:
            print(f"  ·  {t['sample_name']}: no new past editions on {site}")
            continue
        for e in past:
            cid = queue_backfill_proposal(t, e)
            audit(
                "backfill",
                cid,
                None,
                {"sample_name": t.get("sample_name")},
                {"evidence": e.get("evidence", "")},
                f"past edition {e.get('year')}",
            )
            queued += 1
            print(
                f"  +  {t['sample_name']} {e.get('year')}: {e.get('start_date')} "
                f"-> queued for review (cand {cid})"
            )
    print(
        f"\n[keeper] historical backfill: {queued} past-edition proposal(s) "
        f"queued for steward review (no auto-insert)."
    )


def main():
    if MISSION == "backfill":
        run_backfill()
    else:
        run_forecast()


if __name__ == "__main__":
    main()
