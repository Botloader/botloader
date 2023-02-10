-- Add migration script here
ALTER TABLE guild_scripts
    DROP CONSTRAINT guild_scripts_guild_id_name_key;

CREATE UNIQUE INDEX guild_scripts_guild_id_name_key ON guild_scripts (guild_id, plugin_id, name);

