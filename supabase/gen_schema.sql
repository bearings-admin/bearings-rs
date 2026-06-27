-- supabase/gen_schema.sql — definition of the public.schema_dump() function that
-- scripts/gen_schema.py calls to (re)generate or --check supabase/schema.sql.
--
-- This is already applied to the database (migration create_schema_dump_function);
-- kept here so the function is reviewable in the repo and re-creatable if needed.
-- It runs the catalog introspection server-side and returns ordered DDL rows, so a
-- Supabase key is enough — no DB password / pg_dump. SECURITY DEFINER; service_role only.

CREATE OR REPLACE FUNCTION public.schema_dump()
RETURNS TABLE(seq int, kind text, name text, ddl text)
LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_catalog AS $fn$
  SELECT 0,'sequence',sequencename::text, format('CREATE SEQUENCE IF NOT EXISTS %I;', sequencename)
  FROM pg_sequences WHERE schemaname='public'
  UNION ALL
  SELECT 1,'type',t.typname::text,
    format('CREATE TYPE %I AS ENUM (%s);', t.typname,
      string_agg(quote_literal(e.enumlabel), ', ' ORDER BY e.enumsortorder))
  FROM pg_type t JOIN pg_enum e ON e.enumtypid=t.oid
  JOIN pg_namespace n ON n.oid=t.typnamespace WHERE n.nspname='public' GROUP BY t.typname
  UNION ALL
  SELECT 2,'table',c.relname::text,
    format(E'CREATE TABLE %I (\n%s%s\n);', c.relname,
      (SELECT string_agg('    '||quote_ident(a.attname)||' '||format_type(a.atttypid,a.atttypmod)
          ||CASE WHEN a.attnotnull THEN ' NOT NULL' ELSE '' END
          ||COALESCE(' DEFAULT '||pg_get_expr(ad.adbin,ad.adrelid),''), E',\n' ORDER BY a.attnum)
       FROM pg_attribute a LEFT JOIN pg_attrdef ad ON ad.adrelid=a.attrelid AND ad.adnum=a.attnum
       WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),
      COALESCE((SELECT E',\n'||string_agg('    '||pg_get_constraintdef(con.oid), E',\n' ORDER BY con.contype DESC, con.conname)
       FROM pg_constraint con WHERE con.conrelid=c.oid AND con.contype<>'f'),''))
  FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relkind='r' AND c.relname NOT LIKE '\_%'
  UNION ALL
  SELECT 3,'fk',c.relname::text,
    format('ALTER TABLE %I ADD CONSTRAINT %I %s;', c.relname, con.conname, pg_get_constraintdef(con.oid))
  FROM pg_constraint con JOIN pg_class c ON c.oid=con.conrelid
  JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND con.contype='f'
  UNION ALL
  SELECT 4,'index',i.indexname::text, i.indexdef||';'
  FROM pg_indexes i WHERE i.schemaname='public' AND i.tablename NOT LIKE '\_%'
    AND NOT EXISTS (SELECT 1 FROM pg_constraint con WHERE con.conindid=(format('public.%I',i.indexname)::regclass))
  UNION ALL
  SELECT 5,'view',v.viewname::text,
    format(E'CREATE OR REPLACE VIEW %I AS\n%s', v.viewname, pg_get_viewdef(format('public.%I',v.viewname)::regclass, true))
  FROM pg_views v WHERE v.schemaname='public'
  UNION ALL
  SELECT 6,'function',p.proname::text, pg_get_functiondef(p.oid)||';'
  FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
  WHERE n.nspname='public' AND p.proname<>'schema_dump'
  UNION ALL
  SELECT 7,'rls',tablename::text, format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY;', tablename)
  FROM (SELECT DISTINCT tablename FROM pg_policies WHERE schemaname='public') x
  UNION ALL
  SELECT 8,'policy',policyname::text,
    format('CREATE POLICY %I ON %I AS %s FOR %s TO %s%s%s;',
      policyname, tablename, permissive, cmd, array_to_string(roles, ', '),
      COALESCE(' USING ('||qual||')',''), COALESCE(' WITH CHECK ('||with_check||')',''))
  FROM pg_policies WHERE schemaname='public'
$fn$;
REVOKE ALL ON FUNCTION public.schema_dump() FROM public;
GRANT EXECUTE ON FUNCTION public.schema_dump() TO service_role;
