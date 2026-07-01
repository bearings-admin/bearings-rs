#!/usr/bin/env python3
"""Bearings link checker — flags dead or parked website links across the directory.

Motivation: a steward eyeballing a zone caught a sauna whose website had lapsed to a
domain-for-sale page (2026-06-28). This automates that QA: it scans the URL columns on
places / clubs / competitions / digital_spaces, does a polite GET on each, and classifies:
  - dead    : 404/410/5xx, DNS failure, connection refused, timeout
  - parked  : the final page is a domain-for-sale / parking page
  - blocked : 401/403/429 (often bot-blocking on a *live* site — reported, not alarmed)
  - ok

Proposes-never-mutates: it never edits rows. It prints a report, writes a log, and (if
anything is flagged) emails the steward via the shared digest transport. The steward fixes
flagged rows by hand. Zero deps (urllib), matching feed_reader.py / keeper.py.

Run:      python3 scripts/link_check.py     (env from /opt/bearings-rs/.env)
Schedule: bearings-linkcheck.timer (weekly).
Tune:     LINKCHECK_TIMEOUT (s, default 12), LINKCHECK_SLEEP (s between requests, 0.4).
"""
import json
import os
import re
import ssl
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


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
UA = "Bearings-LinkCheck/1.0 (+https://bearings.community; ursasteward@pm.me)"
TIMEOUT = int(os.environ.get("LINKCHECK_TIMEOUT", "12"))
SLEEP = float(os.environ.get("LINKCHECK_SLEEP", "0.4"))
RETRIES = int(os.environ.get("LINKCHECK_RETRIES", "1"))   # retry transient DNS/timeout
RETRY_BACKOFF = float(os.environ.get("LINKCHECK_BACKOFF", "4.0"))
# Reachability check, not a cert audit: an expired/mismatched cert doesn't mean the site
# is gone, so don't let TLS verification mask a live page. (We never transact here.)
_SSL_CTX = ssl._create_unverified_context()

# (table, id_col, name_col, url_col, extra PostgREST filter)
TARGETS = [
    ("places", "id", "name", "website", "&active=eq.true"),
    ("clubs", "id", "name", "website", "&active=eq.true"),
    ("competitions", "id", "name", "website", "&active=eq.true"),
    ("digital_spaces", "id", "name", "url", ""),
]

# Domain-parking hosts a dead domain typically redirects to.
PARKED_HOSTS = [
    "sedo", "dan.com", "afternic", "hugedomains", "parkingcrew", "bodis",
    "undeveloped", "domainmarket", "namedrive", "above.com",
]
# Phrases specific to for-sale / parking pages (kept tight to avoid false positives
# like a venue advertising "tickets for sale").
PARKED_MARKERS = [
    "this domain is for sale", "buy this domain", "domain is for sale",
    "the domain name is for sale", "domain for sale", "this website is for sale",
    "is for sale. make an offer", "parked free", "domain parking",
    "interested in this domain",
]


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


def classify(url):
    """Return (status, detail): ok / dead / parked / blocked / skip. Pure-ish — the
    network call is the only side effect; the parked/dead/blocked logic is tested."""
    if not url or not url.startswith("http"):
        return ("skip", "not an http url")
    # HTTPError is definitive (server answered) — never retry it. Transient network/DNS
    # failures get RETRIES attempts with backoff before we call the link dead.
    for attempt in range(RETRIES + 1):
        try:
            req = Request(url, headers={"User-Agent": UA})
            with urlopen(req, timeout=TIMEOUT, context=_SSL_CTX) as r:
                final = r.geturl()
                code = getattr(r, "status", 200)
                body = r.read(6000).decode("utf-8", "replace").lower()
            return classify_body(final, code, body)
        except HTTPError as e:
            if e.code in (401, 403, 429):
                return ("blocked", f"HTTP {e.code} (bot-blocked?)")
            return ("dead", f"HTTP {e.code}")
        except (URLError, OSError) as e:
            if attempt < RETRIES:
                time.sleep(RETRY_BACKOFF * (attempt + 1))
                continue
            reason = getattr(e, "reason", e)
            return ("dead", f"unreachable after {RETRIES + 1} tries ({reason})")
        except Exception as e:
            return ("dead", f"error ({type(e).__name__})")


def classify_body(final_url, code, body):
    """Pure classifier for a fetched page — split out so it can be unit-tested."""
    host = re.sub(r"^https?://", "", final_url).split("/")[0].lower()
    if any(h in host for h in PARKED_HOSTS) or any(m in body for m in PARKED_MARKERS):
        return ("parked", f"for-sale/parking page -> {host}")
    if code >= 400:
        return ("dead", f"HTTP {code}")
    return ("ok", "")


def run():
    dead, parked, blocked, checked = [], [], [], 0
    for table, idc, namec, urlc, filt in TARGETS:
        try:
            rows = supa_get(f"{table}?select={idc},{namec},{urlc}&{urlc}=not.is.null{filt}")
        except Exception as e:
            print(f"[linkcheck] {table}: fetch failed — {e}")
            continue
        for row in rows:
            url = (row.get(urlc) or "").strip()
            if not url:
                continue
            status, detail = classify(url)
            checked += 1
            entry = f"{table}#{row[idc]} {row.get(namec, '')} — {url}  ({detail})"
            if status == "dead":
                dead.append(entry)
            elif status == "parked":
                parked.append(entry)
            elif status == "blocked":
                blocked.append(entry)
            time.sleep(SLEEP)

    flagged = len(dead) + len(parked)
    L = [f"Bearings link check — {checked} links scanned, {flagged} flagged.", ""]
    if parked:
        L += [f"PARKED / FOR SALE ({len(parked)}):"] + [f"  - {e}" for e in parked] + [""]
    if dead:
        L += [f"DEAD ({len(dead)}):"] + [f"  - {e}" for e in dead] + [""]
    if not flagged:
        L += ["No dead or parked links. ✓", ""]
    if blocked:
        L += [f"Couldn't verify — bot-blocked, likely still live ({len(blocked)}):"]
        L += [f"  - {e}" for e in blocked]
    body = "\n".join(L)
    print(body)

    try:
        os.makedirs("/opt/bearings-rs/logs", exist_ok=True)
        with open("/opt/bearings-rs/logs/linkcheck-latest.txt", "w", encoding="utf-8") as f:
            f.write(body + "\n")
    except Exception:
        pass

    if flagged:
        try:
            from digest import send_digest  # reuse the tested Resend/SMTP transport
            send_digest({"subject": f"[Bearings] link check: {flagged} flagged link(s)", "body": body})
        except Exception as e:
            print(f"[linkcheck] email skipped: {e}")


if __name__ == "__main__":
    run()
