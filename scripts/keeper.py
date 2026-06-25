#!/usr/bin/env python3
"""Bearings keeper — v1: forecast confirmation.

Reads predicted recurrences (the `event_predictions` view), fetches each series'
official website, and asks Claude whether the *next* edition's dates have actually
been announced. When it finds confirmed dates it queues a REVIEWABLE proposal into
the admin review queue (`candidate_events`, status=pending) — it never writes to
`events` directly (the steward approves in the admin panel, one click). Approving a
confirmation also makes the corresponding forecast resolve itself.

Zero third-party deps (urllib, matching feed_reader.py): talks to Supabase
PostgREST and the Anthropic Messages API over raw HTTP. Keys come from
/opt/bearings-rs/.env (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY, ANTHROPIC_API_KEY).
Model via KEEPER_MODEL (default claude-opus-4-8; claude-haiku-4-5 for cheap runs).
"""
import os
import re
import json
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
UA = "Bearings-Keeper/1.0 (+https://bearings.community)"


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
        f'You verify bear-event dates for a community directory. Event: "{name}" in {city}. '
        f"Using ONLY the website text below, determine whether the {year} edition's dates are "
        f"announced. Do not guess or use prior knowledge.\n\n"
        f"Respond with ONLY a JSON object, no prose:\n"
        f'{{"announced": true|false, "start_date": "YYYY-MM-DD or empty", '
        f'"end_date": "YYYY-MM-DD or empty", "evidence": "short quote from the page or empty"}}\n\n'
        f"WEBSITE TEXT:\n{page}"
    )
    return parse_json(claude(prompt))


def queue_proposal(pred, year, found):
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
    cand = {k: v for k, v in cand.items() if v is not None}
    return supa_post("candidate_events", cand)


def main():
    preds = supa_get(
        "event_predictions?select=sample_name,city,country,predicted_date,confidence,website"
        "&order=predicted_date"
    )
    print(f"[keeper] checking {len(preds)} forecasted series for announced dates (model={MODEL})\n")
    queued = 0
    for p in preds:
        site = p.get("website")
        if not site:
            print(f"  - {p['sample_name']}: no website on file — skip")
            continue
        r = check(p)
        year = (p.get("predicted_date") or "")[:4]
        if r.get("error"):
            print(f"  ! {p['sample_name']}: fetch/check failed — {r['error']}")
        elif r.get("announced") and r.get("start_date"):
            print(
                f"  CONFIRM  {p['sample_name']} {year}: {r['start_date']} -> {r.get('end_date', '')} "
                f"(forecast ~{p['predicted_date']})\n"
                f"           source:   {site}\n"
                f"           evidence: {r.get('evidence', '')[:160]}"
            )
            code = queue_proposal(p, year, r)
            print(f"           -> queued to admin review queue (HTTP {code})")
            queued += 1
        else:
            print(f"  ·  {p['sample_name']}: {year} not announced yet on {site}")
    print(
        f"\n[keeper] {queued} confirmation(s) queued for steward review "
        f"in the admin panel (no auto-insert)."
    )


if __name__ == "__main__":
    main()
