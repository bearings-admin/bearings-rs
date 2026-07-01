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

# Mission selector: "forecast" (default, weekly date-confirmation), "backfill"
# (research PAST editions of single-edition series), or "lineage" (research missing
# YEARS of a titleholder lineage from official Hall-of-Fame / press sources).
MISSION = os.environ.get("KEEPER_MISSION", "forecast").strip().lower()
# Cap per-run work for the backfill/lineage missions (API cost + politeness to sites).
BACKFILL_LIMIT = int(os.environ.get("KEEPER_BACKFILL_LIMIT", "8"))
LINEAGE_LIMIT = int(os.environ.get("KEEPER_LINEAGE_LIMIT", "6"))


def load_directive(fname):
    p = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "directives", fname
    )
    return open(p, encoding="utf-8").read().split("\n---\n", 1)[-1].strip()


# Loaded only when the matching mission runs.
BACKFILL_PROMPT = (
    load_directive("historical_backfill.md") if MISSION == "backfill" else ""
)
LINEAGE_PROMPT = (
    load_directive("lineage_harvest.md") if MISSION == "lineage" else ""
)
DISCOVER_PROMPT = (
    load_directive("in_language_discovery.md") if MISSION == "discover" else ""
)

# In-language discovery rotation: one language per weekday (Mon=0 .. Sun=6). The
# keeper otherwise searches only in English, so this is the seam that reaches
# local-language listings (esp. Southern-hemisphere summer events that fill the
# Northern-winter gap). Override a run with KEEPER_LANG=<language>.
LANG_ROTATION = [
    ("Portuguese", ["Brazil", "Portugal"]),                                    # Mon
    ("Spanish", ["Mexico", "Spain", "Argentina", "Chile", "Colombia",
                 "Costa Rica", "Uruguay", "Venezuela", "Puerto Rico"]),        # Tue
    ("German", ["Germany", "Austria", "Switzerland"]),                         # Wed
    ("Italian", ["Italy"]),                                                    # Thu
    ("French", ["France", "Belgium", "Canada", "Switzerland", "Luxembourg"]),  # Fri
    ("Thai", ["Thailand"]),                                                    # Sat
    ("Dutch", ["Netherlands", "Belgium"]),                                     # Sun
]


def todays_language():
    """Return (language, [countries]) — KEEPER_LANG overrides the weekday rotation."""
    forced = os.environ.get("KEEPER_LANG", "").strip()
    if forced:
        for lang, countries in LANG_ROTATION:
            if lang.lower() == forced.lower():
                return lang, countries
        return forced, [forced]  # unknown language: search it broadly
    return LANG_ROTATION[date.today().weekday()]


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


def _norm(s):
    """Lowercase + collapse whitespace for tolerant substring comparison."""
    return re.sub(r"\s+", " ", (s or "").lower()).strip()


def _event_key(name):
    """Dedup key: drop a trailing edition year so 'X Bear Pride 2026' == 'X Bear Pride'."""
    return _norm(re.sub(r"\s*\b(19|20)\d{2}\b\s*$", "", name or ""))


def evidence_in_page(evidence, page):
    """True only if the model's quoted evidence actually occurs in the fetched
    page (case/whitespace-insensitive). The keeper directive asks for a *verbatim
    quote from the page*; verifying it here stops a hallucinated or injected quote
    from passing the auto-apply gate. Ambiguous cases fall back to human review."""
    ev = _norm(evidence).strip("\"'“”‘’ .")
    if len(ev) < 12:  # substantive length is gated in is_slam_dunk (raw ev >= 25)
        return False
    return ev in _norm(page)


def verify_evidence(source_url, evidence):
    """Fetch the claimed source and check the model's quote really appears on it —
    the forecast keeper's auto-apply guard, applied to the (human-reviewed) discovery
    and lineage proposals so fabricated rows don't reach the review queue. Returns:
      'verified'   — the quote is on the page,
      'unverified' — page fetched but the quote is NOT there (likely fabricated),
      'unchecked'  — no usable source_url, or the page could not be fetched.
    """
    if not source_url or not str(source_url).startswith("http"):
        return "unchecked"
    try:
        page = fetch_text(source_url)
    except Exception:
        return "unchecked"
    return "verified" if evidence_in_page(evidence, page) else "unverified"


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
    result = parse_json(claude(prompt))
    if not isinstance(result, dict):
        result = {}
    # Keep the fetched page so the auto-apply gate can verify the model's quoted
    # evidence really appears in it (guards a hallucinated/injected quote).
    result["_page"] = page
    return result


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
    if not evidence_in_page(ev, found.get("_page", "")):
        return False, "evidence quote not found on page (possible hallucination)"
    return True, "official site + verified dated quote + in-window"


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


def find_lineage(target):
    title = target.get("title_name", "")
    country = target.get("country") or ""
    have = ", ".join(str(y) for y in (target.get("held_years") or [])) or "(none)"
    prompt = (
        LINEAGE_PROMPT.replace("{{TITLE}}", title)
        .replace("{{COUNTRY}}", country)
        .replace("{{HAVE}}", have)
    )
    try:
        out = claude(prompt, tools=WEB_SEARCH_TOOL, max_tokens=2000)
    except Exception as e:
        return {"error": str(e)}
    return {"holders": parse_json_list(out)}


def queue_lineage_proposal(target, h, verify="unchecked"):
    year = str(h.get("year") or "")[:4]
    cand = {
        "title_name": target["title_name"],
        "holder_name": h.get("name") or h.get("holder_name"),
        "year": int(year) if year.isdigit() else None,
        "city": h.get("city") or None,
        "country": target.get("country") or None,
        "competition_id": target.get("competition_id"),
        "source_url": (h.get("source_url") or h.get("source") or "")[:300] or None,
        "evidence": (f"[{verify}] " + (h.get("evidence") or ""))[:300] or None,
        "status": "pending",
    }
    cand = {k: v for k, v in cand.items() if v is not None}
    created = supa_post(
        "candidate_title_holders",
        cand,
        prefer="resolution=ignore-duplicates,return=representation",
    )
    return created[0]["id"] if isinstance(created, list) and created else None


def run_lineage():
    targets = supa_get(
        "titleholder_lineage_status?select=title_name,competition_id,country,"
        "first_year,last_year,holders,gap_years,held_years"
        # Only titles worth researching: an interior gap, or very thin (≤2 holders,
        # likely truncated). Biggest gaps first so we hit Poland/Montréal, not the
        # obscure single-entry titles.
        "&or=(gap_years.gt.0,holders.lte.2)"
        f"&order=gap_years.desc,holders.asc&limit={LINEAGE_LIMIT}"
    )
    this_year = date.today().year
    print(
        f"[keeper] lineage harvest: scanning {len(targets)} titles for missing "
        f"years (model={MODEL})\n"
    )
    queued = 0
    for t in targets:
        held = set(t.get("held_years") or [])
        span = (t.get("last_year") or 0) - (t.get("first_year") or 0) + 1
        # Skip titles that are already a complete contiguous run.
        if span <= len(held) and len(held) >= 3:
            print(f"  ·  {t['title_name']}: no gaps to fill")
            continue
        r = find_lineage(t)
        if r.get("error"):
            print(f"  ! {t['title_name']}: research failed — {r['error']}")
            continue
        new = [
            h
            for h in r.get("holders", [])
            if str(h.get("year") or "")[:4].isdigit()
            and int(str(h["year"])[:4]) <= this_year
            and int(str(h["year"])[:4]) not in held
            and (h.get("name") or h.get("holder_name"))
        ]
        if not new:
            print(f"  ·  {t['title_name']}: no new years found")
            continue
        for h in new:
            v = verify_evidence(h.get("source_url"), h.get("evidence"))
            if v == "unverified":
                print(f"  ·  {t['title_name']} {h.get('year')}: skipped "
                      "(quote not found on source — likely fabricated)")
                continue
            cid = queue_lineage_proposal(t, h, v)
            audit(
                "lineage",
                cid,
                None,
                {"sample_name": t["title_name"]},
                {"evidence": h.get("evidence", "")},
                f"year {h.get('year')}",
            )
            queued += 1
            print(
                f"  +  {t['title_name']} {h.get('year')}: "
                f"{h.get('name') or h.get('holder_name')} -> queued (cand {cid})"
            )
    print(
        f"\n[keeper] lineage harvest: {queued} titleholder proposal(s) "
        f"queued for review (no auto-insert)."
    )


def queue_discovery(lang, ev, verify="unchecked"):
    start = (ev.get("start_date") or "").strip()
    end = (ev.get("end_date") or "").strip()
    cand = {
        "raw_title": (ev.get("name") or "").strip(),
        "raw_description": (
            f"Keeper in-language discovery ({lang}, {MODEL}; {verify}). "
            f"{ev.get('city') or ''}. Evidence: {(ev.get('evidence') or '')[:200]}"
        ),
        "raw_date": (start.replace("-", "") if start else None),
        "parsed_start": start or None,
        "parsed_end": end or None,
        "parsed_city": ev.get("city") or None,
        "parsed_country": ev.get("country") or None,
        "parsed_type": ev.get("type") or "bear-run",
        "source_url": (ev.get("source_url") or "")[:300] or None,
        "status": "pending",
    }
    cand = {k: v for k, v in cand.items() if v is not None}
    created = supa_post(
        "candidate_events",
        cand,
        prefer="resolution=ignore-duplicates,return=representation",
    )
    return created[0]["id"] if isinstance(created, list) and created else None


def run_discover():
    lang, countries = todays_language()
    # Existing active events in these countries — dedup hint for Claude + a hard
    # code-side dedup before we queue anything.
    all_ev = supa_get("events?select=name,country&active=eq.true&limit=1000")
    cset = set(countries)
    in_scope = [e["name"] for e in all_ev if e.get("country") in cset]
    have = {_event_key(n) for n in in_scope}  # year-stripped so "X 2026" == "X"
    have_names = in_scope[:60]
    prompt = (
        DISCOVER_PROMPT.replace("{{LANG}}", lang)
        .replace("{{COUNTRIES}}", ", ".join(countries))
        .replace("{{EXISTING}}", "; ".join(have_names) or "(none yet)")
    )
    print(
        f"[keeper] in-language discovery: {lang} — {', '.join(countries)} "
        f"(model={MODEL})\n"
    )
    try:
        out = claude(prompt, tools=WEB_SEARCH_TOOL, max_tokens=2500)
    except Exception as e:
        print(f"  ! search failed — {e}")
        return
    today = date.today().isoformat()
    queued = 0
    for ev in parse_json_list(out):
        name = (ev.get("name") or "").strip()
        start = (ev.get("start_date") or "").strip()
        if not name or not start or not ev.get("source_url"):
            continue
        if start < today:  # discovery is forward-looking; past editions are backfill's job
            print(f"  ·  {name}: skipped (past — {start})")
            continue
        if _event_key(name) in have:
            print(f"  ·  {name}: already have it")
            continue
        v = verify_evidence(ev.get("source_url"), ev.get("evidence"))
        if v == "unverified":
            print(f"  ·  {name}: skipped (quote not found on source — likely fabricated)")
            continue
        have.add(_event_key(name))  # guard against dups within this batch
        cid = queue_discovery(lang, ev, v)
        audit(
            "discover",
            cid,
            None,
            {"sample_name": name},
            {"evidence": ev.get("evidence", "")},
            f"{lang}: {ev.get('country', '')} [{v}]",
        )
        queued += 1
        print(
            f"  +  {name} ({ev.get('city', '')}, {start}) -> queued (cand {cid})"
        )
    print(
        f"\n[keeper] in-language discovery ({lang}): {queued} new event(s) "
        f"queued for steward review (no auto-insert)."
    )


def main():
    if MISSION == "backfill":
        run_backfill()
    elif MISSION == "lineage":
        run_lineage()
    elif MISSION == "discover":
        run_discover()
    else:
        run_forecast()


if __name__ == "__main__":
    main()
