-- Split machine-derived script state out of guild_scripts.
--
-- guild_scripts keeps only user-owned config (source, settings values, plugin
-- fields); everything observed by running the script in a vm (command and
-- interval timer contributions, settings definitions) moves to
-- guild_script_derived, written only by the scheduler. source_hash,
-- settings_hash and runtime_version record the inputs the derived state was
-- observed under so writers can skip the write when nothing changed and
-- readers can detect staleness.
CREATE TABLE guild_script_derived(
    script_id bigint PRIMARY KEY REFERENCES guild_scripts(id) ON DELETE CASCADE,
    guild_id bigint NOT NULL,
    source_hash text NOT NULL,
    settings_hash text NOT NULL,
    runtime_version text NOT NULL,
    contributes_commands jsonb NOT NULL DEFAULT '[]',
    contributes_interval_timers jsonb NOT NULL DEFAULT '[]',
    settings_definitions jsonb,
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX guild_script_derived_guild_id_idx ON guild_script_derived(guild_id);

ALTER TABLE guild_scripts
    DROP COLUMN contributes_commands,
    DROP COLUMN contributes_interval_timers,
    DROP COLUMN settings_definitions;

