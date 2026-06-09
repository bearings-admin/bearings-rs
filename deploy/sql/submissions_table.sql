
-- deploy/sql/submissions_table.sql
-- Run in Supabase SQL editor.
-- Creates the public-write submissions intake table.

CREATE TABLE IF NOT EXISTS submissions (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  submission_type text NOT NULL CHECK (submission_type IN ('event','place','club','story','creator','digital_space')),
  name text NOT NULL,
  city text,
  country text,
  link text,
  description text,
  contact_email text,
  submitter_name text,
  source text,
  status text DEFAULT 'pending_review' CHECK (status IN ('pending_review','auto_approved','rejected','live')),
  privacy_mode boolean DEFAULT false,
  urgent boolean DEFAULT false,
  reviewed_by text,
  reviewed_at timestamptz,
  created_at timestamptz DEFAULT now()
);

-- RLS: public can insert, only service role can read/update
ALTER TABLE submissions ENABLE ROW LEVEL SECURITY;

CREATE POLICY "public_insert_submissions" ON submissions
  FOR INSERT WITH CHECK (true);

CREATE POLICY "service_role_all_submissions" ON submissions
  FOR ALL USING (auth.role() = 'service_role');

-- Index for steward review queue
CREATE INDEX IF NOT EXISTS idx_submissions_status ON submissions(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_submissions_urgent ON submissions(urgent, created_at DESC) WHERE urgent = true;

COMMENT ON TABLE submissions IS
  'Public-write intake for new listing submissions.
   Bears submit events, places, clubs, etc. for review.
   Primary intake is the chatbot agent — this is the fallback form.
   Only the submissions and newsletter_subscribers tables accept public writes.';
