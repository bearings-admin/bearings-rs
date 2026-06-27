# Supabase / database setup

This directory captures the database **structure** so the project isn't tied to
Supabase — the data is standard PostgreSQL and can move to any Postgres host.

## Files

- **`schema.sql`** — the full current public schema (sequences, enums, tables,
  constraints, foreign keys, indexes, views, functions, RLS policies). **Generated
  from the live catalog** (not a `pg_dump` — the app server holds only the REST
  keys, not the DB password). Regenerate it with `scripts/gen_schema.py` (below).
- **`gen_schema.sql`** — definition of the `public.schema_dump()` RPC (already applied)
  that introspects the catalog server-side. **`../scripts/gen_schema.py`** calls it to
  regenerate / drift-check `schema.sql` — password-free (see below).
- **`../deploy/sql/`** — the original hand-written DDL for specific features
  (`places_nearby`, `submissions_table`, `zone_functions`, `user_preferences_wallet`).

## Source of truth & change workflow

**The live database catalog is the source of truth — `schema.sql` mirrors it.**
Two things make this non-obvious and worth stating plainly:

- **The Supabase migration tracker is NOT complete.** 25 of the 39 tables — including
  the core domain (`events`, `places`, `clubs`, `competitions`, `title_holders`,
  `campaigns`, `bear_history`, …) — were created *outside* the migration system
  (dashboard / raw SQL, before the migration discipline started). So you cannot rebuild
  the schema from `supabase/migrations` alone; `schema.sql` (catalog-generated) is what
  reproduces it.
- **Schema changes currently bypass the PR/CI gate.** They're applied straight to the
  DB (dashboard / MCP `apply_migration`). Going forward, treat a schema change like a
  code change: apply it, **then regenerate `schema.sql` and commit it in the same PR**,
  so the repo and the database stay in lockstep and reviewers can see the diff.

### Regenerate / check `schema.sql` (no DB password needed)

```sh
python3 scripts/gen_schema.py            # rewrite supabase/schema.sql from the live DB
python3 scripts/gen_schema.py --check    # exit 1 (with a diff) if it has drifted
```

It calls the `public.schema_dump()` RPC over PostgREST using the Supabase keys in
`.env` — no DB password / `pg_dump`. (If you *do* have the DB password, `supabase db
dump --schema public -f supabase/schema.sql` is the byte-exact alternative.)

### Enforce drift in CI (opt-in)

The CI "Python" job runs `gen_schema.py --check` **only if** the repo has the
`SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY` **Actions secrets** set (it skips
otherwise, so it never blocks). Enabling it means whoever changes the DB must also
commit the regenerated `schema.sql` before their next PR can merge. Adding the
service-role key to Actions is a security choice — it's the owner's call:

```sh
gh secret set SUPABASE_URL --repo bearings-admin/bearings-rs
gh secret set SUPABASE_SERVICE_ROLE_KEY --repo bearings-admin/bearings-rs
```

A scheduled VPS check (it already has the key) is the other option, and catches the
common case where the DB is changed outside any PR.

## Authoritative backup (recommended)

For a canonical, restorable dump, run `pg_dump` with the database password from the
Supabase dashboard (Settings → Database → Connection string):

```sh
# structure only
pg_dump --schema-only --no-owner --no-privileges \
  "postgresql://postgres:<DB_PASSWORD>@db.mntdhflffhrjjvipxgyl.supabase.co:5432/postgres" \
  > schema.sql

# data only (kept separate from the app, by design)
pg_dump --data-only --no-owner \
  "postgresql://postgres:<DB_PASSWORD>@db.mntdhflffhrjjvipxgyl.supabase.co:5432/postgres" \
  > data.sql
```

> Data is intentionally **not** committed here — keeping it outside the app/repo
> is a deliberate security choice. Export it separately when needed.

## Porting to another Postgres

Supabase *is* PostgreSQL, so moving is straightforward:

1. Provision any Postgres (self-hosted, RDS, Neon, a Postgres on the VPS, …).
2. `psql "<new connection string>" -f schema.sql`
3. Load data from your separate `data.sql` export.
4. Point the backend at the new database — only `bearings-backend/src/db.rs`
   changes. With the current PostgREST client that means a new base URL (and a
   PostgREST instance in front); with the `sqlx` option (see
   `bearings-backend/ARCHITECTURE.md`) it's just a connection string.

The app talks to the database only through the repository layer
(`bearings-backend/src/repositories/`), so the database host is swappable without
touching routes, services, or rendering.
