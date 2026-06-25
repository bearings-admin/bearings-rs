#!/usr/bin/env python3
"""Bearings keeper — v1: forecast confirmation.

Reads predicted recurrences (the `event_predictions` view), fetches each series'
official website, and asks Claude whether the *next* edition's dates have actually
been announced. Prints REVIEWABLE proposals — it never writes to `events` (the
steward confirms). This is the first, narrowly-scoped keeper mission; it reuses the
forecast as a prioritized worklist (what to look for, and where).

Zero third-party deps (urllib, matching feed_reader.py): talks to Supabase
PostgREST and the Anthropic Messages API over raw HTTP. Keys come from
/opt/bearings-rs/.env (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY, ANTHROPIC_API_KEY).
Model is claude-opus-4-8 by default; set KEEPER_MODEL in .env to use a cheaper one
(e.g. claude-haiku-4-5) for this routine extraction.
"""
import os
import re
import json
from urllib.request import urlopen, Request


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


def main():
    preds = supa_get(
        "event_predictions?select=sample_name,city,country,predicted_date,confidence,website"
        "&order=predicted_date"
    )
    print(f"[keeper] checking {len(preds)} forecasted series for announced dates (model={MODEL})\n")
    proposals = []
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
            proposals.append(
                {**p, "found_start": r["start_date"], "found_end": r.get("end_date", "")}
            )
        else:
            print(f"  ·  {p['sample_name']}: {year} not announced yet on {site}")
    print(
        f"\n[keeper] {len(proposals)} confirmable date(s) found — "
        f"review before adding (no auto-insert)."
    )


if __name__ == "__main__":
    main()
