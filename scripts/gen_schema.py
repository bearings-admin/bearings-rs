#!/usr/bin/env python3
"""Regenerate — or --check — supabase/schema.sql from the live database.

Calls the public.schema_dump() RPC over PostgREST (Supabase keys from .env) and
assembles the current public schema. The live catalog is the source of truth (25
of 39 tables predate the migration tracker), so this file — not the migrations —
reproduces the schema.

    python3 scripts/gen_schema.py            # rewrite supabase/schema.sql
    python3 scripts/gen_schema.py --check    # exit 1 (with a diff) if it drifted

No DB password needed (that's why this exists): the RPC runs the catalog
introspection server-side and returns DDL rows. Output is deterministic (sorted,
no timestamp) so --check is a clean comparison.
"""
import os
import sys
import json
import urllib.request
import difflib

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
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCHEMA_PATH = os.path.join(ROOT, "supabase", "schema.sql")


def load_env(p="/opt/bearings-rs/.env"):
    if os.path.exists(p):
        for line in open(p):
            line = line.strip()
            if line and "=" in line and not line.startswith("#"):
                k, v = line.split("=", 1)
                os.environ.setdefault(k.strip(), v.strip())


def fetch_rows():
    load_env()
    url = os.environ["SUPABASE_URL"]
    key = os.environ["SUPABASE_SERVICE_ROLE_KEY"]
    req = urllib.request.Request(
        f"{url}/rest/v1/rpc/schema_dump",
        data=b"{}",
        method="POST",
        headers={
            "apikey": key,
            "Authorization": f"Bearer {key}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        },
    )
    rows = json.load(urllib.request.urlopen(req, timeout=30))
    assert rows, "schema_dump() returned no rows"
    rows.sort(key=lambda r: (r["seq"], r["name"]))
    return rows


def render(rows):
    bar = "-- " + "=" * 72 + "\n"
    out = [
        "-- Bearings — database schema (GENERATED from the live catalog; do not hand-edit)\n",
        "-- Regenerate / check: scripts/gen_schema.py [--check]  (see supabase/README.md)\n",
        "-- The live catalog is the source of truth — 25 of 39 tables predate the migration\n",
        "-- tracker, so this generated file (not the migrations) reproduces the schema.\n",
        "-- Catalog-derived, not pg_dump (sequences bare); `supabase db dump` for exact restore.\n",
    ]
    last = None
    for r in rows:
        if r["seq"] != last:
            out += ["\n", bar, f"-- {SECTIONS.get(r['seq'], r['kind'])}\n", bar]
            last = r["seq"]
        out.append(r["ddl"].rstrip() + "\n\n")
    return "".join(out)


def main():
    content = render(fetch_rows())
    if "--check" in sys.argv:
        current = ""
        if os.path.exists(SCHEMA_PATH):
            current = open(SCHEMA_PATH, encoding="utf-8").read()
        if current == content:
            print("OK — supabase/schema.sql matches the live database.")
            return 0
        sys.stdout.writelines(
            difflib.unified_diff(
                current.splitlines(True),
                content.splitlines(True),
                "supabase/schema.sql (committed)",
                "live database",
                n=1,
            )
        )
        print(
            "\nDRIFT: supabase/schema.sql is out of date. "
            "Run `python3 scripts/gen_schema.py` and commit.",
            file=sys.stderr,
        )
        return 1
    with open(SCHEMA_PATH, "w", encoding="utf-8", newline="\n") as f:
        f.write(content)
    print(f"WROTE {SCHEMA_PATH} — {content.count('CREATE TABLE ')} tables, {len(content)} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
