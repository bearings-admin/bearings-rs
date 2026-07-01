#!/usr/bin/env python3
"""Tests for digest.py — the nightly research-digest builder and its email transport
selection. Guards build_digest output and, critically, that the Resend send sets a
User-Agent: a missing UA caused Cloudflare to silently 403 the digest in prod (PR #25).
No real network — urlopen is stubbed.

Runs under pytest (CI: `pytest scripts/`) or standalone (`python3 test_digest.py`).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import digest as d  # noqa: E402


def _digest():
    return d.build_digest(
        "2026-06-27T00:00:00Z",
        stats=[
            {"org": "Feed A", "parsed": 5, "new": 2, "past": 1},
            {"org": "Feed B", "parsed": 0, "new": 0, "past": 0, "error": "timeout"},
        ],
        total_new=2,
        total_past=1,
        pending_count=3,
        gaps=[{"scope": "national", "name": "Mr Bear X", "country": "USA"}],
        archived=[{"name": "Old Run 2025"}],
        predictions=[
            {"sample_name": "Y Pride", "city": "Z", "predicted_date": "2027-05-01", "confidence": "high"}
        ],
    )


def test_build_digest_subject_and_counts():
    dig = _digest()
    assert dig["subject"] == "[Bearings] research digest 2026-06-27: 2 new, 3 pending"
    b = dig["body"]
    assert "New candidates queued:  2" in b
    assert "Pending review queue:   3" in b
    assert "Feed A: 5 parsed, 2 new, 1 past" in b
    assert "ERROR: timeout" in b  # per-feed error surfaced
    assert "Mr Bear X" in b  # gaps section
    assert "Old Run 2025" in b  # archived section
    assert "Y Pride" in b  # predictions section
    assert "?zone=admin" in b


def test_build_digest_omits_empty_sections():
    dig = d.build_digest(
        "2026-06-27T00:00:00Z", stats=[], total_new=0, total_past=0, pending_count=0, gaps=[]
    )
    b = dig["body"]
    assert "Competitions missing a title holder" not in b
    assert "Archived now-past events" not in b
    assert "Likely to repeat" not in b


class _FakeResp:
    def __init__(self, status=200):
        self.status = status

    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False


def _stub_urlopen(capture):
    def fake(req, timeout=0):
        capture["req"] = req
        return _FakeResp(200)

    return fake


def _clear_transport_env():
    for key in ("RESEND_API_KEY", "DIGEST_SMTP_HOST", "DIGEST_FROM", "DIGEST_TO"):
        os.environ.pop(key, None)


def test_send_digest_resend_sets_user_agent():
    """Regression guard for PR #25: Resend's API is behind Cloudflare, which 403s the
    default urllib User-Agent (error 1010). The Resend send MUST set a real UA."""
    _clear_transport_env()
    os.environ["RESEND_API_KEY"] = "re_test"
    cap = {}
    orig = d.urlopen
    d.urlopen = _stub_urlopen(cap)
    try:
        d.send_digest({"subject": "s", "body": "b"})
    finally:
        d.urlopen = orig
        _clear_transport_env()
    req = cap.get("req")
    assert req is not None, "Resend path was not taken"
    assert "api.resend.com" in req.full_url
    ua = req.get_header("User-agent")
    assert ua and "Bearings" in ua


def test_send_digest_no_transport_is_log_only():
    """With no transport configured, send_digest must not raise (it just logs)."""
    _clear_transport_env()
    d.send_digest({"subject": "s", "body": "b"})  # must not raise / touch the network


def test_build_digest_titleholder_queue_and_agent_health():
    dig = d.build_digest(
        "2026-06-27T00:00:00Z", stats=[], total_new=5, total_past=0, pending_count=3, gaps=[],
        pending_th=4, agents_7d={"discover": 6, "auto_apply": 2, "lineage": 1},
    )
    b = dig["body"]
    assert "3 events, 4 titleholder proposals" in b
    assert "Agents (last 7 days) — 9 action(s):" in b
    assert "discover: 6" in b
    assert dig["subject"].endswith("7 pending")  # 3 events + 4 titleholder proposals


def test_build_digest_warns_when_backlog_builds():
    dig = d.build_digest(
        "2026-06-27T00:00:00Z", stats=[], total_new=0, total_past=0, pending_count=38, gaps=[],
        pending_th=5,  # 38 + 5 = 43 >= 40
    )
    assert "Review backlog is building" in dig["body"]


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
