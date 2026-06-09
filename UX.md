
# bearings-rs — UX Design Intent

**Last updated:** 2026-06-06
**Purpose:** Captures the UX philosophy before it becomes routes and queries.
Read this before building any frontend or designing any API response shape.

---

## The Core Problem With the Lovable Iteration

The Lovable site improved visually but went static. Long lists of everything
replaced the dynamic, contextual experience. A bear in Ottawa in November
does not need a list of 130 events — they need the three things happening
near them this season, and a way to plan the two big trips they're
considering next year.

The fix is not fewer pages. It is a temporal and spatial lens applied
consistently across all content.

---

## Two Lenses: When and Where

Every piece of content in Bearings has a natural relationship to:

**WHEN** — relative to now
- Happening today / this week → highest priority
- Happening this season (next 90 days) → planning horizon
- Happening this year → trip planning
- Past → archive, memory, context
- Future beyond a year → early awareness only

**WHERE** — relative to the bear
- Here (their city or current location) → highest relevance
- Nearby (within a drive/flight) → strong relevance
- Destination (a specific city they're considering) → trip context
- Global → lowest priority unless specifically requested

These two lenses combine. The ideal default view is:
**"What is happening near me, soon?"**

---

## The Four Zones — Reimagined With Lenses

### NOW
**Lens: Here + Today/This Week**

Not a static snapshot. A live answer to: what is the bear world doing
right now, relative to where I am?

Content:
- Hot events (hot=true) within the next 30 days, sorted by proximity
- Bars and venues near current location with a bear night tonight/this week
- Active campaigns — always visible, location-neutral
- Current title holders — always visible, global

The bear flag timeline bar stays — it gives the annual rhythm at a glance.
But the default event list starts with proximity, not alphabetical/date order.

API shape this needs:
```
GET /api/events?upcoming_only=true&lat=45.4&lng=-75.7&radius_km=500&limit=10
GET /api/places/nearby?lat=45.4&lng=-75.7&radius_km=50&bear_popular=true
```

### COMING UP
**Lens: Adjustable WHERE + Adjustable WHEN**

This is the trip planner. The bear sets two things:
- **Here**: their home city, OR a destination city for a trip
- **When**: a season or date range (default: next 6 months)

The result: "What are the bear events in Europe in September?"
or: "What is happening in Berlin in October?"

Filters available:
- Event type (bear run, cruise, leather, social)
- Size (local → major)
- Continent / country / city
- Month or season
- Distance from a point

The iCal export applies to whatever the current filter state is.
A bear building their October Germany trip subscribes to:
`/api/events/ical.ics?country=Germany&month=October`
and it auto-refreshes in their calendar.

This is where "long lists" become useful — the bear is in planning mode
and wants to see everything available in a given window.

### BEAR ARCHIVES
**Lens: Past + Global (no spatial filter)**

History doesn't need a location filter. The IBR 1992 winners are
relevant regardless of where you are. The timeline spine from 1987
is the permanent backbone.

Content that makes sense here:
- Title holder lineage by competition
- Club histories (founded year, milestones)
- Community milestones timeline
- Oral histories / stories
- Closed venues (preserved as memory, not removed)

The archives should feel like a library, not a list. Slow, deliberate,
deep. Not filtered by proximity.

### BEAR FUTURE
**Lens: Global + Forward**

Governance doesn't need a location filter either. NORTH token proposals
are community-wide. The treasury is global. The only time location matters
here is for proposal beneficiaries (e.g. "senior bear programme in Ottawa").

---

## The Trip Planner Concept (COMING UP expansion)

Within the COMING UP zone, a "plan a trip" mode:

**Input:**
1. Destination city or region (or: "I'm open, show me where things are happening")
2. Date range or season
3. What I want: events only / venues only / both

**Output:**
- Events in that window at that location
- Nearby venues (bars, saunas, campgrounds) within the destination
- Nearby clubs that could be contacted for local knowledge
- iCal export of the filtered events

This is entirely achievable with existing API routes:
```
# Events in Berlin, September
GET /api/events?country=Germany&month=September

# Venues near Berlin city centre
GET /api/places/nearby?lat=52.52&lng=13.40&radius_km=30

# Clubs in Germany
GET /api/clubs?country=Germany
```

The frontend combines these three calls into a single "trip view".
No new backend work needed — just a composed frontend experience.

**The "here" toggle:**
Default: user's detected location (from browser geolocation API or IP).
Override: bear types a city name → frontend geocodes it → uses that lat/lng.
Stored in user_preferences (session-based, no login needed).

**The "now" toggle:**
Default: current season (next 90 days).
Override: bear picks a season/month from a selector.
This is just a query param — no session storage needed.

---

## What This Means for the Rust API

The existing routes support this model. A few additions would complete it:

### 1. GET /api/events — add proximity sorting

Currently sorted by `start_date.asc`. When `lat` and `lng` are provided,
sort by distance first, then date. Use the Haversine distance calculated
in PostgREST or an RPC.

```
GET /api/events?lat=45.4&lng=-75.7&radius_km=500&upcoming_only=true
```

The `places_nearby` RPC pattern already exists — apply the same to events.

### 2. GET /api/now — composite "here and now" endpoint

A single endpoint that returns the full NOW zone payload in one call:
```json
{
  "hot_events": [...],       // hot=true, upcoming, near lat/lng
  "nearby_venues": [...],    // bear_popular, within radius
  "active_campaigns": [...], // always global
  "current_titles": [...]    // always global
}
```

Reduces the frontend from 4 calls to 1. Lovable makes multiple sequential
calls which is why it felt slow and static — each section loaded independently.

### 3. GET /api/coming-up — trip planner endpoint

```
GET /api/coming-up?lat=52.52&lng=13.40&radius_km=200&season=autumn&limit=20
```

Returns:
```json
{
  "events": [...],    // filtered by location + season
  "venues": [...],    // filtered by proximity
  "clubs": [...]      // filtered by country of destination
}
```

Season mapping (server-side):
- spring: March–May
- summer: June–August
- autumn: September–November
- winter: December–February

### 4. User preferences for location

No login needed. Session-based:
```
POST /api/preferences
{ "session_id": "...", "home_lat": 45.4, "home_lng": -75.7 }

GET /api/events?use_home_location=true
```

The user_preferences table already exists with `session_id`.

---

## What This Means for the Lovable Frontend

When rebuilding pages with Lovable prompts, each zone prompt should specify:

**NOW prompt additions:**
- "Default event list sorted by proximity to detected location"
- "Timeline bar shows event density, clicking a month filters the event list"
- "Hot badge on events where hot=true"
- "Show inclusion_flag_codes as small coloured badges, never hide the listing"

**COMING UP prompt additions:**
- "Two controls at top: HERE (location selector) and WHEN (season/month picker)"
- "Results update dynamically as controls change"
- "iCal subscribe button exports current filter state"
- "Trip view: when a destination city is set, show nearby venues below events"

**BEAR ARCHIVES prompt:**
- "Timeline spine is the primary navigation — decade markers, click to jump"
- "No location filter — archives are always global"
- "Closed venues visible but visually dimmed with 'closed YYYY' label"

---

## Explicit Non-Goals

- No dating app features
- No social feed
- No user-generated content beyond submissions
- No chat or messaging
- No map as the primary UI (maps are supplementary, not primary)
- No recommendation engine (proximity + recency is enough)
- No notifications (iCal subscription handles event updates)

The platform should feel like a very good library, not an app.
Calm, navigable, trustworthy. The bear comes to find something real.
