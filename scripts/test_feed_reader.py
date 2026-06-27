#!/usr/bin/env python3
"""Tests for feed_reader.py pure logic — RSS/Atom/iCal parsing, the per-feed title
filter, and country guessing. No network: only pure functions are exercised. The
nightly feed reader runs unattended and writes to the review queue, so these guard
the parsing/filter paths most likely to silently break.

Runs under pytest (CI: `pytest scripts/`) or standalone (`python3 test_feed_reader.py`).
"""
import os
import sys

# feed_reader.py reads these at import; harmless test defaults (CI has no real .env).
os.environ.setdefault("SUPABASE_URL", "http://localhost")
os.environ.setdefault("SUPABASE_SERVICE_ROLE_KEY", "test")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import feed_reader as fr  # noqa: E402


# ── RSS / Atom parsing ─────────────────────────────────────────
RSS = b"""<?xml version="1.0"?><rss><channel>
  <item><title>Bear Run 2026</title><link>https://x.test/e/1</link>
    <description>&lt;p&gt;A &lt;b&gt;great&lt;/b&gt; run&lt;/p&gt;</description>
    <pubDate>Mon, 10 Aug 2026 00:00:00 GMT</pubDate></item>
  <item><title>No link here</title></item>
</channel></rss>"""


def test_rss_parses_item_and_strips_html():
    items = fr.parse_rss_items(RSS)
    assert len(items) == 1  # the link-less item is dropped
    it = items[0]
    assert it["raw_title"] == "Bear Run 2026"
    assert it["source_url"] == "https://x.test/e/1"
    assert "<b>" not in it["raw_description"] and "great" in it["raw_description"]


ATOM = b"""<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom">
  <entry><title>Bear Week PTown</title>
    <link href="https://x.test/a/1"/><summary>Summer fun</summary>
    <updated>2026-07-11T00:00:00Z</updated></entry></feed>"""


def test_atom_fallback():
    items = fr.parse_rss_items(ATOM)
    assert len(items) == 1
    assert items[0]["raw_title"] == "Bear Week PTown"
    assert items[0]["source_url"] == "https://x.test/a/1"


# ── iCal helpers ───────────────────────────────────────────────
def test_ical_date_variants():
    assert fr._parse_ical_date("20260815") == "2026-08-15"
    assert fr._parse_ical_date("20260815T180000Z") == "2026-08-15"
    assert fr._parse_ical_date("2026-08-15") == "2026-08-15"
    assert fr._parse_ical_date("garbage") is None
    assert fr._parse_ical_date("") is None


def test_ical_unfold_joins_continuation_lines():
    out = fr._ical_unfold(b"DESCRIPTION:Line one\r\n continued\r\nNEXT:x")
    assert "Line onecontinued" in out


def test_ical_unescape():
    assert fr._ical_unescape("a\\,b\\;c\\nd") == "a,b;c\nd"


# ── iCal VEVENT parsing ────────────────────────────────────────
ICS = b"""BEGIN:VCALENDAR
BEGIN:VEVENT
SUMMARY:Bear Frolic Ottawa
DTSTART;VALUE=DATE:20261008
DTEND;VALUE=DATE:20261012
URL:https://ottawabears.com/frolic
DESCRIPTION:A weekend\\, with fun
LOCATION:Ottawa\\, Canada
END:VEVENT
BEGIN:VEVENT
SUMMARY:No URL event
UID:abc-123
DTSTART:20260901
END:VEVENT
BEGIN:VEVENT
DTSTART:20260101
END:VEVENT
END:VCALENDAR"""


def test_ical_parses_vevents():
    items = fr.parse_ical_items(ICS, "https://feed.test/cal.ics")
    assert len(items) == 2  # third VEVENT has no SUMMARY -> skipped
    a, b = items
    assert a["raw_title"] == "Bear Frolic Ottawa"
    assert a["source_url"] == "https://ottawabears.com/frolic"
    assert a["parsed_start"] == "2026-10-08"
    assert a["parsed_end"] == "2026-10-12"
    assert a["raw_location"] == "Ottawa, Canada"  # unescaped
    # second has no URL -> source_url falls back to feed#uid
    assert b["source_url"] == "https://feed.test/cal.ics#abc-123"
    assert b["parsed_start"] == "2026-09-01"


# ── relevance + per-feed title filter ──────────────────────────
def test_looks_like_event():
    assert fr.looks_like_event({"raw_title": "Mr Bear France crowned", "raw_description": ""})
    assert not fr.looks_like_event({"raw_title": "Tuesday yoga", "raw_description": "stretch"})
    # iCal sources skip the keyword filter
    assert fr.looks_like_event({"raw_title": "anything", "raw_description": ""}, skip_filter=True)


def test_passes_filter_title_filter_takes_precedence():
    frolic = {"raw_title": "Bear Frolic Weekend", "raw_description": ""}
    coffee = {"raw_title": "Sunday Coffee Meetup", "raw_description": ""}
    assert fr.passes_filter(frolic, "frolic, pride", skip_filter=False)
    assert not fr.passes_filter(coffee, "frolic, pride", skip_filter=False)


def test_passes_filter_falls_back_to_keywords():
    bear = {"raw_title": "Bear Week", "raw_description": ""}
    other = {"raw_title": "Brunch", "raw_description": ""}
    assert fr.passes_filter(bear, None, skip_filter=False)
    assert not fr.passes_filter(other, "", skip_filter=False)


# ── country guessing ───────────────────────────────────────────
def test_guess_country():
    assert fr.guess_country("Big party in Berlin this weekend") == "Germany"
    assert fr.guess_country("Toronto Bears social") == "Canada"
    assert fr.guess_country("Held in Lisbon") == "Portugal"
    assert fr.guess_country("somewhere unknown") is None


if __name__ == "__main__":
    tests = [v for n, v in sorted(globals().items()) if n.startswith("test_") and callable(v)]
    failed = 0
    for fn in tests:
        try:
            fn()
            print(f"PASS {fn.__name__}")
        except AssertionError as e:
            failed += 1
            print(f"FAIL {fn.__name__}: {e}")
    print(f"\n{len(tests) - failed}/{len(tests)} passed")
    sys.exit(1 if failed else 0)
