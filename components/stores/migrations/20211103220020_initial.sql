-- Add migration script here
CREATE TABLE IF NOT EXISTS joined_guilds (
    id bigint PRIMARY KEY,
    name text NOT NULL,
    icon text NOT NULL,
    owner_id bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS guild_scripts (
    id bigserial PRIMARY KEY,
    guild_id bigint NOT NULL,
    name text NOT NULL,
    original_source text NOT NULL,
    enabled boolean NOT NULL,
    UNIQUE (guild_id, name)
);

CREATE INDEX IF NOT EXISTS guild_scripts_guild_id_name_idx ON guild_scripts (guild_id, name);

CREATE TABLE IF NOT EXISTS guild_meta_configs (
    guild_id bigserial PRIMARY KEY,
    error_channel_id bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS discord_oauth_tokens (
    user_id bigint PRIMARY KEY,
    discord_bearer_token text NOT NULL,
    discord_refresh_token text NOT NULL,
    discord_token_expires_at timestamp with time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS web_sessions (
    token text PRIMARY KEY,
    kind smallint NOT NULL,
    user_id bigint NOT NULL REFERENCES discord_oauth_tokens (user_id) ON DELETE CASCADE,
    discriminator smallint NOT NULL,
    username text NOT NULL,
    avatar text NOT NULL,
    created_at timestamp with time zone NOT NULL
);

