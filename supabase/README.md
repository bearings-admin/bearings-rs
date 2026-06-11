# Supabase / database setup

This directory captures the database **structure** so the project isn't tied to
Supabase — the data is standard PostgreSQL and can move to any Postgres host.

## Files

- **`schema.sql`** — the full public schema (tables, constraints, functions,
  triggers, views, RLS policies), captured via catalog introspection on
  2026-06-11. It reconstructs the structure; it is *not* a `pg_dump` (the app
  server holds only the REST keys, not the database password).
- **`../deploy/sql/`** — the original hand-written DDL for specific features
  (`places_nearby`, `submissions_table`, `zone_functions`, `user_preferences_wallet`).

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
