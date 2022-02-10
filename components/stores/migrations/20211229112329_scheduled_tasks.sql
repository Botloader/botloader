-- Add migration script here
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id bigserial PRIMARY KEY,
    guild_id bigint NOT NULL,
    name text NOT NULL,
    unique_key text,
    value jsonb NOT NULL,
    exec_at timestamp with time zone NOT NULL
);

CREATE INDEX scheduled_tasks_guild_id_exec_at_idx ON scheduled_tasks (guild_id, exec_at);

CREATE UNIQUE INDEX scheduled_tasks_unique_key_idx ON scheduled_tasks (guild_id, name, unique_key)
WHERE (unique_key IS NOT NULL);

