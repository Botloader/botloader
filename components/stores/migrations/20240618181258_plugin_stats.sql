ALTER TABLE plugins
    ADD COLUMN installed_guilds integer;

ALTER TABLE plugins
    ADD COLUMN installed_guilds_updated_at timestamp with time zone;

ALTER TABLE plugins
    ADD COLUMN discord_thread_id bigint;

