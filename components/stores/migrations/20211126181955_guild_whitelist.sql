-- Add migration script here
CREATE TABLE IF NOT EXISTS guild_whitelist (
    guild_id bigint PRIMARY KEY NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

