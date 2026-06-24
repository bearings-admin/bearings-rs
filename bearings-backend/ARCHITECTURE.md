# bearings-backend — architecture & decisions

A short, honest tour of how the backend is built and *why*, with measurements.
Aimed at a reviewer deciding whether the design choices were deliberate.

## Shape

```
HTTP ─▶ routes/         thin: parse request, map result → HTTP
        services/       business-logic seam over repo traits (currently minimal)
        repositories/   data access; one trait + one Supabase impl per resource
        db.rs           SupabaseClient: get_json / post_rpc / write_json (+ TTL cache)
        ssr/ + ui.rs    server-rendered HTML zones, HTMX-enhanced
                 │
                 ▼
        Supabase PostgREST  (PostgreSQL behind a REST API)
```

- **Library + thin binary.** `lib.rs` owns the app and exposes `build_app(db) -> Router`; `main.rs` only wires env → config → serve. Integration tests in `tests/` import the library and drive a `TestServer` with no socket.
- **Trait-based repositories (DIP).** Read handlers depend on repository *traits* (e.g. `EventRepository`), not on `SupabaseClient`, so they can be unit-tested against fakes. (Write handlers — submissions, upvote, revival — still call the client directly; routing them through the repo layer too is a known cleanup.)
- **One query chokepoint.** Every read goes through `SupabaseClient::get_json(url)`, so caching, timeouts, and error logging live in exactly one place. User-supplied filter values are percent-encoded in one place (`repositories::clause`), which closes PostgREST filter-injection.

## Measured performance

Server-side timings (`curl`, on the box, so app + DB only):

| Path | Cold (cache miss) | Warm (cache hit) |
|---|---|---|
| `/health` (no DB) | — | **0.3 ms** |
| `/?zone=now` (3 queries) | ~45–99 ms | **0.6 ms** |
| `/api/events` (1 query) | ~40–51 ms | **0.5 ms** |

**The dominant cost is the network round-trip to Supabase, not the framework or
rendering.** `/health` proves app overhead is ~0.3 ms. Everything else is the
PostgREST hop. This single fact drives the decisions below.

## Decision: Axum (not Rocket)

Axum sits directly on hyper + tower; we use the tower middleware stack (compression,
tracing, scoped CORS, and a request body-size limit). The Supabase HTTP client
carries its own 5 s connect / 15 s request timeout (`db.rs`). Axum benchmarks at
lower overhead than Rocket, whose strength is ergonomic macros/guards rather than
raw throughput.

But the honest reason to **not** relitigate this: at 0.3 ms of framework time
against a ~45 ms network hop, the framework choice is performance-irrelevant here.
Rocket would render the same page in the same ~45 ms. We optimise the data path
instead (below), which is where the time actually is.

## Decision: PostgREST today, `sqlx` on the table

We talk to PostgreSQL through Supabase's PostgREST (REST-over-Postgres) rather than
a direct driver. Trade-offs, stated plainly:

**For PostgREST (current):**
- No DB port or connection pool to manage; just an HTTPS URL + key. Trivial to run
  anywhere, including from a constrained host.
- Row-level security and the RPC functions live in the database, not the app.

**Against PostgREST / for `sqlx`:**
- Every read is an HTTP request — that's the ~45 ms above. A direct Postgres
  connection (pooled) removes that hop.
- `sqlx::query_as!` checks SQL **at compile time** against the live schema — a
  renamed column fails `cargo build`, not production.
- Real joins in one round-trip instead of N PostgREST calls per page.

The repository layer is the seam: only `db.rs` and the repo impls know about
PostgREST. Swapping them for `sqlx` would not touch routes, services, or SSR.

**Portability.** Nothing here is locked to Supabase. The schema is standard
PostgreSQL; the data is a `pg_dump` away from any Postgres (self-hosted, RDS,
Neon, …). "Supabase" is just a managed Postgres + PostgREST + auth. Porting the
database means pointing at a different Postgres; porting *off* PostgREST is the
`sqlx` migration above. Both are open, neither is forced.

## Decision: HTMX (not a WASM SPA)

Pages are server-rendered HTML; HTMX (~14 KB, no build step) swaps fragments for
interactivity (filtered lists, upvote buttons). Versus a Yew/WASM SPA: no WASM
bundle to download, parse, and hydrate, so first paint is immediate and the
client stays cheap — the right call for a content/directory site. The cost is
that rich client state is awkward; we don't need it here.

## Caching

A 30 s in-memory TTL cache keyed by query URL (`cache.rs`) fronts `get_json`.
Warm reads skip the network hop entirely (the ~150× above). Staleness is bounded
by the TTL — a deliberate trade of strict consistency for far fewer round-trips,
which suits a slowly-changing public directory. Writes (votes, upvotes,
submissions) go through other methods and surface on the next refresh. The CSS is
served once at `/style.css` (browser-cached) instead of inline in every page.

One `Mutex<HashMap>` is enough at this key cardinality; if a profile ever showed
contention, `moka` drops in behind the same interface.

## Testing

- `cargo test --lib` — unit tests, no network: `Zone::parse`, row deserialisation,
  the `clause` injection guard, `esc` XSS guard, and the TTL cache.
- `cargo test --test api_tests` — HTTP integration tests against a `TestServer`
  (skip gracefully without `SUPABASE_URL`).

## Considered and deferred

- **Full Maud (compile-time HTML) rewrite of the SSR zones.** Attractive
  (type-checked templates, fewer allocations), but the SSR layer is slated for a
  Leptos frontend; rewriting it twice isn't worth it. The string-built HTML is
  contained in `ssr/zones/` and `ui.rs`.
- **`sqlx` migration.** See above — the repository seam makes it a localized change
  when/if the network hop or compile-time-checked SQL becomes worth the loss of
  PostgREST's zero-setup portability.
