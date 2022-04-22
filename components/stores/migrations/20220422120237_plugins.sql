-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id bigint NOT NULL PRIMARY KEY,
    username text NOT NULL,
    discriminator smallint NOT NULL,
    avatar: text NOT NULL,
    is_dev boolean NOT NULL,
    is_subscriber boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS plugins (
    id bigserial NOT NULL PRIMARY KEY,
    created_at timestamp with time zone NOT NULL,
    author_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL,
    short_desc text NOT NULL,
    long_description text NOT NULL,
    is_published boolean NOT NULL,
    is_official boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS plugin_versions (
    plugin_id bigint NOT NULL REFERENCES plugins (id) ON DELETE CASCADE,
    created_at timestamp with time zone NOT NULL,
    kind smallint NOT NULL,
    data jsonb NOT NULL,
    version_major smallint NOT NULL,
    version_minor smallint NOT NULL,
    PRIMARY KEY (plugin_id, version_major, version_minor),
);

CREATE TABLE IF NOT EXISTS guild_plugin_subscriptions (
    guild_id bigint NOT NULL,
    plugin_id bigint NOT NULL REFERENCES plugins (id) ON DELETE CASCADE,
    pinned_version_major bigint,
    pinned_version_minor bigint,
    use_latest_stable boolean NOT NULL,
    use_devel boolean NOT NULL,
    PRIMARY KEY (guild_id, plugin_id)
);

