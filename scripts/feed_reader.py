#!/usr/bin/env python3
"""
Bearings feed reader — runs nightly via systemd timer.
Handles feed_type: rss, ical, ical-static
Fetches watched_feeds where active=true, parses items,
inserts new ones into candidate_events.

Usage: python3 /opt/bearings-rs/scripts/feed_reader.py
"""

import os, sys, json, re, xml.etree.ElementTree as ET
from datetime import datetime, timezone, date
from urllib.request import urlopen, Request
from urllib.error import URLError, HTTPError
from digest import build_digest, write_log, send_digest

SUPABASE_URL = os.environ["SUPABASE_URL"]
SUPABASE_KEY = os.environ["SUPABASE_SERVICE_ROLE_KEY"]

HEADERS = {
    "apikey": SUPABASE_KEY,
    "Authorization": f"Bearer {SUPABASE_KEY}",
    "Content-Type": "application/json",
}

UA = "Bearings-FeedReader/1.0 (+https://bearings.community)"

def api_get(path):
    req = Request(f"{SUPABASE_URL}/rest/v1/{path}",
                  headers={**HEADERS, "Accept": "application/json"})
    with urlopen(req, timeout=15) as r:
        return json.loads(r.read())

def api_post(path, data, prefer="return=minimal"):
    body = json.dumps(data).encode()
    req = Request(f"{SUPABASE_URL}/rest/v1/{path}", data=body,
                  headers={**HEADERS, "Prefer": prefer}, method="POST")
    try:
        with urlopen(req, timeout=15) as r:
            return r.status, r.read()
    except HTTPError as e:
        return e.code, e.read()

def api_patch(path, data):
    body = json.dumps(data).encode()
    req = Request(f"{SUPABASE_URL}/rest/v1/{path}", data=body,
                  headers=HEADERS, method="PATCH")
    try:
        with urlopen(req, timeout=15) as r:
            return r.status
    except HTTPError as e:
        return e.code

# ── HTTP fetch with conditional headers ───────────────────────
def fetch_url(url, etag=None, last_mod=None):
    req = Request(url, headers={"User-Agent": UA})
    if etag:
        req.add_header("If-None-Match", etag)
    if last_mod:
        req.add_header("If-Modified-Since", last_mod)
    try:
        with urlopen(req, timeout=30) as r:
            new_etag = r.headers.get("ETag")
            new_lmod = r.headers.get("Last-Modified")
            return r.read(), new_etag, new_lmod, r.status
    except HTTPError as e:
        if e.code == 304:
            return None, etag, last_mod, 304
        raise

# ── RSS 2.0 / Atom parser ──────────────────────────────────────
def parse_rss_items(xml_bytes):
    root = ET.fromstring(xml_bytes)
    items = []
    for item in root.iter("item"):
        t = (item.findtext("title") or "").strip()
        u = (item.findtext("link") or "").strip()
        d = (item.findtext("description") or "").strip()
        pub = (item.findtext("pubDate") or "").strip()
        d_clean = re.sub(r"<[^>]+>", " ", d).strip()
        d_clean = re.sub(r"\s+", " ", d_clean)[:600]
        if t and u:
            items.append({"raw_title": t, "source_url": u,
                          "raw_description": d_clean, "raw_date": pub})
    if not items:
        ns = "http://www.w3.org/2005/Atom"
        for entry in root.iter(f"{{{ns}}}entry"):
            t = (entry.findtext(f"{{{ns}}}title") or "").strip()
            link_el = entry.find(f"{{{ns}}}link")
            u = (link_el.get("href") if link_el is not None else "") or ""
            d = (entry.findtext(f"{{{ns}}}summary") or
                 entry.findtext(f"{{{ns}}}content") or "").strip()
            pub = (entry.findtext(f"{{{ns}}}updated") or "").strip()
            d_clean = re.sub(r"<[^>]+>", " ", d).strip()[:600]
            if t and u:
                items.append({"raw_title": t, "source_url": u,
                              "raw_description": d_clean, "raw_date": pub})
    return items

# ── iCal / .ics parser ─────────────────────────────────────────
def _ical_unescape(val):
    # Unescape iCal text: backslash-n, backslash-comma, backslash-semicolon, backslash-colon
    return val.replace("\\n", "\n").replace("\\,", ",") \
               .replace("\\;", ";").replace("\\:", ":")

def _ical_unfold(raw_bytes):
    """Unfold continued lines (RFC 5545 §3.1)."""
    text = raw_bytes.decode("utf-8", errors="replace")
    return re.sub(r"\r?\n[ \t]", "", text)

def _parse_ical_date(val):
    """Parse DATE or DATETIME value to ISO date string. Returns None on failure."""
    val = val.split(";")[-1]   # strip param junk if present
    val = val.strip().rstrip("Z").replace("-", "")
    try:
        if len(val) >= 8:
            return date(int(val[0:4]), int(val[4:6]), int(val[6:8])).isoformat()
    except (ValueError, IndexError):
        pass
    return None

def parse_ical_items(raw_bytes, feed_url):
    """
    Parse a .ics file and return a list of dicts matching the RSS item shape.
    Each VEVENT becomes one item. source_url is the URL field or the feed_url
    with a UID fragment if no URL property is present.
    """
    text = _ical_unfold(raw_bytes)
    items = []
    # Split into VEVENT blocks
    for block in re.split(r"BEGIN:VEVENT", text)[1:]:
        end = block.find("END:VEVENT")
        if end != -1:
            block = block[:end]

        def prop(name):
            # Match PROPERTY or PROPERTY;params: value
            m = re.search(
                r"(?:^|\n)" + re.escape(name) + r"(?:;[^\r\n:]*)?:([^\r\n]*)",
                block, re.IGNORECASE
            )
            return _ical_unescape(m.group(1).strip()) if m else ""

        summary  = prop("SUMMARY")
        uid      = prop("UID")
        url      = prop("URL")
        desc     = prop("DESCRIPTION")
        location = prop("LOCATION")
        dtstart  = prop("DTSTART")
        dtend    = prop("DTEND")

        if not summary:
            continue

        # Build a stable source_url: prefer URL field, else feed_url#uid
        if url:
            source_url = url
        elif uid:
            source_url = f"{feed_url}#{uid}"
        else:
            continue  # can't deduplicate without a key

        start_iso = _parse_ical_date(dtstart) if dtstart else None
        end_iso   = _parse_ical_date(dtend)   if dtend   else None

        desc_clean = re.sub(r"<[^>]+>", " ", desc).strip()
        desc_clean = re.sub(r"\s+", " ", desc_clean)[:600]

        items.append({
            "raw_title":       summary,
            "source_url":      source_url,
            "raw_description": desc_clean,
            "raw_date":        dtstart,
            "raw_location":    location,
            "parsed_start":    start_iso,
            "parsed_end":      end_iso,
        })
    return items

# ── Event relevance filter ─────────────────────────────────────
EVENT_KW = [
    "bear run", "bear week", "bear weekend", "bear festival", "bear party",
    "bear pride", "mr bear", "mr. bear", "title holder", "crowned",
    "competition", "bear bar", "leather bar", "leather week", "fetish week",
    "folsom", "tbru", "nab weekend", "ibr", "lazybear", "lazy bear",
    "provincetown bear", "bear world", "bear events", "opens", "closes",
    "new venue", "bear club",
]

# iCal feeds are already bear/leather-specific sources — skip the keyword
# filter for them so all events pass through (steward reviews in admin queue).
ICAL_FEEDS_SKIP_FILTER = True

def looks_like_event(item, skip_filter=False):
    if skip_filter:
        return True
    text = (item["raw_title"] + " " + item.get("raw_description", "")).lower()
    return any(kw in text for kw in EVENT_KW)

# ── Country hint extraction ────────────────────────────────────
COUNTRY_MAP = {
    "usa": "USA", "united states": "USA", "germany": "Germany",
    "uk": "UK", "united kingdom": "UK", "england": "UK", "scotland": "UK",
    "australia": "Australia", "france": "France", "netherlands": "Netherlands",
    "spain": "Spain", "canada": "Canada", "brazil": "Brazil",
    "thailand": "Thailand", "portugal": "Portugal", "belgium": "Belgium",
    "ireland": "Ireland", "italy": "Italy", "sweden": "Sweden",
    "norway": "Norway", "japan": "Japan", "south africa": "South Africa",
    "new zealand": "New Zealand", "malaysia": "Malaysia",
    "singapore": "Singapore", "philippines": "Philippines",
    "mexico": "Mexico", "argentina": "Argentina", "colombia": "Colombia",
    "iceland": "Iceland", "denmark": "Denmark", "finland": "Finland",
    "austria": "Austria", "switzerland": "Switzerland", "poland": "Poland",
    "czech": "Czech Republic", "luxembourg": "Luxembourg",
    "amsterdam": "Netherlands", "berlin": "Germany", "london": "UK",
    "paris": "France", "barcelona": "Spain", "madrid": "Spain",
    "gran canaria": "Spain", "maspalomas": "Spain",
    "chicago": "USA", "san francisco": "USA", "seattle": "USA",
    "new york": "USA", "los angeles": "USA", "dallas": "USA",
    "toronto": "Canada", "montreal": "Canada", "vancouver": "Canada",
    "sydney": "Australia", "melbourne": "Australia",
    "lisbon": "Portugal", "porto": "Portugal",
}

def guess_country(text):
    tl = text.lower()
    for hint, name in COUNTRY_MAP.items():
        if hint in tl:
            return name
    return None

# ── Process one feed ───────────────────────────────────────────
def process_feed(feed):
    fid       = feed["id"]
    url       = feed["url"]
    org       = feed["org_name"]
    ftype     = feed["feed_type"]
    etag      = feed.get("last_etag")
    lmod      = feed.get("last_modified")
    is_ical   = ftype in ("ical", "ical-static")

    print(f"\n  [{org}] {url}")
    try:
        raw, new_etag, new_lmod, status = fetch_url(url, etag, lmod)
    except Exception as e:
        print(f"    ERROR: {e}")
        errors = (feed.get("fetch_errors") or 0) + 1
        api_patch(f"watched_feeds?id=eq.{fid}", {"fetch_errors": errors})
        return {"org": org, "parsed": 0, "new": 0, "past": 0, "skipped": 0, "error": str(e)}

    api_patch(f"watched_feeds?id=eq.{fid}", {
        "last_fetched":  datetime.now(timezone.utc).isoformat(),
        "last_etag":     new_etag,
        "last_modified": new_lmod,
        "fetch_errors":  0,
    })

    if status == 304 or raw is None:
        print(f"    304 Not Modified — nothing to parse")
        return {"org": org, "parsed": 0, "new": 0, "past": 0, "skipped": 0, "error": None}

    if is_ical:
        items = parse_ical_items(raw, url)
    else:
        items = parse_rss_items(raw)

    print(f"    {len(items)} items parsed")

    feed_new = feed_skip = feed_past = 0
    today_iso = date.today().isoformat()
    for item in items:
        skip_filter = is_ical and ICAL_FEEDS_SKIP_FILTER
        if not looks_like_event(item, skip_filter=skip_filter):
            feed_skip += 1
            continue
        ps = item.get("parsed_start")
        if ps and ps < today_iso:          # never queue events already in the past
            feed_past += 1
            continue

        text = item["raw_title"] + " " + item.get("raw_description", "") + " " + item.get("raw_location", "")
        candidate = {
            "feed_id":         fid,
            "source_url":      item["source_url"],
            "raw_title":       item["raw_title"],
            "raw_description": item.get("raw_description", ""),
            "raw_date":        item.get("raw_date", ""),
            "raw_location":    item.get("raw_location", ""),
            "parsed_name":     item["raw_title"],
            "parsed_country":  guess_country(text),
            "parsed_start":    item.get("parsed_start"),
            "parsed_end":      item.get("parsed_end"),
            "status":          "pending",
        }
        # Remove None values — Supabase REST doesn't need explicit nulls
        candidate = {k: v for k, v in candidate.items() if v is not None}

        code, _ = api_post(
            "candidate_events",
            candidate,
            prefer="resolution=ignore-duplicates,return=minimal"
        )
        if code in (200, 201):
            feed_new += 1
        # 409 / duplicate = silent skip

    label = "non-event items skipped" if not is_ical else "already-seen skipped"
    print(f"    {feed_new} new  |  {feed_past} past-skipped  |  {feed_skip} {label}")
    return {"org": org, "parsed": len(items), "new": feed_new,
            "past": feed_past, "skipped": feed_skip, "error": None}

# ── Main ───────────────────────────────────────────────────────

# ── Title-holder gap research ─────────────────────────────────
HOLDER_SIGNALS = re.compile(
    r"(winner|titleholder|title holder|crowned|reigning|sash|"
    r"current\s+(mr|bear)|\bmr\.?\s+bear\b|20[12][0-9])", re.I)

def report_missing_title_holders():
    """Surface active competitions with no title holders, and scan any official
    site for likely winner leads. NEVER inserts — the steward verifies a source
    and adds the record (gap records over guesses; CONST: source required)."""
    try:
        gaps = api_get(
            "competitions_missing_holders"
            "?select=name,scope,country,city,website&order=scope.asc,country.asc")
    except Exception as e:
        print(f"\n[title-holders] could not load gap view: {e}")
        return []

    if not gaps:
        print("\n[title-holders] no gaps - every active competition has a holder.")
        return []

    print(f"\n[title-holders] {len(gaps)} competitions missing holders:")
    for g in gaps:
        site = (g.get("website") or "").strip()
        has_site = site.startswith("http")
        loc = (g.get("city") or "").strip()
        print(f"    - [{g['scope']}] {g['name']} - {loc}, {g['country']}  "
              f"{site if has_site else '(no website on record)'}")
        if not has_site:
            continue
        try:
            body, *_ = fetch_url(site)
        except Exception as e:
            print(f"        site fetch failed: {e}")
            continue
        if not body:
            continue
        text = re.sub(r"<[^>]+>", " ", body.decode("utf-8", "ignore"))
        leads = []
        for line in re.split(r"[\n\.•]", text):
            line = " ".join(line.split())
            if 8 < len(line) < 160 and HOLDER_SIGNALS.search(line):
                leads.append(line)
        if leads:
            for ln in leads[:3]:
                print(f"        lead: {ln}")
        else:
            print("        (no obvious winner text - needs manual check)")

    print("[title-holders] leads are UNVERIFIED - steward confirms a source "
          "before adding (no guessed names).")
    return gaps


def main():
    ts = datetime.now(timezone.utc).isoformat()
    print(f"[{ts}] Bearings feed reader starting")

    feeds = api_get(
        "watched_feeds?active=eq.true"
        "&feed_type=in.(rss,ical,ical-static)"
        "&select=id,url,org_name,feed_type,last_etag,last_modified,fetch_errors"
        "&order=id.asc"
    )
    rss_feeds   = [f for f in feeds if f["feed_type"] == "rss"]
    ical_feeds  = [f for f in feeds if f["feed_type"] in ("ical", "ical-static")]
    print(f"  {len(rss_feeds)} RSS  |  {len(ical_feeds)} iCal  feeds active")

    stats = []
    for feed in feeds:
        stats.append(process_feed(feed))

    total_new  = sum(s["new"] for s in stats)
    total_past = sum(s["past"] for s in stats)
    print(f"\n[done] {total_new} new candidates queued  |  {total_past} past-dated skipped")

    gaps = report_missing_title_holders()

    try:
        pending_count = len(api_get("candidate_events?status=eq.pending&select=id"))
    except Exception:
        pending_count = -1

    digest = build_digest(ts, stats, total_new, total_past, pending_count, gaps)
    write_log(digest)
    send_digest(digest)

if __name__ == "__main__":
    env_path = "/opt/bearings-rs/.env"
    if os.path.exists(env_path):
        for line in open(env_path):
            line = line.strip()
            if line and "=" in line and not line.startswith("#"):
                k, v = line.split("=", 1)
                os.environ.setdefault(k.strip(), v.strip())
    main()
