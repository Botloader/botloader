-- Add migration script here
-- This migration adds plugin_id to scheduled_tasks and bucket_store
--
-- scheduled_tasks
ALTER TABLE scheduled_tasks
    ADD COLUMN plugin_id bigint;

DROP UNIQUE INDEX scheduled_tasks_unique_key_idx;

CREATE UNIQUE INDEX scheduled_tasks_unique_key_idx ON scheduled_tasks (guild_id, plugin_id, name, unique_key)
WHERE (unique_key IS NOT NULL);

-- bucket_store
ALTER TABLE bucket_store
    ADD COLUMN plugin_id bigint;

ALTER TABLE bucket_store
    DROP CONSTRAINT bucket_store_pkey;

ALTER TABLE bucket_store
    ADD PRIMARY KEY (guild_id, plugin_id, bucket, key);

DROP INDEX bucket_store_float_idx;

CREATE INDEX bucket_store_float_idx ON bucket_store (guild_id, plugin_id, bucket, value_float)
WHERE (value_float IS NOT NULL);

