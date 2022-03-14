-- Add migration script here
CREATE TABLE IF NOT EXISTS premium_slots (
    id bigserial PRIMARY KEY,
    title text NOT NULL,
    user_id bigint,
    message text NOT NULL,
    source text NOT NULL,
    source_id text NOT NULL,
    tier int NOT NULL,
    state int NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    manage_url text NOT NULL,
    attached_guild_id bigint
);

CREATE INDEX IF NOT EXISTS premium_slots_guild_id_idx ON premium_slots (attached_guild_id);

CREATE INDEX IF NOT EXISTS premium_slots_user_id_idx ON premium_slots (user_id);

CREATE UNIQUE INDEX IF NOT EXISTS premium_slots_source_id_idx ON premium_slots (source, source_id);

