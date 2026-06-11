# Bearings Research Directive
## Automated Data Collection — Safe Methods & Protocols

_This document governs how Bearings agents and crons collect event, title holder,
and community data. It distinguishes safe automated methods from slow/manual ones,
and records what cannot yet be automated._

---

## Tier 1 — Fully Automated (Nightly Cron)

These run every night at 02:00 UTC via `bearings-feeds.timer` →
`/opt/bearings-rs/scripts/feed_reader.py`.

All results land in `candidate_events` with `status = 'pending'` for steward review
at `/?zone=admin&token=<ADMIN_TOKEN>`.

### Active feeds

| Feed | Type | URL | Scope | Notes |
|---|---|---|---|---|
| Bear World Magazine | `rss` | https://bearworldmag.com/feed/ | Global | Keyword-filtered |
| The Bear Calendar | `ical` | https://thebearcalendar.com/events/list/?ical=1 | Global | All events pass |
| Amsterdam Bear Pride | `ical` | https://amsterdambearpride.com/en/events/?ical=1 | Netherlands | All events pass |
| Bear Naked Club Chicago | `ical` | https://bearnaked.club/events/list/?ical=1 | Chicago | All events pass |
| SF Eagle | `ical` | https://sf-eagle.com/events/list/?ical=1 | San Francisco | All events pass |
| SF Leather & LGBTQ District | `ical` | https://sfleatherdistrict.org/events/month/?ical=1 | San Francisco | Mixed LGBTQ |
| London Leathermen | `ical` | (Google Calendar public .ics) | London | All events pass |
| XL Bears Seattle | `ical` | (Google Calendar public .ics) | Seattle | All events pass |
| Bear Carnival Gran Canaria | `ical-static` | (annual .ics file) | Gran Canaria | Annual URL refresh needed |

### Feed reader behaviour
- ETag / Last-Modified conditional requests — feeds that haven't changed return 304
  and cost nothing to process
- Duplicate URLs silently skipped (`source_url` UNIQUE constraint)
- `parsed_start` / `parsed_end` extracted from iCal DTSTART/DTEND
- Country hinted from title + description + location text
- `feed_reader.py` must be run with env vars from `/opt/bearings-rs/.env`

### Annual maintenance — `ical-static` feeds
Each January, an agent should:
1. Query `SELECT id, org_name, url, notes FROM watched_feeds WHERE feed_type = 'ical-static'`
2. For each, visit the organisation's website and find the new year's programme file URL
3. PATCH the record with the new URL and reset `last_etag`, `last_modified` to NULL
4. Confirm by running `feed_reader.py` manually

---

## Tier 2 — Semi-Automated (Monthly or Quarterly)

These sources are public and parseable but require more careful handling.
**Not yet wired to the cron.** Implement only after adding robots.txt check to the
scraper and rate-limiting to ≥5s between requests.

### Candidate scrape sources

| Source | URL | Frequency | Why careful |
|---|---|---|---|
| Bear Events EU | https://bearevents.eu/ | Monthly | Static site, check robots.txt |
| Gay Travel 4 U Bear Events | https://www.gaytravel4u.com/bear-events-not-to-miss/ | Quarterly | Large list page, JS-rendered |
| Folsom Street Events | https://www.folsomstreet.org/fs-events-calendar | On-demand | Per-event .ics links only, no feed |

### Rules for Tier 2 scraping
1. Always fetch `/robots.txt` first. If `Disallow: /` or `Crawl-delay` is set, respect it.
2. Set `User-Agent: Bearings-FeedReader/1.0 (+https://bearings.community)`
3. Minimum 5 seconds between page requests
4. Do not re-scrape within the declared frequency window
5. Store `scraped_at` timestamp; back off for 30 days on HTTP 429 / 503

### Implementation path
Extend `feed_reader.py` with a `scrape_html()` function that:
- Fetches the page with the standard UA
- Extracts `<article>` / `<li class="event">` blocks using regex (no external deps)
- Feeds items through the same keyword filter + country guesser
- Inserts into `candidate_events` as normal

---

## Tier 3 — Manual / Agent-Assisted (As Needed)

These cannot be automated due to auth walls, no public API, or ethical constraints.
A human steward or a prompted agent should check them on the cadence shown.

### Title holder research

| Source | Cadence | Method |
|---|---|---|
| Mr Bear International / Thailand | Post-contest (check mrbearthailand.com) | Agent web search |
| Mr Bear Europe (Lisbon, July) | After July 19 2026 | Agent: check euro.ursos.pt |
| TBRU winners | Post-event | Agent: check dallasbears.org Facebook or email contest@tbru.org |
| NAB Bear 2026 | Verify Cliff Boyd re-elected | Agent web search |
| Mr Bear UK 2026 | Applications open, contest TBD | Agent: check ukbear.co.uk |
| IML (International Mr Leather) | Post-Memorial Day weekend | Agent web search |
| Bear Pride city winners | Post-event | Agent web search per city |

### Event discovery — permanent manual gaps

| Region | Gap | Reason |
|---|---|---|
| Brazil | Bear events (São Paulo, Rio) | No English RSS/iCal; Portuguese FB groups only |
| Japan | Bear events (Tokyo, Osaka) | Japanese-language sites, no feeds |
| South Korea | Seoul bear events | Korean-language only |
| Eastern Europe | Warsaw, Prague bear events | Sporadic, FB-only |
| Middle East | Underground events | Safety-critical — do not automate discovery |

For Brazil and Japan especially: the steward should periodically reach out directly to
known community contacts rather than trying to scrape.

### Eventbrite organiser feeds (third-party proxy)
URL pattern: `https://eb-to-ical.daylightpirates.org/eventbrite-organizer-ical?organizer={ID}`

This is a third-party proxy and may go offline. Use on-demand only, not in the nightly cron.
Known organiser IDs to check manually post-event for title results and future dates:
- Lazy Bear Week (find organiser ID on lazybearweekend.com Eventbrite link)
- Tidal Wave Party
- TBRU (check yearly)

---

## Tier 4 — Dead Ends (Do Not Pursue)

| Method | Status |
|---|---|
| Eventbrite search API | Dead since 2020 |
| Facebook page iCal | Removed — user-private only |
| Meetup group iCal | Broken for unauthenticated access |
| Leather Archives LibCal | Subscribe URL contains session token — not publicly subscribable |
| Recon.com events | No public feed |
| Bearracuda | No public feed; mailing list only |
| Grindr / Scruff events | App-only, no public calendar |

---

## Cron Schedule Summary

| Job | Schedule | What it does |
|---|---|---|
| `bearings-feeds.timer` | Nightly 02:00 UTC | RSS + iCal feeds → candidate_events |
| Annual iCal-static refresh | Every January (agent task) | Update ical-static URLs for new year |
| Tier 2 scrape (future) | Monthly/quarterly | bearevents.eu, gaytravel4u — not yet live |
| Title holder check (agent) | Post-contest | Search + PATCH title_holders via Supabase REST |

---

## Adding a New Feed

1. Verify the URL returns valid RSS/iCal (test with curl)
2. Check `robots.txt` if it's a website rather than a dedicated feed endpoint
3. INSERT into `watched_feeds`:
   ```sql
   INSERT INTO watched_feeds (url, feed_type, org_name, description, active)
   VALUES ('https://...', 'ical', 'Org Name', 'Description', true);
   ```
4. Run `feed_reader.py` manually to confirm it parses
5. If events look wrong, adjust `COUNTRY_MAP` or `EVENT_KW` in `feed_reader.py`
