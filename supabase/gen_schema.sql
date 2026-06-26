-- supabase/gen_schema.sql — stage the live schema for scripts/gen_schema.py.
--
-- Why this exists: the app server holds only the PostgREST keys, not the database
-- password, so `pg_dump` / `supabase db dump` aren't available here. This stages a
-- catalog-derived DDL dump into a temp table that gen_schema.py then pulls over
-- PostgREST (using the service key already in .env) to write supabase/schema.sql —
-- a regeneration path that needs no DB password.
--
-- Regenerate schema.sql:
--   1. Run THIS file against the database (Supabase SQL editor, or the MCP).
--   2. python3 scripts/gen_schema.py        (writes supabase/schema.sql)
--   3. DROP TABLE public._schema_dump;       (cleanup)
-- If you DO have the DB password, `supabase db dump --schema public` is the cleaner
-- canonical alternative (see README.md).
--
-- Creates+populates a transient public._schema_dump table; safe to drop afterwards.

DROP TABLE IF EXISTS public._schema_dump;
CREATE TABLE public._schema_dump (seq int, kind text, name text, ddl text);

-- 0: sequences
INSERT INTO public._schema_dump
SELECT 0,'sequence',sequencename, format('CREATE SEQUENCE IF NOT EXISTS %I;', sequencename)
FROM pg_sequences WHERE schemaname='public';

-- 1: enum types
INSERT INTO public._schema_dump
SELECT 1,'type',t.typname,
  format('CREATE TYPE %I AS ENUM (%s);', t.typname,
    string_agg(quote_literal(e.enumlabel), ', ' ORDER BY e.enumsortorder))
FROM pg_type t JOIN pg_enum e ON e.enumtypid=t.oid
JOIN pg_namespace n ON n.oid=t.typnamespace WHERE n.nspname='public' GROUP BY t.typname;

-- 2: tables (columns + non-FK constraints)
INSERT INTO public._schema_dump
SELECT 2,'table',c.relname,
  format(E'CREATE TABLE %I (\n%s%s\n);', c.relname,
    (SELECT string_agg('    '||quote_ident(a.attname)||' '||format_type(a.atttypid,a.atttypmod)
        ||CASE WHEN a.attnotnull THEN ' NOT NULL' ELSE '' END
        ||COALESCE(' DEFAULT '||pg_get_expr(ad.adbin,ad.adrelid),''), E',\n' ORDER BY a.attnum)
     FROM pg_attribute a LEFT JOIN pg_attrdef ad ON ad.adrelid=a.attrelid AND ad.adnum=a.attnum
     WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),
    COALESCE((SELECT E',\n'||string_agg('    '||pg_get_constraintdef(con.oid), E',\n' ORDER BY con.contype DESC, con.conname)
     FROM pg_constraint con WHERE con.conrelid=c.oid AND con.contype<>'f'),''))
FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
WHERE n.nspname='public' AND c.relkind='r' AND c.relname NOT LIKE '\_schema\_%';

-- 3: foreign keys (added after all tables exist, so order doesn't matter)
INSERT INTO public._schema_dump
SELECT 3,'fk',c.relname,
  format('ALTER TABLE %I ADD CONSTRAINT %I %s;', c.relname, con.conname, pg_get_constraintdef(con.oid))
FROM pg_constraint con JOIN pg_class c ON c.oid=con.conrelid
JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND con.contype='f';

-- 4: secondary indexes (not backing a constraint)
INSERT INTO public._schema_dump
SELECT 4,'index',i.indexname, i.indexdef||';'
FROM pg_indexes i WHERE i.schemaname='public' AND i.tablename NOT LIKE '\_schema\_%'
  AND NOT EXISTS (SELECT 1 FROM pg_constraint con WHERE con.conindid=(format('public.%I',i.indexname)::regclass));

-- 5: views
INSERT INTO public._schema_dump
SELECT 5,'view',v.viewname,
  format(E'CREATE OR REPLACE VIEW %I AS\n%s', v.viewname, pg_get_viewdef(format('public.%I',v.viewname)::regclass, true))
FROM pg_views v WHERE v.schemaname='public';

-- 6: functions
INSERT INTO public._schema_dump
SELECT 6,'function',p.proname, pg_get_functiondef(p.oid)||';'
FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public';

-- 7: RLS enable
INSERT INTO public._schema_dump
SELECT 7,'rls',tablename, format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY;', tablename)
FROM (SELECT DISTINCT tablename FROM pg_policies WHERE schemaname='public') x;

-- 8: policies
INSERT INTO public._schema_dump
SELECT 8,'policy',policyname,
  format('CREATE POLICY %I ON %I AS %s FOR %s TO %s%s%s;',
    policyname, tablename, permissive, cmd, array_to_string(roles, ', '),
    COALESCE(' USING ('||qual||')',''), COALESCE(' WITH CHECK ('||with_check||')',''))
FROM pg_policies WHERE schemaname='public';

-- Let PostgREST see the new table, then run scripts/gen_schema.py.
SELECT pg_notify('pgrst','reload schema');
