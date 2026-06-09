
-- deploy/sql/user_preferences_wallet.sql
-- Run in Supabase SQL editor BEFORE enabling Bear Future zone.
-- Adds embedded wallet and contributor tier fields.

ALTER TABLE user_preferences
  ADD COLUMN IF NOT EXISTS contributor_tier text DEFAULT 'anonymous',
  ADD COLUMN IF NOT EXISTS custodial_wallet text,
  ADD COLUMN IF NOT EXISTS self_custody_wallet text,
  ADD COLUMN IF NOT EXISTS wallet_type text DEFAULT 'custodial',
  ADD COLUMN IF NOT EXISTS privy_user_id text,
  ADD COLUMN IF NOT EXISTS token_chain text DEFAULT 'cardano';

COMMENT ON COLUMN user_preferences.contributor_tier IS
  'anonymous | community | verified_contributor | club_officer | steward';
COMMENT ON COLUMN user_preferences.custodial_wallet IS
  'Cardano address of the server-side custodial wallet. NEVER returned in public API.';
COMMENT ON COLUMN user_preferences.self_custody_wallet IS
  'Cardano address of the bear''s own wallet (Eternl, Lace). NEVER returned in public API.';
