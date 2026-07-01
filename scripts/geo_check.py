#!/usr/bin/env python3
"""Bearings geo-audit — flags places with missing, placeholder, or wrong coordinates.

Two tiers, both propose-never-mutate (emails the steward via the digest transport + logs):
  Tier 1 (no network): null coords; low-precision placeholder coords (hand-guessed, e.g.
    46.0/-122.0); non-cruise 0/0; coords outside the stated country's bounding box.
  Tier 2 (Nominatim / OpenStreetMap geocode): geocode "city, country" and flag places whose
    stored coords are >GEO_MAX_KM from the geocoded point — catches mislocations like a
    resort tagged 'Guerneville' that's really in West Virginia. Respects the OSM usage
    policy: <=1 req/s + identifying User-Agent. Disable with GEO_GEOCODE=0.

Motivated by the link-fix passes, which exposed a second error class in early
source='manual' places: wrong province/town + placeholder coords (Shadow Falls ON→BC,
Roseland CA→WV, Triangle Rec Guerneville→Granite Falls WA).

Run:      python3 scripts/geo_check.py
Schedule: bearings-geocheck.timer (monthly).
Tune:     GEO_GEOCODE (1/0), GEO_MAX_KM (25), GEO_SLEEP (1.1s), GEO_LIMIT (0 = all).
"""
import json
import math
import os
import time
from urllib.error import HTTPError, URLError
from urllib.parse import quote
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
UA = "Bearings-GeoAudit/1.0 (+https://bearings.community; ursasteward@pm.me)"
GEOCODE = os.environ.get("GEO_GEOCODE", "1").strip().lower() not in ("0", "false", "no", "")
MAX_KM = float(os.environ.get("GEO_MAX_KM", "25"))
GEO_SLEEP = float(os.environ.get("GEO_SLEEP", "1.1"))  # OSM policy: <= 1 req/s
GEO_LIMIT = int(os.environ.get("GEO_LIMIT", "0"))       # cap Tier-2 geocodes (0 = all)

# Generous (lat_min, lat_max, lng_min, lng_max) boxes for the countries we hold the most
# places in — a cheap Tier-1 wrong-country catch. Countries not listed are skipped (no
# false positives); Tier 2 covers the rest.
COUNTRY_BBOX = {
    "USA": (24.0, 49.5, -125.0, -66.5), "Canada": (41.5, 70.0, -141.0, -52.0),
    "UK": (49.5, 61.0, -8.7, 2.0), "Germany": (47.2, 55.1, 5.8, 15.1),
    "France": (41.2, 51.2, -5.2, 9.6), "Spain": (35.9, 43.9, -9.4, 4.4),
    "Netherlands": (50.7, 53.6, 3.3, 7.3), "Belgium": (49.4, 51.6, 2.5, 6.5),
    "Italy": (36.6, 47.1, 6.6, 18.6), "Australia": (-44.0, -10.0, 112.0, 154.0),
    "Brazil": (-34.0, 5.3, -74.0, -34.8), "Thailand": (5.6, 20.5, 97.3, 105.7),
    "Japan": (24.0, 45.6, 122.9, 146.0), "Portugal": (36.9, 42.2, -9.6, -6.1),
    "Mexico": (14.5, 32.8, -118.5, -86.7),
}
CRUISE_WORDS = ("At Sea", "Rhine", "Mediterranean")


def supa_get(path):
    req = Request(
        f"{SUPABASE_URL}/rest/v1/{path}",
        headers={"apikey": SUPABASE_KEY, "Authorization": f"Bearer {SUPABASE_KEY}",
                 "Accept": "application/json"},
    )
    with urlopen(req, timeout=20) as r:
        return json.loads(r.read())


def haversine_km(a_lat, a_lng, b_lat, b_lng):
    R = 6371.0
    p1, p2 = math.radians(a_lat), math.radians(b_lat)
    dphi = math.radians(b_lat - a_lat)
    dlmb = math.radians(b_lng - a_lng)
    h = math.sin(dphi / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dlmb / 2) ** 2
    return 2 * R * math.asin(math.sqrt(h))


def is_placeholder(lat, lng):
    """True if coords look hand-guessed: <=1 decimal of precision on both axes."""
    return round(lat, 1) == lat and round(lng, 1) == lng


def out_of_country(country, lat, lng):
    box = COUNTRY_BBOX.get(country)
    if not box:
        return False
    la0, la1, lo0, lo1 = box
    return not (la0 <= lat <= la1 and lo0 <= lng <= lo1)


def is_cruise(place):
    city = place.get("city") or ""
    return (place.get("place_type") == "cruise-ship"
            or (place.get("country") or "") in ("International", "Global")
            or any(w in city for w in CRUISE_WORDS))


def geocode(city, country):
    """Nominatim/OSM lookup of 'city, country' -> (lat, lng) or None."""
    q = ", ".join([p for p in [city, country] if p])
    url = f"https://nominatim.openstreetmap.org/search?format=jsonv2&limit=1&q={quote(q)}"
    try:
        with urlopen(Request(url, headers={"User-Agent": UA}), timeout=20) as r:
            data = json.loads(r.read())
        return (float(data[0]["lat"]), float(data[0]["lon"])) if data else None
    except (HTTPError, URLError, ValueError, KeyError, IndexError, OSError):
        return None


def run():
    rows = supa_get("places?select=id,name,city,country,address,lat,lng,place_type&active=eq.true")
    missing, placeholder, wrong_country, mislocated = [], [], [], []

    for p in rows:
        lat, lng = p.get("lat"), p.get("lng")
        tag = f"places#{p['id']} {p.get('name', '')} ({p.get('city') or '?'}, {p.get('country') or '?'})"
        if lat is None or lng is None:
            missing.append(tag)
            continue
        lat, lng = float(lat), float(lng)
        if lat == 0 and lng == 0:
            if not is_cruise(p):
                placeholder.append(f"{tag} — 0/0")
            continue
        if is_cruise(p):
            continue
        if is_placeholder(lat, lng):
            placeholder.append(f"{tag} — {lat}/{lng}")
        if out_of_country(p.get("country") or "", lat, lng):
            wrong_country.append(f"{tag} — coords {lat}/{lng} outside stated country")

    geocoded = 0
    if GEOCODE:
        for p in rows:
            if GEO_LIMIT and geocoded >= GEO_LIMIT:
                break
            lat, lng = p.get("lat"), p.get("lng")
            city = p.get("city") or ""
            if lat is None or lng is None or is_cruise(p) or not city:
                continue
            lat, lng = float(lat), float(lng)
            if lat == 0 and lng == 0:
                continue
            g = geocode(city, p.get("country") or "")
            geocoded += 1
            time.sleep(GEO_SLEEP)
            if not g:
                continue
            d = haversine_km(lat, lng, g[0], g[1])
            if d > MAX_KM:
                mislocated.append(
                    f"places#{p['id']} {p.get('name', '')} — stored {lat:.3f}/{lng:.3f} is "
                    f"{d:.0f} km from '{city}, {p.get('country') or ''}' ({g[0]:.3f}/{g[1]:.3f})"
                )

    flagged = len(placeholder) + len(wrong_country) + len(mislocated)
    L = [f"Bearings geo-audit — {len(rows)} active places; "
         f"{flagged} coord issue(s), {len(missing)} missing coords.", ""]
    for label, items in (("MISLOCATED (coords far from stated city)", mislocated),
                         ("WRONG COUNTRY (coords outside country box)", wrong_country),
                         ("PLACEHOLDER / GUESSED COORDS", placeholder),
                         ("MISSING COORDS", missing)):
        if items:
            L += [f"{label} ({len(items)}):"] + [f"  - {e}" for e in items] + [""]
    if not (flagged or missing):
        L.append("All coordinates look sane. ✓")
    if GEOCODE:
        L += ["", f"(Tier 2 geocoded {geocoded} places via Nominatim/OpenStreetMap "
                  "— © OpenStreetMap contributors.)"]
    body = "\n".join(L)
    print(body)

    try:
        os.makedirs("/opt/bearings-rs/logs", exist_ok=True)
        with open("/opt/bearings-rs/logs/geocheck-latest.txt", "w", encoding="utf-8") as f:
            f.write(body + "\n")
    except Exception:
        pass

    if flagged or missing:
        try:
            from digest import send_digest
            send_digest({"subject": f"[Bearings] geo-audit: {flagged} coord issue(s), "
                                    f"{len(missing)} missing", "body": body})
        except Exception as e:
            print(f"[geocheck] email skipped: {e}")


if __name__ == "__main__":
    run()
