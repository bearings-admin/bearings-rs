#!/usr/bin/env python3
"""Tests for geo_check.py pure logic — distance, placeholder detection, country-box, cruise
detection, and geocode parsing (stubbed). No live network.

Runs under pytest (CI: `pytest scripts/`) or standalone (`python3 test_geo_check.py`).
"""
import json
import os
import sys

os.environ.setdefault("SUPABASE_URL", "http://localhost")
os.environ.setdefault("SUPABASE_SERVICE_ROLE_KEY", "test")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import geo_check as gc  # noqa: E402


def test_haversine_known_distance():
    # one degree of longitude at the equator is ~111 km
    assert abs(gc.haversine_km(0.0, 0.0, 0.0, 1.0) - 111.19) < 1.0
    assert gc.haversine_km(48.0826, -121.9682, 48.0826, -121.9682) == 0.0


def test_is_placeholder():
    assert gc.is_placeholder(46.0, -122.0) is True
    assert gc.is_placeholder(45.5, -122.5) is True
    assert gc.is_placeholder(48.0826, -121.9682) is False  # real geocoded precision


def test_out_of_country():
    assert gc.out_of_country("USA", 45.0, -100.0) is False        # inside
    assert gc.out_of_country("USA", 48.0, 10.0) is True           # those are German coords
    assert gc.out_of_country("Neverland", 0.0, 0.0) is False      # unmapped -> skip


def test_is_cruise():
    assert gc.is_cruise({"place_type": "cruise-ship", "city": "X", "country": "USA"})
    assert gc.is_cruise({"place_type": "bar", "city": "At Sea", "country": "International"})
    assert not gc.is_cruise({"place_type": "bar", "city": "Osaka", "country": "Japan"})


class _Resp:
    def __init__(self, payload):
        self._b = json.dumps(payload).encode()

    def read(self):
        return self._b

    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False


def _with_urlopen(payload, fn):
    orig = gc.urlopen
    gc.urlopen = lambda req, timeout=0: _Resp(payload)
    try:
        return fn()
    finally:
        gc.urlopen = orig


def test_geocode_parses_result():
    got = _with_urlopen([{"lat": "38.502", "lon": "-123.000"}],
                        lambda: gc.geocode("Guerneville", "USA"))
    assert got == (38.502, -123.0)


def test_geocode_empty_is_none():
    assert _with_urlopen([], lambda: gc.geocode("Nowhere", "USA")) is None


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
