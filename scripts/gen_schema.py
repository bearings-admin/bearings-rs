#!/usr/bin/env python3
"""Regenerate supabase/schema.sql from the live database catalog.

Pulls the assembled DDL from public._schema_dump over PostgREST (service key from
/opt/bearings-rs/.env) and writes a current, ordered schema file. SQL never passes
through anything but this script. The DB is the only complete source of truth: 25 of
the 39 tables were created outside the migration tracker, so this catalog dump — not
the migration history — is what reproduces the real schema.
"""
import os
import json
import datetime
import urllib.request

SECTIONS = {
    0: "Sequences",
    1: "Types (enums)",
    2: "Tables (columns + PK/unique/check)",
    3: "Foreign keys (added after all tables exist)",
    4: "Indexes (non-constraint)",
    5: "Views",
    6: "Functions",
    7: "Row-Level Security (enable)",
    8: "Policies",
}


def load_env(p="/opt/bearings-rs/.env"):
    d = {}
    for line in open(p):
        line = line.strip()
        if line and "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            d[k.strip()] = v.strip()
    return d


e = load_env()
url, key = e["SUPABASE_URL"], e["SUPABASE_SERVICE_ROLE_KEY"]
req = urllib.request.Request(
    f"{url}/rest/v1/_schema_dump?select=seq,kind,name,ddl&order=seq,name",
    headers={"apikey": key, "Authorization": f"Bearer {key}", "Accept": "application/json"},
)
rows = json.load(urllib.request.urlopen(req, timeout=30))
assert rows, "no rows from _schema_dump (PostgREST schema cache not reloaded?)"

bar = "-- " + "=" * 72 + "\n"
out = [
    "-- Bearings — database schema (GENERATED from the live catalog; do not hand-edit)\n",
    f"-- Generated {datetime.date.today().isoformat()}. This is the SCHEMA only (no row data).\n",
    "--\n",
    "-- Why catalog-generated, not migrations: 25 of the 39 tables (events, places,\n",
    "-- clubs, competitions, title_holders, campaigns, ...) were created outside the\n",
    "-- Supabase migration tracker (dashboard / raw SQL), so the migration history is\n",
    "-- NOT a complete source of truth. The live catalog is. Regenerate with\n",
    "-- scripts/gen_schema.py (see supabase/README.md).\n",
    "--\n",
    "-- Caveats: catalog-derived, not `pg_dump`. Sequences are emitted bare (no exact\n",
    "-- start/owned-by); for a byte-exact restore use `supabase db dump`. Good enough to\n",
    "-- recreate structure and to review the data model.\n\n",
]
last_seq = None
for r in rows:
    if r["seq"] != last_seq:
        out.append("\n" + bar)
        out.append(f"-- {SECTIONS.get(r['seq'], r['kind'])}\n")
        out.append(bar)
        last_seq = r["seq"]
    out.append(r["ddl"].rstrip() + "\n\n")

with open("supabase/schema.sql", "w", encoding="utf-8", newline="\n") as f:
    f.write("".join(out))
print(f"WROTE supabase/schema.sql — {len(rows)} objects, {os.path.getsize('supabase/schema.sql')} bytes")
