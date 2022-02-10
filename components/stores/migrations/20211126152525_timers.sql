-- Add migration script here
CREATE TABLE IF NOT EXISTS interval_timers (
    guild_id bigint NOT NULL,
    script_id bigint NOT NULL,
    timer_name text NOT NULL,
    interval_minutes int,
    interval_cron text,
    last_run_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    PRIMARY KEY (guild_id, script_id, timer_name)
);

ALTER TABLE guild_scripts
    ADD COLUMN IF NOT EXISTS contributes_interval_timers jsonb NOT NULL DEFAULT '[]';

