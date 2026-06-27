-- Bearings — database schema (GENERATED from the live catalog; do not hand-edit)
-- Regenerate / check: scripts/gen_schema.py [--check]  (see supabase/README.md)
-- The live catalog is the source of truth — 25 of 39 tables predate the migration
-- tracker, so this generated file (not the migrations) reproduces the schema.
-- Catalog-derived, not pg_dump (sequences bare); `supabase db dump` for exact restore.

-- ========================================================================
-- Sequences
-- ========================================================================
CREATE SEQUENCE IF NOT EXISTS agent_actions_id_seq;

CREATE SEQUENCE IF NOT EXISTS agent_inbox_id_seq;

CREATE SEQUENCE IF NOT EXISTS agent_posts_id_seq;

CREATE SEQUENCE IF NOT EXISTS artifacts_id_seq;

CREATE SEQUENCE IF NOT EXISTS bear_future_proposals_id_seq;

CREATE SEQUENCE IF NOT EXISTS bear_history_id_seq;

CREATE SEQUENCE IF NOT EXISTS bear_regions_id_seq;

CREATE SEQUENCE IF NOT EXISTS campaigns_id_seq;

CREATE SEQUENCE IF NOT EXISTS candidate_events_id_seq;

CREATE SEQUENCE IF NOT EXISTS candidate_title_holders_id_seq;

CREATE SEQUENCE IF NOT EXISTS clubs_id_seq;

CREATE SEQUENCE IF NOT EXISTS code_id_seq;

CREATE SEQUENCE IF NOT EXISTS competitions_id_seq;

CREATE SEQUENCE IF NOT EXISTS creator_event_links_id_seq;

CREATE SEQUENCE IF NOT EXISTS creators_id_seq;

CREATE SEQUENCE IF NOT EXISTS digital_space_event_links_id_seq;

CREATE SEQUENCE IF NOT EXISTS digital_spaces_id_seq;

CREATE SEQUENCE IF NOT EXISTS document_archive_id_seq;

CREATE SEQUENCE IF NOT EXISTS documents_id_seq;

CREATE SEQUENCE IF NOT EXISTS event_place_links_id_seq;

CREATE SEQUENCE IF NOT EXISTS events_id_seq;

CREATE SEQUENCE IF NOT EXISTS future_ideas_id_seq;

CREATE SEQUENCE IF NOT EXISTS inclusion_flags_id_seq;

CREATE SEQUENCE IF NOT EXISTS kindred_sources_id_seq;

CREATE SEQUENCE IF NOT EXISTS media_id_seq;

CREATE SEQUENCE IF NOT EXISTS newsletter_subscribers_id_seq;

CREATE SEQUENCE IF NOT EXISTS operating_costs_id_seq;

CREATE SEQUENCE IF NOT EXISTS operational_ledger_id_seq;

CREATE SEQUENCE IF NOT EXISTS places_id_seq;

CREATE SEQUENCE IF NOT EXISTS sponsor_event_links_id_seq;

CREATE SEQUENCE IF NOT EXISTS sponsors_id_seq;

CREATE SEQUENCE IF NOT EXISTS stores_id_seq;

CREATE SEQUENCE IF NOT EXISTS stories_id_seq;

CREATE SEQUENCE IF NOT EXISTS submissions_id_seq;

CREATE SEQUENCE IF NOT EXISTS title_holders_id_seq;

CREATE SEQUENCE IF NOT EXISTS translations_id_seq;

CREATE SEQUENCE IF NOT EXISTS user_preferences_id_seq;

CREATE SEQUENCE IF NOT EXISTS watched_feeds_id_seq;


-- ========================================================================
-- Types (enums)
-- ========================================================================
CREATE TYPE holder_status_type AS ENUM ('active', 'holdover', 'vacant', 'unknown');


-- ========================================================================
-- Tables (columns + PK/unique/check)
-- ========================================================================
CREATE TABLE agent_actions (
    id bigint NOT NULL DEFAULT nextval('agent_actions_id_seq'::regclass),
    agent text NOT NULL,
    action text NOT NULL,
    candidate_id bigint,
    event_id bigint,
    series_name text,
    detail text,
    model text,
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE agent_inbox (
    id bigint NOT NULL,
    platform text NOT NULL DEFAULT 'bluesky'::text,
    post_uri text NOT NULL,
    post_cid text,
    author_handle text,
    author_did text,
    post_text text,
    in_reply_to_uri text,
    reply_to_post_id bigint,
    intent text,
    status text DEFAULT 'unread'::text,
    response_text text,
    response_uri text,
    responded_at timestamp with time zone,
    escalated_to_steward boolean DEFAULT false,
    escalation_reason text,
    received_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE agent_posts (
    id bigint NOT NULL,
    platform text NOT NULL DEFAULT 'bluesky'::text,
    post_type text NOT NULL,
    event_id bigint,
    place_id bigint,
    creator_id bigint,
    club_id bigint,
    campaign_id bigint,
    history_id bigint,
    post_text text NOT NULL,
    post_uri text,
    post_cid text,
    scheduled_for timestamp with time zone,
    published_at timestamp with time zone,
    status text DEFAULT 'draft'::text,
    like_count integer DEFAULT 0,
    repost_count integer DEFAULT 0,
    reply_count integer DEFAULT 0,
    quote_count integer DEFAULT 0,
    generated_by text DEFAULT 'n8n-agent'::text,
    reviewed_by_steward boolean DEFAULT false,
    notes text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE artifacts (
    id bigint NOT NULL,
    kind text NOT NULL DEFAULT 'photo'::text,
    title text NOT NULL,
    description text,
    transcription text,
    contributor text,
    provenance text,
    captured_on date,
    image_url text,
    entity_type text,
    entity_id bigint,
    active boolean NOT NULL DEFAULT true,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE bear_future_proposals (
    id bigint NOT NULL,
    title text NOT NULL,
    description text NOT NULL,
    cause_category text,
    target_amount_usdc numeric NOT NULL,
    target_amount_usd numeric,
    raised_usdc numeric DEFAULT 0,
    receiving_wallet text,
    applicant_name text,
    applicant_email text,
    applicant_club_id bigint,
    supporting_link text,
    status text DEFAULT 'draft'::text,
    funded_at timestamp with time zone,
    tx_hash text,
    privacy_mode boolean DEFAULT false,
    urgent boolean DEFAULT false,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    reviewed_by_steward boolean DEFAULT false,
    steward_notes text,
    PRIMARY KEY (id)
);

CREATE TABLE bear_history (
    id bigint NOT NULL,
    title text NOT NULL,
    year integer NOT NULL,
    month integer,
    description text,
    significance text,
    category text,
    link text,
    creator_id bigint,
    event_id bigint,
    club_id bigint,
    tags text[],
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    featured boolean DEFAULT false,
    PRIMARY KEY (id)
);

CREATE TABLE bear_regions (
    id bigint NOT NULL,
    region text NOT NULL,
    scope text DEFAULT 'country'::text,
    organizing_style text,
    gathering_pattern text,
    primary_platforms text,
    research_sources text,
    safety_note text,
    discovery_note text,
    active boolean DEFAULT true,
    updated_at timestamp with time zone DEFAULT now(),
    payment_rails text,
    language text,
    PRIMARY KEY (id)
);

CREATE TABLE campaigns (
    id bigint NOT NULL,
    name text NOT NULL,
    goal integer,
    raised integer DEFAULT 0,
    currency text DEFAULT 'CAD'::text,
    urgent boolean DEFAULT false,
    link text,
    org text,
    description text,
    days_remaining integer,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    ends_at date,
    cra_number text,
    usdc_accepted boolean DEFAULT false,
    privacy_mode boolean DEFAULT false,
    cause text,
    donate_url text,
    PRIMARY KEY (id)
);

CREATE TABLE candidate_events (
    id bigint NOT NULL DEFAULT nextval('candidate_events_id_seq'::regclass),
    feed_id bigint,
    source_url text NOT NULL,
    raw_title text NOT NULL,
    raw_description text,
    raw_date text,
    raw_location text,
    parsed_name text,
    parsed_city text,
    parsed_country text,
    parsed_start date,
    parsed_end date,
    parsed_type text,
    status text NOT NULL DEFAULT 'pending'::text,
    steward_notes text,
    event_id bigint,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    reviewed_at timestamp with time zone,
    UNIQUE (source_url),
    PRIMARY KEY (id),
    CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text, 'duplicate'::text, 'auto_applied'::text])))
);

CREATE TABLE candidate_title_holders (
    id bigint NOT NULL DEFAULT nextval('candidate_title_holders_id_seq'::regclass),
    title_name text NOT NULL,
    holder_name text NOT NULL,
    year integer,
    city text,
    country text,
    competition_id bigint,
    source_url text,
    evidence text,
    status text NOT NULL DEFAULT 'pending'::text,
    title_holder_id bigint,
    steward_notes text,
    reviewed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id),
    CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text, 'duplicate'::text])))
);

CREATE TABLE clubs (
    id bigint NOT NULL,
    name text NOT NULL,
    club_type text DEFAULT 'event-producing'::text,
    city text,
    country text,
    lat numeric(9,6),
    lng numeric(9,6),
    founded_year integer,
    closed_year integer,
    active boolean DEFAULT true,
    website text,
    facebook text,
    instagram text,
    description text,
    tags text[],
    contact_email text,
    ical_url text,
    home_place_id bigint,
    membership_fee text,
    meeting_schedule text,
    charity_name text,
    charity_link text,
    created_at timestamp with time zone DEFAULT now(),
    founding_note text,
    bluesky_handle text,
    linktree_url text,
    slug text,
    contact_name text,
    contact_social text,
    validator_notes text,
    outreach_status text DEFAULT 'not_started'::text,
    revival_votes integer DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE TABLE code (
    id bigint NOT NULL,
    crate text NOT NULL,
    file_path text NOT NULL,
    content text NOT NULL,
    language text DEFAULT 'rust'::text,
    description text,
    version text,
    updated_at timestamp with time zone DEFAULT now(),
    updated_by text DEFAULT 'agent'::text,
    active boolean DEFAULT true,
    UNIQUE (crate, file_path),
    PRIMARY KEY (id)
);

CREATE TABLE competitions (
    id bigint NOT NULL,
    name text NOT NULL,
    competition_type text DEFAULT 'bear'::text,
    scope text DEFAULT 'regional'::text,
    frequency text DEFAULT 'annual'::text,
    owning_club_id bigint,
    host_event_id bigint,
    city text,
    country text,
    website text,
    description text,
    tags text[],
    active boolean DEFAULT true,
    founded_year integer,
    charity_focus text,
    inclusion_flag_codes text[],
    inclusion_notes text,
    created_at timestamp with time zone DEFAULT now(),
    discontinued_year integer,
    slug text,
    contact_email text,
    contact_name text,
    contact_social text,
    validator_notes text,
    outreach_status text DEFAULT 'not_started'::text,
    PRIMARY KEY (id)
);

CREATE TABLE creator_event_links (
    id bigint NOT NULL,
    creator_id bigint,
    event_id bigint,
    role text,
    year integer,
    note text,
    official boolean DEFAULT false,
    PRIMARY KEY (id)
);

CREATE TABLE creators (
    id bigint NOT NULL,
    name text NOT NULL,
    creator_type text,
    pronouns text,
    city text,
    country text,
    lat numeric,
    lng numeric,
    website text,
    instagram text,
    facebook text,
    spotify_link text,
    youtube_link text,
    bandcamp_link text,
    patreon_link text,
    booking_link text,
    bio text,
    tags text[],
    bear_community_member boolean DEFAULT true,
    bear_affiliated boolean DEFAULT true,
    active boolean DEFAULT true,
    verified boolean DEFAULT false,
    inclusion_flag_codes text[],
    inclusion_notes text,
    created_at timestamp with time zone DEFAULT now(),
    bluesky_handle text,
    linktree_url text,
    etsy_link text,
    slug text,
    PRIMARY KEY (id)
);

CREATE TABLE digital_space_event_links (
    id bigint NOT NULL,
    digital_space_id bigint,
    event_id bigint,
    relationship text,
    note text,
    official boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE digital_spaces (
    id bigint NOT NULL,
    name text NOT NULL,
    space_type text NOT NULL,
    platform text,
    url text,
    app_store_ios text,
    app_store_android text,
    description text,
    tags text[],
    bear_specific boolean DEFAULT true,
    community_types text[],
    nsfw_content boolean DEFAULT false,
    age_restriction text,
    member_count integer,
    member_count_note text,
    active boolean DEFAULT true,
    verified boolean DEFAULT false,
    discord_invite text,
    twitch_handle text,
    bluesky_handle text,
    instagram text,
    tiktok_handle text,
    reddit_handle text,
    linked_club_id bigint,
    linked_creator_id bigint,
    city text,
    country text,
    region text,
    inclusion_flag_codes text[],
    inclusion_notes text,
    founded_year integer,
    closed_year integer,
    created_at timestamp with time zone DEFAULT now(),
    telegram_link text,
    telegram_handle text,
    discord_server_id text,
    youtube_handle text,
    patreon_link text,
    onlyfans_link text,
    game_title text,
    game_server text,
    covid_origin boolean DEFAULT false,
    event_linked boolean DEFAULT false,
    booking_notes text,
    PRIMARY KEY (id)
);

CREATE TABLE document_archive (
    id bigint NOT NULL,
    slug text NOT NULL,
    title text NOT NULL,
    version text,
    content text NOT NULL,
    archived_at timestamp with time zone DEFAULT now(),
    archived_by text DEFAULT 'steward'::text,
    notes text,
    PRIMARY KEY (id)
);

CREATE TABLE documents (
    id bigint NOT NULL,
    slug text NOT NULL,
    title text NOT NULL,
    version text,
    content text NOT NULL,
    updated_at timestamp with time zone DEFAULT now(),
    updated_by text DEFAULT 'steward'::text,
    active boolean DEFAULT true,
    tags text[],
    UNIQUE (slug),
    PRIMARY KEY (id)
);

CREATE TABLE event_dupe_ignores (
    lo_id bigint NOT NULL,
    hi_id bigint NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    PRIMARY KEY (lo_id, hi_id)
);

CREATE TABLE event_place_links (
    id bigint NOT NULL,
    event_id bigint,
    place_id bigint,
    relationship text,
    note text,
    official boolean DEFAULT false,
    distance_km numeric,
    PRIMARY KEY (id)
);

CREATE TABLE events (
    id bigint NOT NULL,
    name text NOT NULL,
    city text,
    country text,
    lat numeric(9,6),
    lng numeric(9,6),
    start_date date,
    end_date date,
    month text,
    type text DEFAULT 'event'::text,
    size text DEFAULT 'regional'::text,
    hot boolean DEFAULT false,
    link text,
    tags text[],
    description text,
    going integer DEFAULT 0,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    source text DEFAULT 'manual'::text,
    source_id text,
    charity_name text,
    charity_link text,
    host_hotel text,
    host_hotel_link text,
    inclusion_flag_codes text[],
    inclusion_notes text,
    status text DEFAULT 'upcoming'::text,
    archive_notes text,
    bluesky_handle text,
    slug text,
    event_mode text DEFAULT 'physical'::text,
    stream_url text,
    platform text,
    recurring boolean DEFAULT false,
    recurrence_note text,
    PRIMARY KEY (id)
);

CREATE TABLE future_ideas (
    id bigint NOT NULL DEFAULT nextval('future_ideas_id_seq'::regclass),
    icon text NOT NULL DEFAULT '💡'::text,
    title text NOT NULL,
    description text NOT NULL,
    upvotes integer NOT NULL DEFAULT 0,
    source text NOT NULL DEFAULT 'curated'::text,
    active boolean NOT NULL DEFAULT true,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE inclusion_flags (
    id bigint NOT NULL,
    code text NOT NULL,
    label text NOT NULL,
    description text NOT NULL,
    severity text DEFAULT 'caution'::text,
    affected_groups text[],
    icon text,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    UNIQUE (code),
    PRIMARY KEY (id)
);

CREATE TABLE kindred_sources (
    id bigint NOT NULL DEFAULT nextval('kindred_sources_id_seq'::regclass),
    name text NOT NULL,
    url text NOT NULL,
    blurb text,
    language text DEFAULT 'en'::text,
    kind text,
    featured boolean DEFAULT false,
    sort_order integer DEFAULT 100,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE media (
    id bigint NOT NULL,
    title text NOT NULL,
    creator_id bigint,
    media_type text,
    year integer,
    status text DEFAULT 'released'::text,
    description text,
    tags text[],
    link text,
    streaming_link text,
    trailer_link text,
    event_id bigint,
    bear_community_subject boolean DEFAULT false,
    featured boolean DEFAULT false,
    active boolean DEFAULT true,
    inclusion_flag_codes text[],
    inclusion_notes text,
    created_at timestamp with time zone DEFAULT now(),
    timeline_year integer,
    timeline_month integer,
    affiliate_link text,
    affiliate_pct numeric,
    PRIMARY KEY (id)
);

CREATE TABLE newsletter_subscribers (
    id bigint NOT NULL,
    email text NOT NULL,
    city text,
    language text DEFAULT 'en'::text,
    frequency text DEFAULT 'weekly'::text,
    interests text[],
    created_at timestamp with time zone DEFAULT now(),
    active boolean DEFAULT true,
    show_flagged_venues boolean DEFAULT true,
    hide_flag_codes text[] DEFAULT ARRAY[]::text[],
    UNIQUE (email),
    PRIMARY KEY (id)
);

CREATE TABLE operating_costs (
    id bigint NOT NULL,
    label text NOT NULL,
    amount_usd numeric NOT NULL DEFAULT 0,
    cadence text NOT NULL DEFAULT 'monthly'::text,
    category text,
    note text,
    active boolean NOT NULL DEFAULT true,
    updated_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE operational_ledger (
    id bigint NOT NULL,
    tx_hash text,
    tx_date date NOT NULL,
    direction text NOT NULL,
    amount_usdc numeric,
    amount_usd numeric,
    vendor text,
    description text,
    category text,
    authorized_by text,
    authorization_phase integer DEFAULT 1,
    donor_display text,
    donor_wallet text,
    active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    notes text,
    PRIMARY KEY (id)
);

CREATE TABLE places (
    id bigint NOT NULL,
    name text NOT NULL,
    place_type text NOT NULL,
    can_stay boolean DEFAULT false,
    hours_type text,
    city text,
    country text,
    lat numeric(9,6),
    lng numeric(9,6),
    address text,
    website text,
    booking_link text,
    misterbnb_link text,
    facebook text,
    instagram text,
    ical_url text,
    description text,
    tags text[],
    bear_owned boolean DEFAULT false,
    bear_welcoming boolean DEFAULT true,
    bear_popular boolean DEFAULT false,
    clothing_optional boolean DEFAULT false,
    men_only boolean DEFAULT false,
    price_range text,
    brunch_available boolean DEFAULT false,
    cuisine_type text,
    active boolean DEFAULT true,
    verified boolean DEFAULT false,
    closed_year integer,
    contact_email text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    source text DEFAULT 'manual'::text,
    inclusion_flag_codes text[],
    inclusion_notes text,
    inclusive_alternative_id bigint,
    has_bear_night boolean DEFAULT false,
    bear_night_schedule text,
    overnight_rate text,
    locker_rate text,
    foreigner_friendly boolean DEFAULT true,
    tattoo_policy text,
    booking_notes text,
    hours_open text,
    seasonal_open_month text,
    seasonal_close_month text,
    acreage integer,
    total_sites integer,
    has_rv_hookups boolean DEFAULT false,
    has_cabins boolean DEFAULT false,
    has_pool boolean DEFAULT false,
    has_hot_tub boolean DEFAULT false,
    booking_platform text,
    status text DEFAULT 'open'::text,
    archive_notes text,
    bluesky_handle text,
    slug text,
    revival_votes integer DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE TABLE platform_settings (
    key text NOT NULL,
    value text,
    note text,
    PRIMARY KEY (key)
);

CREATE TABLE sponsor_event_links (
    id bigint NOT NULL,
    sponsor_id bigint,
    event_id bigint,
    year integer,
    tier text,
    PRIMARY KEY (id)
);

CREATE TABLE sponsors (
    id bigint NOT NULL,
    name text NOT NULL,
    category text,
    website text,
    description text,
    bear_owned boolean DEFAULT false,
    bear_founded boolean DEFAULT false,
    size_inclusive boolean DEFAULT false,
    ships_global boolean DEFAULT false,
    active boolean DEFAULT true,
    featured boolean DEFAULT false,
    affiliate_link text,
    affiliate_pct numeric,
    tags text[],
    created_at timestamp with time zone DEFAULT now(),
    PRIMARY KEY (id)
);

CREATE TABLE stores (
    id bigint NOT NULL,
    name text NOT NULL,
    city text,
    country text,
    type text,
    link text,
    description text,
    tags text[],
    featured boolean DEFAULT false,
    active boolean DEFAULT true,
    available_for_events boolean DEFAULT false,
    available_for_auctions boolean DEFAULT false,
    size_inclusive boolean DEFAULT false,
    max_size text,
    ships_global boolean DEFAULT false,
    bear_owned boolean DEFAULT false,
    affiliate_link text,
    affiliate_pct numeric,
    contact_email text,
    created_at timestamp with time zone DEFAULT now(),
    bluesky_handle text,
    linktree_url text,
    etsy_link text,
    PRIMARY KEY (id)
);

CREATE TABLE stories (
    id bigint NOT NULL,
    title text NOT NULL,
    story_type text,
    creator_id bigint,
    event_id bigint,
    club_id bigint,
    year integer,
    body text,
    excerpt text,
    link text,
    archive_source text,
    language_code text DEFAULT 'en'::text,
    tags text[],
    active boolean DEFAULT true,
    featured boolean DEFAULT false,
    privacy_mode boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now(),
    timeline_year integer,
    timeline_month integer,
    bear_history_id bigint,
    PRIMARY KEY (id)
);

CREATE TABLE submissions (
    id bigint NOT NULL,
    submission_type text,
    name text,
    city text,
    country text,
    link text,
    description text,
    contact_email text,
    status text DEFAULT 'pending'::text,
    created_at timestamp with time zone DEFAULT now(),
    reviewed_at timestamp with time zone,
    notes text,
    chat_transcript jsonb,
    proposed_table text,
    proposed_fields jsonb,
    governance_ready boolean DEFAULT false,
    urgent boolean DEFAULT false,
    privacy_mode boolean DEFAULT false,
    source text DEFAULT 'chatbot'::text,
    PRIMARY KEY (id)
);

CREATE TABLE title_holders (
    id bigint NOT NULL,
    title_name text NOT NULL,
    holder_name text,
    year integer,
    club_id bigint,
    event_id bigint,
    city text,
    country text,
    charity_name text,
    charity_link text,
    fundraising_total integer,
    currency text DEFAULT 'USD'::text,
    photo_link text,
    bio text,
    contact_for_bookings text,
    active boolean DEFAULT true,
    inclusion_flag_codes text[],
    inclusion_notes text,
    created_at timestamp with time zone DEFAULT now(),
    competition_id bigint,
    holder_status holder_status_type NOT NULL DEFAULT 'active'::holder_status_type,
    holdover_reason text,
    PRIMARY KEY (id)
);

CREATE TABLE translations (
    id bigint NOT NULL,
    language_code text NOT NULL,
    key text NOT NULL,
    value text NOT NULL,
    status text DEFAULT 'approved'::text,
    submitted_by text,
    created_at timestamp with time zone DEFAULT now(),
    UNIQUE (language_code, key),
    PRIMARY KEY (id)
);

CREATE TABLE user_preferences (
    id bigint NOT NULL,
    session_id text,
    user_email text,
    show_all_venues boolean DEFAULT true,
    hide_flag_codes text[] DEFAULT ARRAY[]::text[],
    warn_flag_codes text[] DEFAULT ARRAY['trans_exclusionary'::text, 'cis_men_only'::text, 'bear_community_disputed'::text],
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    UNIQUE (session_id),
    PRIMARY KEY (id)
);

CREATE TABLE watched_feeds (
    id bigint NOT NULL DEFAULT nextval('watched_feeds_id_seq'::regclass),
    url text NOT NULL,
    feed_type text NOT NULL,
    org_name text NOT NULL,
    description text,
    last_fetched timestamp with time zone,
    last_etag text,
    last_modified text,
    fetch_errors integer NOT NULL DEFAULT 0,
    active boolean NOT NULL DEFAULT true,
    notes text,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    title_filter text,
    UNIQUE (url),
    PRIMARY KEY (id),
    CHECK ((feed_type = ANY (ARRAY['rss'::text, 'ical'::text, 'ical-static'::text, 'eventbrite'::text, 'scrape'::text])))
);


-- ========================================================================
-- Foreign keys (added after all tables exist)
-- ========================================================================
ALTER TABLE agent_inbox ADD CONSTRAINT agent_inbox_reply_to_post_id_fkey FOREIGN KEY (reply_to_post_id) REFERENCES agent_posts(id);

ALTER TABLE agent_posts ADD CONSTRAINT agent_posts_campaign_id_fkey FOREIGN KEY (campaign_id) REFERENCES campaigns(id);

ALTER TABLE agent_posts ADD CONSTRAINT agent_posts_club_id_fkey FOREIGN KEY (club_id) REFERENCES clubs(id);

ALTER TABLE agent_posts ADD CONSTRAINT agent_posts_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES creators(id);

ALTER TABLE agent_posts ADD CONSTRAINT agent_posts_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);

ALTER TABLE agent_posts ADD CONSTRAINT agent_posts_history_id_fkey FOREIGN KEY (history_id) REFERENCES bear_history(id);

ALTER TABLE agent_posts ADD CONSTRAINT agent_posts_place_id_fkey FOREIGN KEY (place_id) REFERENCES places(id);

ALTER TABLE bear_future_proposals ADD CONSTRAINT bear_future_proposals_applicant_club_id_fkey FOREIGN KEY (applicant_club_id) REFERENCES clubs(id);

ALTER TABLE bear_history ADD CONSTRAINT bear_history_club_id_fkey FOREIGN KEY (club_id) REFERENCES clubs(id);

ALTER TABLE bear_history ADD CONSTRAINT bear_history_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES creators(id);

ALTER TABLE bear_history ADD CONSTRAINT bear_history_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);

ALTER TABLE candidate_events ADD CONSTRAINT candidate_events_feed_id_fkey FOREIGN KEY (feed_id) REFERENCES watched_feeds(id);

ALTER TABLE clubs ADD CONSTRAINT clubs_home_place_id_fkey FOREIGN KEY (home_place_id) REFERENCES places(id);

ALTER TABLE competitions ADD CONSTRAINT competitions_host_event_id_fkey FOREIGN KEY (host_event_id) REFERENCES events(id);

ALTER TABLE competitions ADD CONSTRAINT competitions_owning_club_id_fkey FOREIGN KEY (owning_club_id) REFERENCES clubs(id);

ALTER TABLE creator_event_links ADD CONSTRAINT creator_event_links_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES creators(id);

ALTER TABLE creator_event_links ADD CONSTRAINT creator_event_links_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);

ALTER TABLE digital_space_event_links ADD CONSTRAINT digital_space_event_links_digital_space_id_fkey FOREIGN KEY (digital_space_id) REFERENCES digital_spaces(id);

ALTER TABLE digital_space_event_links ADD CONSTRAINT digital_space_event_links_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);

ALTER TABLE digital_spaces ADD CONSTRAINT digital_spaces_linked_club_id_fkey FOREIGN KEY (linked_club_id) REFERENCES clubs(id);

ALTER TABLE digital_spaces ADD CONSTRAINT digital_spaces_linked_creator_id_fkey FOREIGN KEY (linked_creator_id) REFERENCES creators(id);

ALTER TABLE event_place_links ADD CONSTRAINT event_place_links_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE;

ALTER TABLE event_place_links ADD CONSTRAINT event_place_links_place_id_fkey FOREIGN KEY (place_id) REFERENCES places(id) ON DELETE CASCADE;

ALTER TABLE media ADD CONSTRAINT media_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES creators(id);

ALTER TABLE media ADD CONSTRAINT media_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);

ALTER TABLE sponsor_event_links ADD CONSTRAINT sponsor_event_links_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE;

ALTER TABLE sponsor_event_links ADD CONSTRAINT sponsor_event_links_sponsor_id_fkey FOREIGN KEY (sponsor_id) REFERENCES sponsors(id) ON DELETE CASCADE;

ALTER TABLE stories ADD CONSTRAINT stories_bear_history_id_fkey FOREIGN KEY (bear_history_id) REFERENCES bear_history(id);

ALTER TABLE stories ADD CONSTRAINT stories_club_id_fkey FOREIGN KEY (club_id) REFERENCES clubs(id);

ALTER TABLE stories ADD CONSTRAINT stories_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES creators(id);

ALTER TABLE stories ADD CONSTRAINT stories_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);

ALTER TABLE title_holders ADD CONSTRAINT title_holders_club_id_fkey FOREIGN KEY (club_id) REFERENCES clubs(id);

ALTER TABLE title_holders ADD CONSTRAINT title_holders_competition_id_fkey FOREIGN KEY (competition_id) REFERENCES competitions(id);

ALTER TABLE title_holders ADD CONSTRAINT title_holders_event_id_fkey FOREIGN KEY (event_id) REFERENCES events(id);


-- ========================================================================
-- Indexes (non-constraint)
-- ========================================================================
CREATE INDEX agent_actions_created_idx ON public.agent_actions USING btree (created_at DESC);

CREATE INDEX artifacts_entity_idx ON public.artifacts USING btree (entity_type, entity_id);

CREATE UNIQUE INDEX cth_title_year_uniq ON public.candidate_title_holders USING btree (title_name, year);

CREATE UNIQUE INDEX events_active_unique_name_date_city ON public.events USING btree (lower(name), start_date, lower(COALESCE(city, ''::text))) WHERE active;

CREATE INDEX idx_code_crate ON public.code USING btree (crate);

CREATE INDEX idx_code_path ON public.code USING btree (crate, file_path);

CREATE INDEX idx_doc_archive_slug ON public.document_archive USING btree (slug);

CREATE INDEX idx_doc_archive_version ON public.document_archive USING btree (slug, version);

CREATE INDEX idx_documents_slug ON public.documents USING btree (slug);


-- ========================================================================
-- Views
-- ========================================================================
CREATE OR REPLACE VIEW ai_campaign_summary AS
 SELECT name,
    org,
    description,
    link AS donate_link,
    currency,
    goal,
    raised,
    urgent,
    'https://bearings.lovable.app/campaigns'::text AS bearings_url
   FROM campaigns ca
  WHERE active = true AND privacy_mode = false AND link IS NOT NULL AND link <> '#'::text
  ORDER BY urgent DESC, name;

CREATE OR REPLACE VIEW ai_creator_summary AS
 SELECT name,
    creator_type,
    (city || ', '::text) || country AS location,
    bio,
    website,
    instagram,
    spotify_link,
    'https://bearings.lovable.app/creators/'::text || COALESCE(slug, id::text) AS bearings_url
   FROM creators c
  WHERE active = true AND bear_affiliated = true
  ORDER BY verified DESC NULLS LAST, name;

CREATE OR REPLACE VIEW ai_event_summary AS
 SELECT name,
    (city || ', '::text) || country AS location,
    to_char(start_date::timestamp with time zone, 'FMMonth FMDD YYYY'::text) AS starts,
    to_char(end_date::timestamp with time zone, 'FMMonth FMDD YYYY'::text) AS ends,
    month,
    type,
    size,
        CASE size
            WHEN 'major'::text THEN 'Major international event'::text
            WHEN 'regional'::text THEN 'Regional event'::text
            WHEN 'local'::text THEN 'Local event'::text
            ELSE size
        END AS size_label,
    description,
    link AS tickets_link,
    charity_name,
    host_hotel,
    'https://bearings.lovable.app/events/'::text || COALESCE(slug, id::text) AS bearings_url
   FROM events e
  WHERE active = true AND start_date >= CURRENT_DATE AND (inclusion_flag_codes IS NULL OR NOT inclusion_flag_codes @> ARRAY['criminalized_country'::text])
  ORDER BY start_date;

CREATE OR REPLACE VIEW ai_history_summary AS
 SELECT title,
    year,
    description,
    significance,
    category,
    link,
    'https://bearings.lovable.app/history'::text AS bearings_url
   FROM bear_history bh
  WHERE active = true
  ORDER BY year DESC;

CREATE OR REPLACE VIEW ai_place_summary AS
 SELECT name,
    place_type,
    (city || ', '::text) || country AS location,
    description,
    website,
    booking_link,
    bear_owned,
    bear_popular,
    has_bear_night,
    bear_night_schedule,
    hours_open,
        CASE
            WHEN men_only THEN 'Men only'::text
            WHEN clothing_optional THEN 'Clothing optional'::text
            ELSE NULL::text
        END AS access_note,
    'https://bearings.lovable.app/places/'::text || COALESCE(slug, id::text) AS bearings_url
   FROM places p
  WHERE active = true AND (inclusion_flag_codes IS NULL OR NOT inclusion_flag_codes @> ARRAY['criminalized_country'::text])
  ORDER BY bear_popular DESC NULLS LAST, verified DESC NULLS LAST;

CREATE OR REPLACE VIEW ai_title_summary AS
 SELECT th.holder_name,
    th.title_name,
    comp.scope,
    comp.competition_type,
    th.year,
    (th.city || ', '::text) || th.country AS holder_location,
    th.charity_name,
    th.bio,
    'https://bearings.lovable.app/titles'::text AS bearings_url
   FROM title_holders th
     LEFT JOIN competitions comp ON th.competition_id = comp.id
  WHERE th.active = true
  ORDER BY comp.scope, comp.name;

CREATE OR REPLACE VIEW cause_contributions AS
 SELECT title_holders.charity_name AS cause,
    'titleholder'::text AS source,
    title_holders.holder_name AS who,
    title_holders.year AS yr,
    title_holders.fundraising_total::numeric AS amount,
    title_holders.currency
   FROM title_holders
  WHERE title_holders.charity_name IS NOT NULL AND title_holders.charity_name <> ''::text
UNION ALL
 SELECT events.charity_name AS cause,
    'event'::text AS source,
    events.name AS who,
    EXTRACT(year FROM events.start_date)::integer AS yr,
    NULL::numeric AS amount,
    NULL::text AS currency
   FROM events
  WHERE events.charity_name IS NOT NULL AND events.charity_name <> ''::text
UNION ALL
 SELECT COALESCE(NULLIF(campaigns.cause, ''::text), campaigns.org, campaigns.name) AS cause,
    'campaign'::text AS source,
    COALESCE(campaigns.org, campaigns.name) AS who,
    EXTRACT(year FROM COALESCE(campaigns.ends_at::timestamp with time zone, campaigns.created_at))::integer AS yr,
    campaigns.raised::numeric AS amount,
    campaigns.currency
   FROM campaigns;

CREATE OR REPLACE VIEW charity_impact AS
 SELECT (( SELECT COALESCE(sum(campaigns.raised), 0::bigint) AS "coalesce"
           FROM campaigns
          WHERE campaigns.raised IS NOT NULL)) + (( SELECT COALESCE(sum(title_holders.fundraising_total), 0::bigint) AS "coalesce"
           FROM title_holders
          WHERE title_holders.fundraising_total IS NOT NULL)) AS total_raised,
    ( SELECT count(*) AS count
           FROM campaigns) AS campaigns,
    ( SELECT count(DISTINCT cause_contributions.cause) AS count
           FROM cause_contributions
          WHERE cause_contributions.cause IS NOT NULL AND cause_contributions.cause <> ''::text) AS causes,
    ( SELECT count(*) AS count
           FROM title_holders
          WHERE title_holders.charity_name IS NOT NULL AND title_holders.charity_name <> ''::text) AS pledges;

CREATE OR REPLACE VIEW charity_lineage AS
 SELECT th.charity_name AS cause,
    (array_agg(DISTINCT c.name))[1] AS competition,
    count(*) AS sashes,
    count(DISTINCT th.holder_name) AS people,
    min(th.year) AS first_year,
    max(th.year) AS last_year,
    string_agg(DISTINCT th.holder_name, ', '::text) AS names,
    NULLIF(sum(COALESCE(th.fundraising_total, 0)), 0) AS raised
   FROM title_holders th
     LEFT JOIN competitions c ON c.id = th.competition_id
  WHERE th.charity_name IS NOT NULL AND th.charity_name <> ''::text
  GROUP BY th.charity_name;

CREATE OR REPLACE VIEW competition_history AS
 SELECT comp.name AS competition_name,
    comp.scope,
    comp.competition_type,
    comp.owning_club_id,
    cl.name AS owning_club_name,
    th.year,
    th.holder_name,
    th.country AS holder_country,
    th.city AS holder_city,
    th.active AS is_current_holder,
    th.charity_name,
    th.fundraising_total,
    th.currency,
    th.bio,
    th.inclusion_flag_codes AS holder_flags,
    th.inclusion_notes AS holder_notes,
    comp.inclusion_flag_codes AS competition_flags,
    e.name AS host_event_name,
    e.city AS host_event_city
   FROM title_holders th
     JOIN competitions comp ON th.competition_id = comp.id
     LEFT JOIN clubs cl ON comp.owning_club_id = cl.id
     LEFT JOIN events e ON comp.host_event_id = e.id
  ORDER BY comp.name, th.year DESC;

CREATE OR REPLACE VIEW competitions_missing_holders AS
 SELECT id,
    name,
    scope,
    country,
    city,
    website,
    founded_year
   FROM competitions c
  WHERE active = true AND NOT (EXISTS ( SELECT 1
           FROM title_holders th
          WHERE th.competition_id = c.id OR th.title_name = c.name));

CREATE OR REPLACE VIEW current_title_holders AS
 SELECT th.id,
    th.title_name,
    th.holder_name,
    th.holder_status,
    th.holdover_reason,
        CASE th.holder_status
            WHEN 'active'::holder_status_type THEN th.holder_name
            WHEN 'holdover'::holder_status_type THEN th.holder_name
            WHEN 'unknown'::holder_status_type THEN 'Name not recorded'::text
            WHEN 'vacant'::holder_status_type THEN 'Position vacant'::text
            ELSE th.holder_name
        END AS display_name,
        CASE th.holder_status
            WHEN 'holdover'::holder_status_type THEN 'Extended reign'::text
            WHEN 'unknown'::holder_status_type THEN 'Unknown'::text
            WHEN 'vacant'::holder_status_type THEN 'Vacant'::text
            ELSE NULL::text
        END AS display_status,
    th.year,
    th.club_id,
    th.event_id,
    th.city,
    th.country,
    th.charity_name,
    th.charity_link,
    th.fundraising_total,
    th.currency,
    th.photo_link,
    th.bio,
    th.contact_for_bookings,
    th.active,
    th.inclusion_flag_codes,
    th.inclusion_notes,
    th.created_at,
    th.competition_id,
    c.name AS club_name,
    c.website AS club_website,
    e.name AS crowned_at_event_name,
    e.month AS crowned_at_event_month,
    comp.name AS competition_name,
    comp.scope AS competition_scope,
    comp.competition_type,
    comp.host_event_id AS competition_host_event_id
   FROM title_holders th
     LEFT JOIN clubs c ON th.club_id = c.id
     LEFT JOIN events e ON th.event_id = e.id
     LEFT JOIN competitions comp ON th.competition_id = comp.id
  WHERE th.active = true;

CREATE OR REPLACE VIEW event_backfill_targets AS
 WITH series AS (
         SELECT bearings_series_key(events.name) AS series_key,
            count(*) AS editions
           FROM events
          GROUP BY (bearings_series_key(events.name))
        )
 SELECT s.series_key,
    min(e.name) AS sample_name,
    max(e.city) AS city,
    max(e.country) AS country,
    max(e.link) AS website,
    max(e.start_date) AS known_date
   FROM events e
     JOIN series s ON s.series_key = bearings_series_key(e.name)
  WHERE e.link IS NOT NULL AND e.link <> ''::text AND e.link <> '#'::text AND e.type IS NOT NULL AND e.name !~~* '%cruise%'::text AND e.name !~~* '%sailing%'::text AND s.editions = 1
  GROUP BY s.series_key;

CREATE OR REPLACE VIEW event_dupe_candidates AS
 SELECT a.id AS id_a,
    a.name AS name_a,
    a.city AS city_a,
    a.start_date AS date_a,
    b.id AS id_b,
    b.name AS name_b,
    b.city AS city_b,
    b.start_date AS date_b,
    round(similarity(a.name, b.name)::numeric, 2)::text AS sim
   FROM events a
     JOIN events b ON a.id < b.id AND a.active AND b.active AND similarity(a.name, b.name) > 0.55::double precision AND (a.start_date IS NULL OR b.start_date IS NULL OR abs(a.start_date - b.start_date) <= 21)
  WHERE NOT (EXISTS ( SELECT 1
           FROM event_dupe_ignores i
          WHERE i.lo_id = a.id AND i.hi_id = b.id))
  ORDER BY (round(similarity(a.name, b.name)::numeric, 2)::text) DESC, a.start_date;

CREATE OR REPLACE VIEW event_predictions AS
 WITH proj AS (
         SELECT s.base_key,
            s.city_key,
            s.sample_name,
            s.city,
            s.country,
            s.website,
            s.years_observed,
            s.first_year,
            s.last_year,
            s.typical_doy,
            s.doy_spread,
            (make_date(EXTRACT(year FROM CURRENT_DATE)::integer, 1, 1) + (s.typical_doy - 1)::double precision * '1 day'::interval)::date AS d_this,
            (make_date(EXTRACT(year FROM CURRENT_DATE)::integer + 1, 1, 1) + (s.typical_doy - 1)::double precision * '1 day'::interval)::date AS d_next
           FROM event_series s
        ), pick AS (
         SELECT proj.base_key,
            proj.city_key,
            proj.sample_name,
            proj.city,
            proj.country,
            proj.website,
            proj.years_observed,
            proj.first_year,
            proj.last_year,
            proj.typical_doy,
            proj.doy_spread,
            proj.d_this,
            proj.d_next,
                CASE
                    WHEN proj.d_this >= CURRENT_DATE AND proj.d_this < (CURRENT_DATE + '1 year'::interval) THEN proj.d_this
                    WHEN proj.d_next >= CURRENT_DATE AND proj.d_next < (CURRENT_DATE + '1 year'::interval) THEN proj.d_next
                    ELSE NULL::date
                END AS predicted_date
           FROM proj
        )
 SELECT base_key,
    city_key,
    sample_name,
    city,
    country,
    website,
    years_observed,
    last_year,
    typical_doy,
    doy_spread,
    predicted_date,
        CASE
            WHEN years_observed >= 3 AND doy_spread <= 24 THEN 'high'::text
            WHEN years_observed >= 2 THEN 'medium'::text
            ELSE 'low'::text
        END AS confidence
   FROM pick p
  WHERE predicted_date IS NOT NULL AND last_year >= (EXTRACT(year FROM CURRENT_DATE)::integer - 2) AND NOT (EXISTS ( SELECT 1
           FROM events e
          WHERE e.active AND e.start_date >= CURRENT_DATE AND e.start_date < (CURRENT_DATE + '1 year'::interval) AND bearings_series_key(e.name) = p.base_key AND lower(COALESCE(e.city, ''::text)) = p.city_key));

CREATE OR REPLACE VIEW event_series AS
 WITH norm AS (
         SELECT events.id,
            events.name,
            events.city,
            events.country,
            events.start_date,
            events.link,
            EXTRACT(year FROM events.start_date)::integer AS yr,
            EXTRACT(doy FROM events.start_date)::integer AS doy,
            bearings_series_key(events.name) AS base_key,
            lower(COALESCE(events.city, ''::text)) AS city_key
           FROM events
          WHERE events.start_date IS NOT NULL
        )
 SELECT base_key,
    city_key,
    (array_agg(name ORDER BY start_date DESC))[1] AS sample_name,
    (array_agg(city ORDER BY start_date DESC))[1] AS city,
    (array_agg(country ORDER BY start_date DESC))[1] AS country,
    (array_agg(link ORDER BY start_date DESC) FILTER (WHERE link IS NOT NULL AND link <> ''::text AND link <> '#'::text))[1] AS website,
    count(DISTINCT yr) AS years_observed,
    min(yr) AS first_year,
    max(yr) AS last_year,
    percentile_cont(0.5::double precision) WITHIN GROUP (ORDER BY (doy::double precision))::integer AS typical_doy,
    max(doy) - min(doy) AS doy_spread
   FROM norm
  WHERE base_key <> ''::text
  GROUP BY base_key, city_key
 HAVING count(DISTINCT yr) >= 2;

CREATE OR REPLACE VIEW events_with_flags AS
 SELECT id,
    name,
    city,
    country,
    lat,
    lng,
    start_date,
    end_date,
    month,
    type,
    size,
    hot,
    link,
    tags,
    description,
    going,
    active,
    created_at,
    updated_at,
    source,
    source_id,
    charity_name,
    charity_link,
    host_hotel,
    host_hotel_link,
    inclusion_flag_codes,
    inclusion_notes,
        CASE
            WHEN inclusion_flag_codes IS NULL OR array_length(inclusion_flag_codes, 1) IS NULL THEN false
            ELSE true
        END AS has_flags,
    array_length(inclusion_flag_codes, 1) AS flag_count
   FROM events e
  WHERE active = true;

CREATE OR REPLACE VIEW places_near_events AS
 SELECT p.id,
    p.name,
    p.place_type,
    p.can_stay,
    p.hours_type,
    p.city,
    p.country,
    p.lat,
    p.lng,
    p.address,
    p.website,
    p.booking_link,
    p.misterbnb_link,
    p.facebook,
    p.instagram,
    p.ical_url,
    p.description,
    p.tags,
    p.bear_owned,
    p.bear_welcoming,
    p.bear_popular,
    p.clothing_optional,
    p.men_only,
    p.price_range,
    p.brunch_available,
    p.cuisine_type,
    p.active,
    p.verified,
    p.closed_year,
    p.contact_email,
    p.created_at,
    p.updated_at,
    p.source,
    p.inclusion_flag_codes,
    p.inclusion_notes,
    p.inclusive_alternative_id,
    epl.event_id,
    epl.note AS place_note,
    epl.official,
    epl.relationship,
    epl.distance_km,
    e.name AS event_name,
    e.month AS event_month
   FROM places p
     JOIN event_place_links epl ON p.id = epl.place_id
     JOIN events e ON epl.event_id = e.id
  WHERE p.active = true AND e.active = true;

CREATE OR REPLACE VIEW places_with_flags AS
 SELECT p.id,
    p.name,
    p.place_type,
    p.can_stay,
    p.hours_type,
    p.city,
    p.country,
    p.lat,
    p.lng,
    p.address,
    p.website,
    p.booking_link,
    p.misterbnb_link,
    p.facebook,
    p.instagram,
    p.ical_url,
    p.description,
    p.tags,
    p.bear_owned,
    p.bear_welcoming,
    p.bear_popular,
    p.clothing_optional,
    p.men_only,
    p.price_range,
    p.brunch_available,
    p.cuisine_type,
    p.active,
    p.verified,
    p.closed_year,
    p.contact_email,
    p.created_at,
    p.updated_at,
    p.source,
    p.inclusion_flag_codes,
    p.inclusion_notes,
    p.inclusive_alternative_id,
    alt.name AS inclusive_alternative_name,
    alt.website AS inclusive_alternative_link,
        CASE
            WHEN p.inclusion_flag_codes IS NULL OR array_length(p.inclusion_flag_codes, 1) IS NULL THEN false
            ELSE true
        END AS has_flags
   FROM places p
     LEFT JOIN places alt ON p.inclusive_alternative_id = alt.id
  WHERE p.active = true;

CREATE OR REPLACE VIEW titleholder_lineage_status AS
 SELECT title_name,
    max(competition_id) AS competition_id,
    max(country) AS country,
    min(year) AS first_year,
    max(year) AS last_year,
    count(*) AS holders,
    array_agg(DISTINCT year ORDER BY year) AS held_years
   FROM title_holders
  WHERE year IS NOT NULL
  GROUP BY title_name;


-- ========================================================================
-- Functions
-- ========================================================================
CREATE OR REPLACE FUNCTION public.bearings_series_key(p text)
 RETURNS text
 LANGUAGE sql
 IMMUTABLE
AS $function$
  SELECT btrim(coalesce(string_agg(tok, ' ' ORDER BY tok), ''))
  FROM unnest(string_to_array(
    btrim(regexp_replace(
      regexp_replace(lower(coalesce(p,'')), '\m(19|20)[0-9]{2}\M', '', 'g'),
      '[^a-z0-9]+', ' ', 'g')), ' ')) AS tok
  WHERE tok <> '';
$function$
;

CREATE OR REPLACE FUNCTION public.coming_up(input_lat double precision DEFAULT NULL::double precision, input_lng double precision DEFAULT NULL::double precision, radius_km double precision DEFAULT 500, season text DEFAULT NULL::text, from_date date DEFAULT NULL::date, to_date date DEFAULT NULL::date, event_type text DEFAULT NULL::text, country text DEFAULT NULL::text, max_rows integer DEFAULT 30)
 RETURNS json
 LANGUAGE plpgsql
 STABLE SECURITY DEFINER
AS $function$
DECLARE
    v_from    date;
    v_to      date;
    v_events  json;
    v_venues  json;
    v_clubs   json;
    v_country text;
BEGIN
    v_country := country;

    IF from_date IS NOT NULL THEN
        v_from := from_date;
        v_to   := COALESCE(to_date, from_date + interval '90 days');
    ELSE
        CASE season
            WHEN 'spring' THEN v_from := make_date(extract(year FROM CURRENT_DATE)::int, 3, 1);  v_to := make_date(extract(year FROM CURRENT_DATE)::int, 5, 31);
            WHEN 'summer' THEN v_from := make_date(extract(year FROM CURRENT_DATE)::int, 6, 1);  v_to := make_date(extract(year FROM CURRENT_DATE)::int, 8, 31);
            WHEN 'autumn' THEN v_from := make_date(extract(year FROM CURRENT_DATE)::int, 9, 1);  v_to := make_date(extract(year FROM CURRENT_DATE)::int, 11, 30);
            WHEN 'winter' THEN v_from := make_date(extract(year FROM CURRENT_DATE)::int, 12, 1); v_to := make_date(extract(year FROM CURRENT_DATE)::int + 1, 2, 28);
            ELSE v_from := CURRENT_DATE; v_to := CURRENT_DATE + interval '6 months';
        END CASE;
        IF v_from < CURRENT_DATE THEN v_from := CURRENT_DATE; END IF;
    END IF;

    IF input_lat IS NOT NULL AND input_lng IS NOT NULL THEN
        SELECT json_agg(r ORDER BY distance_km, start_date) INTO v_events FROM (
            SELECT id, name, city, e.country, start_date, end_date, type, size,
                   hot, link, description, slug, event_mode, inclusion_flag_codes,
                   round((6371 * acos(
                       cos(radians(input_lat)) * cos(radians(lat::float8))
                       * cos(radians(lng::float8) - radians(input_lng))
                       + sin(radians(input_lat)) * sin(radians(lat::float8))
                   ))::numeric, 1)::float8 AS distance_km
            FROM events e
            WHERE e.active = true AND e.start_date >= v_from AND e.start_date <= v_to
              AND e.lat IS NOT NULL AND e.lng IS NOT NULL
              AND (event_type IS NULL OR e.type = event_type)
              AND (v_country IS NULL OR e.country = v_country)
              AND (6371 * acos(
                  cos(radians(input_lat)) * cos(radians(e.lat::float8))
                  * cos(radians(e.lng::float8) - radians(input_lng))
                  + sin(radians(input_lat)) * sin(radians(e.lat::float8))
              )) <= radius_km
            LIMIT max_rows
        ) r;

        SELECT json_agg(r ORDER BY r.bear_popular DESC NULLS LAST, distance_km) INTO v_venues FROM (
            SELECT id, name, place_type, city, p.country, lat, lng,
                   description, website, bear_night_schedule, has_bear_night,
                   men_only, booking_link, slug, bear_popular,
                   round((6371 * acos(
                       cos(radians(input_lat)) * cos(radians(p.lat::float8))
                       * cos(radians(p.lng::float8) - radians(input_lng))
                       + sin(radians(input_lat)) * sin(radians(p.lat::float8))
                   ))::numeric, 1)::float8 AS distance_km
            FROM places p
            WHERE p.active = true AND p.lat IS NOT NULL AND p.lng IS NOT NULL
              AND (6371 * acos(
                  cos(radians(input_lat)) * cos(radians(p.lat::float8))
                  * cos(radians(p.lng::float8) - radians(input_lng))
                  + sin(radians(input_lat)) * sin(radians(p.lat::float8))
              )) <= LEAST(radius_km, 100)
            LIMIT 15
        ) r;
    ELSE
        SELECT json_agg(r ORDER BY start_date) INTO v_events FROM (
            SELECT id, name, city, e.country, start_date, end_date, type, size,
                   hot, link, description, slug, event_mode, inclusion_flag_codes
            FROM events e
            WHERE e.active = true AND e.start_date >= v_from AND e.start_date <= v_to
              AND (event_type IS NULL OR e.type = event_type)
              AND (v_country IS NULL OR e.country = v_country)
            LIMIT max_rows
        ) r;

        IF v_country IS NOT NULL THEN
            SELECT json_agg(r ORDER BY r.bear_popular DESC NULLS LAST, city) INTO v_venues FROM (
                SELECT id, name, place_type, city, p.country, description,
                       website, bear_night_schedule, has_bear_night, men_only, booking_link, slug,
                       bear_popular
                FROM places p
                WHERE p.active = true AND p.country = v_country
                LIMIT 15
            ) r;
        ELSE
            v_venues := '[]'::json;
        END IF;
    END IF;

    v_country := COALESCE(v_country, (
        SELECT e.country FROM events e
        WHERE e.active = true AND e.start_date >= v_from AND e.start_date <= v_to
        GROUP BY e.country ORDER BY COUNT(*) DESC LIMIT 1
    ));

    IF v_country IS NOT NULL THEN
        SELECT json_agg(r ORDER BY name) INTO v_clubs FROM (
            SELECT id, name, city, c.country, website, description, contact_email
            FROM clubs c
            WHERE c.active = true AND c.country = v_country
            LIMIT 10
        ) r;
    ELSE
        v_clubs := '[]'::json;
    END IF;

    RETURN json_build_object(
        'events',        COALESCE(v_events, '[]'::json),
        'venues',        COALESCE(v_venues, '[]'::json),
        'clubs',         COALESCE(v_clubs, '[]'::json),
        'window_from',   v_from,
        'window_to',     v_to,
        'season',        season,
        'location_used', (input_lat IS NOT NULL)
    );
END;
$function$
;

CREATE OR REPLACE FUNCTION public.events_nearby(input_lat double precision, input_lng double precision, radius_km double precision DEFAULT 500, from_date date DEFAULT CURRENT_DATE, to_date date DEFAULT (CURRENT_DATE + '90 days'::interval), max_rows integer DEFAULT 20)
 RETURNS TABLE(id bigint, name text, city text, country text, lat numeric, lng numeric, start_date date, end_date date, month text, type text, size text, hot boolean, link text, description text, slug text, event_mode text, inclusion_flag_codes text[], distance_km double precision)
 LANGUAGE sql
 STABLE
AS $function$
    SELECT
        id, name, city, country, lat, lng,
        start_date, end_date, month, type, size, hot,
        link, description, slug, event_mode, inclusion_flag_codes,
        round((6371 * acos(
            cos(radians(input_lat)) * cos(radians(lat::float8))
            * cos(radians(lng::float8) - radians(input_lng))
            + sin(radians(input_lat)) * sin(radians(lat::float8))
        ))::numeric, 1)::float8 AS distance_km
    FROM events
    WHERE active = true
      AND start_date >= from_date
      AND start_date <= to_date
      AND lat IS NOT NULL AND lng IS NOT NULL
      AND (
          6371 * acos(
              cos(radians(input_lat)) * cos(radians(lat::float8))
              * cos(radians(lng::float8) - radians(input_lng))
              + sin(radians(input_lat)) * sin(radians(lat::float8))
          )
      ) <= radius_km
    ORDER BY distance_km ASC, start_date ASC
    LIMIT max_rows;
$function$
;

CREATE OR REPLACE FUNCTION public.gin_extract_query_trgm(text, internal, smallint, internal, internal, internal, internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gin_extract_query_trgm$function$
;

CREATE OR REPLACE FUNCTION public.gin_extract_value_trgm(text, internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gin_extract_value_trgm$function$
;

CREATE OR REPLACE FUNCTION public.gin_trgm_consistent(internal, smallint, text, integer, internal, internal, internal, internal)
 RETURNS boolean
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gin_trgm_consistent$function$
;

CREATE OR REPLACE FUNCTION public.gin_trgm_triconsistent(internal, smallint, text, integer, internal, internal, internal)
 RETURNS "char"
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gin_trgm_triconsistent$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_compress(internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_compress$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_consistent(internal, text, smallint, oid, internal)
 RETURNS boolean
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_consistent$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_decompress(internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_decompress$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_distance(internal, text, smallint, oid, internal)
 RETURNS double precision
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_distance$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_in(cstring)
 RETURNS gtrgm
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_in$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_options(internal)
 RETURNS void
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE
AS '$libdir/pg_trgm', $function$gtrgm_options$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_out(gtrgm)
 RETURNS cstring
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_out$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_penalty(internal, internal, internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_penalty$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_picksplit(internal, internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_picksplit$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_same(gtrgm, gtrgm, internal)
 RETURNS internal
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_same$function$
;

CREATE OR REPLACE FUNCTION public.gtrgm_union(internal, internal)
 RETURNS gtrgm
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$gtrgm_union$function$
;

CREATE OR REPLACE FUNCTION public.increment_future_idea_upvotes(idea_id bigint)
 RETURNS bigint
 LANGUAGE sql
 SECURITY DEFINER
 SET search_path TO 'public'
AS $function$
  UPDATE future_ideas
  SET upvotes = COALESCE(upvotes, 0) + 1
  WHERE id = idea_id AND active = true
  RETURNING upvotes;
$function$
;

CREATE OR REPLACE FUNCTION public.increment_proposal_vote_count()
 RETURNS trigger
 LANGUAGE plpgsql
 SECURITY DEFINER
AS $function$
BEGIN
    IF NEW.vote = 'yes' THEN
        UPDATE bear_future_proposals
        SET vote_yes = COALESCE(vote_yes, 0) + COALESCE(NEW.vote_weight, 1)
        WHERE id = NEW.proposal_id;
    ELSIF NEW.vote = 'no' THEN
        UPDATE bear_future_proposals
        SET vote_no = COALESCE(vote_no, 0) + COALESCE(NEW.vote_weight, 1)
        WHERE id = NEW.proposal_id;
    END IF;
    -- abstain: no count change, but vote is recorded
    RETURN NEW;
END;
$function$
;

CREATE OR REPLACE FUNCTION public.increment_revival_votes(p_kind text, p_id bigint)
 RETURNS bigint
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public'
AS $function$
DECLARE v bigint;
BEGIN
  IF p_kind = 'place' THEN
    UPDATE places SET revival_votes = COALESCE(revival_votes,0)+1 WHERE id=p_id RETURNING revival_votes INTO v;
  ELSIF p_kind = 'club' THEN
    UPDATE clubs SET revival_votes = COALESCE(revival_votes,0)+1 WHERE id=p_id RETURNING revival_votes INTO v;
  END IF;
  RETURN v;
END $function$
;

CREATE OR REPLACE FUNCTION public.now_feed(input_lat double precision DEFAULT NULL::double precision, input_lng double precision DEFAULT NULL::double precision, radius_km double precision DEFAULT 500)
 RETURNS json
 LANGUAGE plpgsql
 STABLE SECURITY DEFINER
AS $function$
DECLARE
    hot_events     json;
    nearby_venues  json;
    campaigns      json;
    titles         json;
    from_date      date := CURRENT_DATE;
    to_date        date := CURRENT_DATE + interval '30 days';
BEGIN
    IF input_lat IS NOT NULL AND input_lng IS NOT NULL THEN
        SELECT json_agg(r) INTO hot_events FROM (
            SELECT id, name, city, country, start_date, end_date, type, hot,
                   link, description, slug, inclusion_flag_codes,
                   round((6371 * acos(
                       cos(radians(input_lat)) * cos(radians(lat::float8))
                       * cos(radians(lng::float8) - radians(input_lng))
                       + sin(radians(input_lat)) * sin(radians(lat::float8))
                   ))::numeric, 1) AS distance_km
            FROM events
            WHERE active = true
              AND start_date >= from_date AND start_date <= to_date
              AND lat IS NOT NULL AND lng IS NOT NULL
            ORDER BY
                CASE WHEN hot THEN 0 ELSE 1 END,
                distance_km ASC,
                start_date ASC
            LIMIT 10
        ) r;

        SELECT json_agg(r) INTO nearby_venues FROM (
            SELECT id, name, place_type, city, country, lat, lng,
                   description, website, bear_night_schedule, has_bear_night,
                   men_only, booking_link, slug, inclusion_flag_codes,
                   round((6371 * acos(
                       cos(radians(input_lat)) * cos(radians(lat::float8))
                       * cos(radians(lng::float8) - radians(input_lng))
                       + sin(radians(input_lat)) * sin(radians(lat::float8))
                   ))::numeric, 1) AS distance_km
            FROM places
            WHERE active = true
              AND lat IS NOT NULL AND lng IS NOT NULL
              AND (6371 * acos(
                  cos(radians(input_lat)) * cos(radians(lat::float8))
                  * cos(radians(lng::float8) - radians(input_lng))
                  + sin(radians(input_lat)) * sin(radians(lat::float8))
              )) <= radius_km
            ORDER BY
                CASE WHEN bear_popular THEN 0 ELSE 1 END,
                distance_km ASC
            LIMIT 10
        ) r;
    ELSE
        SELECT json_agg(r) INTO hot_events FROM (
            SELECT id, name, city, country, start_date, end_date, type, hot,
                   link, description, slug, inclusion_flag_codes
            FROM events
            WHERE active = true
              AND start_date >= from_date AND start_date <= to_date
            ORDER BY
                CASE WHEN hot THEN 0 ELSE 1 END,
                start_date ASC
            LIMIT 10
        ) r;
        nearby_venues := '[]'::json;
    END IF;

    SELECT json_agg(r) INTO campaigns FROM (
        SELECT id, name, org, description, link, goal, raised, currency
        FROM campaigns
        WHERE active = true AND privacy_mode = false
        ORDER BY name
    ) r;

    -- Title holders: now includes holder_status and display_status
    SELECT json_agg(r) INTO titles FROM (
        SELECT id, title_name, holder_name, display_name, holder_status,
               display_status, holdover_reason, year, city, country,
               competition_name, competition_scope
        FROM current_title_holders
        ORDER BY title_name
    ) r;

    RETURN json_build_object(
        'hot_events',       COALESCE(hot_events, '[]'::json),
        'nearby_venues',    COALESCE(nearby_venues, '[]'::json),
        'active_campaigns', COALESCE(campaigns, '[]'::json),
        'current_titles',   COALESCE(titles, '[]'::json),
        'as_of',            CURRENT_DATE,
        'location_used',    (input_lat IS NOT NULL)
    );
END;
$function$
;

CREATE OR REPLACE FUNCTION public.set_limit(real)
 RETURNS real
 LANGUAGE c
 STRICT
AS '$libdir/pg_trgm', $function$set_limit$function$
;

CREATE OR REPLACE FUNCTION public.show_limit()
 RETURNS real
 LANGUAGE c
 STABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$show_limit$function$
;

CREATE OR REPLACE FUNCTION public.show_trgm(text)
 RETURNS text[]
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$show_trgm$function$
;

CREATE OR REPLACE FUNCTION public.similarity(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$similarity$function$
;

CREATE OR REPLACE FUNCTION public.similarity_dist(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$similarity_dist$function$
;

CREATE OR REPLACE FUNCTION public.similarity_op(text, text)
 RETURNS boolean
 LANGUAGE c
 STABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$similarity_op$function$
;

CREATE OR REPLACE FUNCTION public.strict_word_similarity(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$strict_word_similarity$function$
;

CREATE OR REPLACE FUNCTION public.strict_word_similarity_commutator_op(text, text)
 RETURNS boolean
 LANGUAGE c
 STABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$strict_word_similarity_commutator_op$function$
;

CREATE OR REPLACE FUNCTION public.strict_word_similarity_dist_commutator_op(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$strict_word_similarity_dist_commutator_op$function$
;

CREATE OR REPLACE FUNCTION public.strict_word_similarity_dist_op(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$strict_word_similarity_dist_op$function$
;

CREATE OR REPLACE FUNCTION public.strict_word_similarity_op(text, text)
 RETURNS boolean
 LANGUAGE c
 STABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$strict_word_similarity_op$function$
;

CREATE OR REPLACE FUNCTION public.word_similarity(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$word_similarity$function$
;

CREATE OR REPLACE FUNCTION public.word_similarity_commutator_op(text, text)
 RETURNS boolean
 LANGUAGE c
 STABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$word_similarity_commutator_op$function$
;

CREATE OR REPLACE FUNCTION public.word_similarity_dist_commutator_op(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$word_similarity_dist_commutator_op$function$
;

CREATE OR REPLACE FUNCTION public.word_similarity_dist_op(text, text)
 RETURNS real
 LANGUAGE c
 IMMUTABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$word_similarity_dist_op$function$
;

CREATE OR REPLACE FUNCTION public.word_similarity_op(text, text)
 RETURNS boolean
 LANGUAGE c
 STABLE PARALLEL SAFE STRICT
AS '$libdir/pg_trgm', $function$word_similarity_op$function$
;


-- ========================================================================
-- Row-Level Security (enable)
-- ========================================================================
ALTER TABLE agent_inbox ENABLE ROW LEVEL SECURITY;

ALTER TABLE agent_posts ENABLE ROW LEVEL SECURITY;

ALTER TABLE artifacts ENABLE ROW LEVEL SECURITY;

ALTER TABLE bear_future_proposals ENABLE ROW LEVEL SECURITY;

ALTER TABLE bear_history ENABLE ROW LEVEL SECURITY;

ALTER TABLE bear_regions ENABLE ROW LEVEL SECURITY;

ALTER TABLE campaigns ENABLE ROW LEVEL SECURITY;

ALTER TABLE candidate_events ENABLE ROW LEVEL SECURITY;

ALTER TABLE clubs ENABLE ROW LEVEL SECURITY;

ALTER TABLE code ENABLE ROW LEVEL SECURITY;

ALTER TABLE competitions ENABLE ROW LEVEL SECURITY;

ALTER TABLE creator_event_links ENABLE ROW LEVEL SECURITY;

ALTER TABLE creators ENABLE ROW LEVEL SECURITY;

ALTER TABLE digital_space_event_links ENABLE ROW LEVEL SECURITY;

ALTER TABLE digital_spaces ENABLE ROW LEVEL SECURITY;

ALTER TABLE document_archive ENABLE ROW LEVEL SECURITY;

ALTER TABLE documents ENABLE ROW LEVEL SECURITY;

ALTER TABLE event_place_links ENABLE ROW LEVEL SECURITY;

ALTER TABLE events ENABLE ROW LEVEL SECURITY;

ALTER TABLE future_ideas ENABLE ROW LEVEL SECURITY;

ALTER TABLE inclusion_flags ENABLE ROW LEVEL SECURITY;

ALTER TABLE kindred_sources ENABLE ROW LEVEL SECURITY;

ALTER TABLE media ENABLE ROW LEVEL SECURITY;

ALTER TABLE newsletter_subscribers ENABLE ROW LEVEL SECURITY;

ALTER TABLE operating_costs ENABLE ROW LEVEL SECURITY;

ALTER TABLE operational_ledger ENABLE ROW LEVEL SECURITY;

ALTER TABLE places ENABLE ROW LEVEL SECURITY;

ALTER TABLE platform_settings ENABLE ROW LEVEL SECURITY;

ALTER TABLE sponsor_event_links ENABLE ROW LEVEL SECURITY;

ALTER TABLE sponsors ENABLE ROW LEVEL SECURITY;

ALTER TABLE stores ENABLE ROW LEVEL SECURITY;

ALTER TABLE stories ENABLE ROW LEVEL SECURITY;

ALTER TABLE submissions ENABLE ROW LEVEL SECURITY;

ALTER TABLE title_holders ENABLE ROW LEVEL SECURITY;

ALTER TABLE translations ENABLE ROW LEVEL SECURITY;

ALTER TABLE user_preferences ENABLE ROW LEVEL SECURITY;

ALTER TABLE watched_feeds ENABLE ROW LEVEL SECURITY;


-- ========================================================================
-- Policies
-- ========================================================================
CREATE POLICY "No public read agent_inbox" ON agent_inbox AS PERMISSIVE FOR SELECT TO public USING (false);

CREATE POLICY "Public insert newsletter" ON newsletter_subscribers AS PERMISSIVE FOR INSERT TO public WITH CHECK (true);

CREATE POLICY "Public insert submissions" ON submissions AS PERMISSIVE FOR INSERT TO public WITH CHECK (true);

CREATE POLICY "Public insert translation suggestions" ON translations AS PERMISSIVE FOR INSERT TO public WITH CHECK ((status = 'pending_review'::text));

CREATE POLICY "Public insert user_preferences" ON user_preferences AS PERMISSIVE FOR INSERT TO public WITH CHECK (true);

CREATE POLICY "Public read agent_posts" ON agent_posts AS PERMISSIVE FOR SELECT TO public USING ((status = 'published'::text));

CREATE POLICY "Public read approved translations" ON translations AS PERMISSIVE FOR SELECT TO public USING ((status = 'approved'::text));

CREATE POLICY "Public read bear_future_proposals" ON bear_future_proposals AS PERMISSIVE FOR SELECT TO public USING (((active = true) OR (active IS NULL)));

CREATE POLICY "Public read bear_history" ON bear_history AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read campaigns" ON campaigns AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read clubs" ON clubs AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read code" ON code AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read competitions" ON competitions AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read creator_event_links" ON creator_event_links AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read creators" ON creators AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read digital_space_event_links" ON digital_space_event_links AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read digital_spaces" ON digital_spaces AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read document_archive" ON document_archive AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read documents" ON documents AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read event_place_links" ON event_place_links AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read events" ON events AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read inclusion_flags" ON inclusion_flags AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read media" ON media AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read operational_ledger" ON operational_ledger AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read places" ON places AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read platform_settings" ON platform_settings AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read sponsor_event_links" ON sponsor_event_links AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read sponsors" ON sponsors AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read stores" ON stores AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY "Public read stories" ON stories AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read title_holders" ON title_holders AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public read user_preferences" ON user_preferences AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "Public update user_preferences" ON user_preferences AS PERMISSIVE FOR UPDATE TO public USING (true);

CREATE POLICY "artifacts public read" ON artifacts AS PERMISSIVE FOR SELECT TO anon, authenticated USING (active);

CREATE POLICY bear_regions_public_read ON bear_regions AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY kindred_sources_public_read ON kindred_sources AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY operating_costs_public_read ON operating_costs AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "public read candidates" ON candidate_events AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY "public read watched_feeds" ON watched_feeds AS PERMISSIVE FOR SELECT TO public USING (true);

CREATE POLICY public_read ON future_ideas AS PERMISSIVE FOR SELECT TO public USING ((active = true));

CREATE POLICY public_upvote ON future_ideas AS PERMISSIVE FOR UPDATE TO public USING ((active = true)) WITH CHECK ((active = true));

