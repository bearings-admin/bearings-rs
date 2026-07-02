#!/usr/bin/env python3
"""Bearings content translator — pre-warms the `content_translations` cache.

Layer 2 of the i18n plan: the static chrome is baked into i18n.rs; this handles the
DYNAMIC DB prose (event/place/campaign descriptions) that can't be baked. It translates
each distinct source string into the five non-English site languages via the Anthropic
API and stores it in `content_translations` (keyed by md5(source_text)+lang). The backend
reads that cache at render time (in-memory, fast) and falls back to English on a miss, so
page loads never call the API — this job fills the cache ahead of them.

Idempotent: skips (src_md5, lang) pairs already cached. Batches strings per API call.
Zero deps (urllib), same pattern as keeper.py / feed_reader.py.

Run:      python3 scripts/translate.py            (all configured fields, all 5 langs)
Schedule: bearings-translate.timer (nightly, after the feed reader adds content).
Tune:     TRANSLATE_MODEL (default claude-haiku-4-5), TRANSLATE_BATCH (12), TRANSLATE_LIMIT.
"""
import hashlib
import json
import os
import re
import socket
from urllib.error import HTTPError
from urllib.request import Request, urlopen

# Hard floor so a stalled connection can't hang the job (a manual run once sat ~3h when a
# urlopen timeout didn't fire). Every socket op now aborts after 90s.
socket.setdefaulttimeout(90)


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
ANTHROPIC_KEY = os.environ["ANTHROPIC_API_KEY"]
MODEL = os.environ.get("TRANSLATE_MODEL", "claude-haiku-4-5")
BATCH = int(os.environ.get("TRANSLATE_BATCH", "12"))
LIMIT = int(os.environ.get("TRANSLATE_LIMIT", "0"))  # 0 = no cap

LANGS = {"de": "German", "es": "Spanish", "fr": "French", "pt": "Portuguese", "th": "Thai"}
# (table, prose field) — DB content clearly worth translating. Proper-noun name/city
# fields are intentionally excluded (brands/place names stay as-is).
FIELDS = [
    ("events", "description"),
    ("places", "description"),
    ("campaigns", "description"),
    ("clubs", "description"),
    ("bear_history", "description"),
    ("bear_history", "significance"),
    ("digital_spaces", "description"),
    ("stores", "description"),
    ("creators", "bio"),
    ("title_holders", "bio"),
]


def md5(s):
    return hashlib.md5(s.encode("utf-8")).hexdigest()


def supa_get(path):
    req = Request(f"{SUPABASE_URL}/rest/v1/{path}",
                  headers={"apikey": SUPABASE_KEY, "Authorization": f"Bearer {SUPABASE_KEY}",
                           "Accept": "application/json"})
    with urlopen(req, timeout=30) as r:
        return json.loads(r.read())


def supa_post(rows):
    body = json.dumps(rows).encode()
    req = Request(f"{SUPABASE_URL}/rest/v1/content_translations", data=body, method="POST",
                  headers={"apikey": SUPABASE_KEY, "Authorization": f"Bearer {SUPABASE_KEY}",
                           "Content-Type": "application/json",
                           "Prefer": "resolution=ignore-duplicates,return=minimal"})
    try:
        with urlopen(req, timeout=30) as r:
            return r.status
    except HTTPError as e:
        return e.code


def claude_translate(strings, lang_name):
    """Translate a list of strings into lang_name; return a same-length list."""
    payload = {
        "model": MODEL, "max_tokens": 4000,
        "messages": [{"role": "user", "content": (
            f"Translate each string in this JSON array into {lang_name}. Return ONLY a JSON "
            f"array of the translations, same length and order — no prose, no keys. Keep it "
            f"natural and concise. Preserve any HTML tags, '→' arrows, emoji, URLs and "
            f"bracketed notes. Do not translate proper nouns (event, venue, city, brand "
            f"names). Strings:\n{json.dumps(strings, ensure_ascii=False)}"
        )}],
    }
    req = Request("https://api.anthropic.com/v1/messages", data=json.dumps(payload).encode(),
                  headers={"x-api-key": ANTHROPIC_KEY, "anthropic-version": "2023-06-01",
                           "content-type": "application/json"})
    with urlopen(req, timeout=120) as r:
        data = json.loads(r.read())
    text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
    m = re.search(r"\[.*\]", text, re.S)
    try:
        out = json.loads(m.group(0)) if m else []
    except Exception:
        out = []
    return out if isinstance(out, list) and len(out) == len(strings) else None


def collect_sources():
    """Distinct non-empty prose strings across the configured fields."""
    seen = {}
    for table, field in FIELDS:
        try:
            rows = supa_get(f"{table}?select={field}&{field}=not.is.null&limit=5000")
        except Exception as e:
            print(f"  ! {table}.{field}: fetch failed — {e}")
            continue
        for row in rows:
            s = (row.get(field) or "").strip()
            if len(s) >= 3:
                seen[s] = md5(s)
    return seen  # {source_text: md5}


def run():
    sources = collect_sources()
    print(f"[translate] {len(sources)} distinct strings across {len(FIELDS)} fields "
          f"→ {len(LANGS)} langs (model={MODEL})\n")
    total = 0
    for lang, lang_name in LANGS.items():
        existing = {r["src_md5"] for r in supa_get(
            f"content_translations?select=src_md5&target_lang=eq.{lang}&limit=100000")}
        todo = [(s, h) for s, h in sources.items() if h not in existing]
        if LIMIT:
            todo = todo[:LIMIT]
        print(f"  {lang}: {len(todo)} to translate ({len(existing)} cached)")
        for i in range(0, len(todo), BATCH):
            chunk = todo[i:i + BATCH]
            strings = [s for s, _ in chunk]
            try:
                translated = claude_translate(strings, lang_name)
            except Exception as e:
                print(f"    ! batch {i}: {e}")
                continue
            if not translated:
                print(f"    ! batch {i}: bad/again model output — skipped")
                continue
            rows = [{"src_md5": h, "target_lang": lang, "source_text": s,
                     "translated_text": t, "model": MODEL}
                    for (s, h), t in zip(chunk, translated)]
            supa_post(rows)
            total += len(rows)
            print(f"    + {lang}: cached {i + len(chunk)}/{len(todo)}")
    print(f"\n[translate] cached {total} translation(s) into content_translations.")


if __name__ == "__main__":
    run()
