#!/usr/bin/env python3
"""Tests for link_check.py classification logic — the parked/dead/blocked decisions that
decide what gets flagged to the steward. No live network: classify_body is pure, and the
classify() HTTP-status branches are exercised with a stubbed urlopen.

Runs under pytest (CI: `pytest scripts/`) or standalone (`python3 test_link_check.py`).
"""
import os
import sys
from urllib.error import HTTPError, URLError

os.environ.setdefault("SUPABASE_URL", "http://localhost")
os.environ.setdefault("SUPABASE_SERVICE_ROLE_KEY", "test")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import link_check as lc  # noqa: E402

lc.RETRIES = 0  # no retry sleeps in tests
lc.RETRY_BACKOFF = 0


# ── pure classifier ────────────────────────────────────────────
def test_parked_by_redirect_host():
    s, _ = lc.classify_body("https://www.hugedomains.com/domain_profile.cfm?d=foo", 200, "buy now")
    assert s == "parked"


def test_parked_by_body_marker():
    s, _ = lc.classify_body("https://saunaoasis.com/", 200, "<h1>this domain is for sale</h1>")
    assert s == "parked"


def test_dead_by_http_code():
    assert lc.classify_body("https://x.test/", 404, "not found")[0] == "dead"


def test_ok_clean_page():
    assert lc.classify_body("https://praguebears.cz/", 200, "welcome to prague bears")[0] == "ok"


def test_venue_saying_tickets_for_sale_is_not_parked():
    # guard against over-broad markers flagging a live venue
    s, _ = lc.classify_body("https://beefdip.com/", 200, "tickets for sale now, join us!")
    assert s == "ok"


# ── classify() network branches (stubbed urlopen) ──────────────
class _Resp:
    def __init__(self, body=b"ok", status=200, url="https://x.test/"):
        self._b, self.status, self._u = body, status, url

    def geturl(self):
        return self._u

    def read(self, n=-1):
        return self._b

    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False


def _with_urlopen(fake, fn):
    orig = lc.urlopen
    lc.urlopen = fake
    try:
        return fn()
    finally:
        lc.urlopen = orig


def test_classify_403_is_blocked_not_dead():
    def fake(req, timeout=0, **kw):
        raise HTTPError("https://x.test/", 403, "Forbidden", None, None)
    s, _ = _with_urlopen(fake, lambda: lc.classify("https://x.test/"))
    assert s == "blocked"


def test_classify_404_is_dead():
    def fake(req, timeout=0, **kw):
        raise HTTPError("https://x.test/", 404, "Not Found", None, None)
    s, _ = _with_urlopen(fake, lambda: lc.classify("https://x.test/"))
    assert s == "dead"


def test_classify_unreachable_is_dead():
    def fake(req, timeout=0, **kw):
        raise URLError("name or service not known")
    s, _ = _with_urlopen(fake, lambda: lc.classify("https://nope.invalid/"))
    assert s == "dead"


def test_classify_non_http_is_skipped():
    assert lc.classify("mailto:x@y.com")[0] == "skip"


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
