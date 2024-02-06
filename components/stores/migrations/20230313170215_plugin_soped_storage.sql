-- Add migration script here
-- This migration adds plugin_id to scheduled_tasks and bucket_store
--
-- scheduled_tasks
ALTER TABLE scheduled_tasks
    ADD COLUMN plugin_id bigint NOT NULL;

DROP INDEX scheduled_tasks_unique_key_idx;

CREATE UNIQUE INDEX scheduled_tasks_unique_key_idx ON scheduled_tasks(guild_id, plugin_id, name, unique_key)
WHERE (unique_key IS NOT NULL);

-- interval timers
--
-- so we already have a script_id column that is unused, it's always set to 0
-- because of that, it's perfect for plugin_id, whose 0 value means guild scoped
ALTER TABLE interval_timers RENAME COLUMN script_id TO plugin_id;

-- bucket_store
ALTER TABLE bucket_store
    ADD COLUMN plugin_id bigint NOT NULL;

ALTER TABLE bucket_store
    DROP CONSTRAINT bucket_store_pkey;

ALTER TABLE bucket_store
    ADD PRIMARY KEY (guild_id, plugin_id, bucket, key);

DROP INDEX bucket_store_float_idx;

CREATE INDEX bucket_store_float_idx ON bucket_store(guild_id, plugin_id, bucket, value_float)
WHERE (value_float IS NOT NULL);

