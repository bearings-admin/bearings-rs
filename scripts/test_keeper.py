#!/usr/bin/env python3
"""Tests for the keeper agent's pure decision logic — especially the auto-apply
gate (is_slam_dunk / evidence_in_page), which can write live events to the public
directory. No network: only the pure functions are exercised.

Runs under pytest (CI: `pytest scripts/`) or standalone (`python3 test_keeper.py`).
"""
import os
import sys

# keeper.py reads these at import time; harmless defaults so it imports without a
# real .env (CI). On the VPS the real .env is already loaded by keeper itself.
os.environ.setdefault("SUPABASE_URL", "http://localhost")
os.environ.setdefault("SUPABASE_SERVICE_ROLE_KEY", "test")
os.environ.setdefault("ANTHROPIC_API_KEY", "test")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import keeper as k  # noqa: E402

PAGE = "The 2027 edition runs January 24-31, 2027 in Puerto Vallarta. Tickets on sale."
PRED = {"predicted_date": "2027-01-25"}


def _found(**kw):
    base = {
        "announced": True,
        "start_date": "2027-01-24",
        "evidence": "The 2027 edition runs January 24-31, 2027",
        "_page": PAGE,
    }
    base.update(kw)
    return base


def test_evidence_in_page_real_quote():
    assert k.evidence_in_page("January 24-31, 2027", PAGE) is True


def test_evidence_in_page_hallucinated_quote_rejected():
    assert k.evidence_in_page("February 2-9, 2027", PAGE) is False


def test_evidence_in_page_empty_rejected():
    assert k.evidence_in_page("", PAGE) is False


def test_evidence_in_page_whitespace_and_case_insensitive():
    assert k.evidence_in_page("january   24-31,\n2027", PAGE) is True


def test_gate_passes_real_quote_in_window():
    ok, why = k.is_slam_dunk(PRED, "2027", _found())
    assert ok, why


def test_gate_holds_hallucinated_quote():
    ok, _ = k.is_slam_dunk(
        PRED, "2027", _found(evidence="The 2027 edition runs February 2-9, 2027")
    )
    assert not ok


def test_gate_holds_out_of_window():
    ok, _ = k.is_slam_dunk(PRED, "2027", _found(start_date="2027-06-24"))
    assert not ok


def test_gate_holds_wrong_year():
    f = _found(
        start_date="2028-01-24",
        evidence="The 2028 edition runs January 24-31, 2028",
        _page="The 2028 edition runs January 24-31, 2028",
    )
    ok, _ = k.is_slam_dunk(PRED, "2027", f)
    assert not ok


def test_gate_holds_not_announced():
    ok, _ = k.is_slam_dunk(PRED, "2027", _found(announced=False))
    assert not ok


def test_next_edition_name():
    assert k.next_edition_name("BeefDip 2026", "2027") == "BeefDip 2027"
    assert k.next_edition_name("BeefDip", "2027") == "BeefDip 2027"


def test_parse_json_tolerant():
    assert k.parse_json('{"a": 1}') == {"a": 1}
    assert k.parse_json('prefix {"a": 2} suffix') == {"a": 2}
    assert k.parse_json("not json at all") == {}


def _with_fetch(stub):
    """Swap keeper.fetch_text for a stub (works under pytest and the standalone runner)."""
    orig = k.fetch_text
    k.fetch_text = stub
    return orig


def test_verify_evidence_verified():
    orig = _with_fetch(lambda url: "The 2027 edition runs January 24-31, 2027 in PV.")
    try:
        assert k.verify_evidence("https://x.com", "runs January 24-31, 2027") == "verified"
    finally:
        k.fetch_text = orig


def test_verify_evidence_unverified_when_quote_absent():
    orig = _with_fetch(lambda url: "totally unrelated page content here")
    try:
        assert k.verify_evidence("https://x.com", "runs January 24-31, 2027") == "unverified"
    finally:
        k.fetch_text = orig


def test_verify_evidence_unchecked_without_source():
    assert k.verify_evidence("", "anything") == "unchecked"
    assert k.verify_evidence("not-a-url", "anything") == "unchecked"


def test_verify_evidence_unchecked_on_fetch_error():
    def boom(url):
        raise RuntimeError("timeout")

    orig = _with_fetch(boom)
    try:
        assert k.verify_evidence("https://x.com", "runs January 24-31, 2027") == "unchecked"
    finally:
        k.fetch_text = orig


if __name__ == "__main__":
    # Standalone runner (no pytest needed): run every test_* and report.
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
