-- Add migration script here
ALTER TABLE plugins
    ADD COLUMN plugin_kind smallint NOT NULL DEFAULT 0;

ALTER TABLE plugins
    ADD COLUMN current_version_number int NOT NULL DEFAULT 0;

ALTER TABLE plugins
    ADD COLUMN script_published_source text;

ALTER TABLE plugins
    ADD COLUMN script_published_version_updated_at timestamptz;

ALTER TABLE plugins
    ADD COLUMN script_dev_source text;

ALTER TABLE plugins
    ADD COLUMN script_dev_version_updated_at timestamptz;

ALTER TABLE plugins
    DROP COLUMN author_id;

ALTER TABLE plugins
    ADD COLUMN author_id bigint NOT NULL;

ALTER TABLE plugins
    ADD COLUMN is_public boolean NOT NULL;

ALTER TABLE plugins RENAME COLUMN short_desc TO short_description;

ALTER TABLE guild_scripts
    ADD COLUMN plugin_id bigint REFERENCES plugins (id);

ALTER TABLE guild_scripts
    ADD COLUMN plugin_auto_update boolean;

DROP TABLE users;

CREATE TABLE user_meta (
    discord_user_id bigint PRIMARY KEY,
    is_admin boolean NOT NULL,
    is_moderator boolean NOT NULL,
    is_verified boolean NOT NULL
);

