
-- deploy/sql/places_nearby.sql
-- Run this in the Supabase SQL editor to enable /api/places/nearby.
-- Uses the Haversine formula to find places within a radius.
--
-- After deployment, test with:
-- SELECT * FROM places_nearby(45.4215, -75.6972, 100);  -- 100km from Ottawa

CREATE OR REPLACE FUNCTION places_nearby(
    input_lat float8,
    input_lng float8,
    radius_km float8 DEFAULT 50
)
RETURNS SETOF places
LANGUAGE sql
STABLE
AS $$
    SELECT *
    FROM places
    WHERE active = true
      AND lat IS NOT NULL
      AND lng IS NOT NULL
      AND (
        6371 * acos(
          cos(radians(input_lat))
          * cos(radians(lat))
          * cos(radians(lng) - radians(input_lng))
          + sin(radians(input_lat))
          * sin(radians(lat))
        )
      ) <= radius_km
    ORDER BY (
        6371 * acos(
          cos(radians(input_lat))
          * cos(radians(lat))
          * cos(radians(lng) - radians(input_lng))
          + sin(radians(input_lat))
          * sin(radians(lat))
        )
    ) ASC;
$$;

-- Grant access to the anon role so the public API can call it
GRANT EXECUTE ON FUNCTION places_nearby TO anon;
