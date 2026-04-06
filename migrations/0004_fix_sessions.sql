-- 0003 created public.tower_sessions which does not match the default table used
-- by tower-sessions-sqlx-store (schema: tower_sessions, table: session).
-- Drop the old table and create the correct schema/table.

DROP TABLE IF EXISTS public.tower_sessions;

CREATE SCHEMA IF NOT EXISTS tower_sessions;

CREATE TABLE IF NOT EXISTS tower_sessions.session (
    id          TEXT PRIMARY KEY,
    data        BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);
