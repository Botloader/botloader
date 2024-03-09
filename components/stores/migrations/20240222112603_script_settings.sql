-- Add migration script here
ALTER TABLE guild_scripts
    ADD COLUMN settings_values jsonb;

ALTER TABLE guild_scripts
    ADD COLUMN settings_definitions jsonb;

