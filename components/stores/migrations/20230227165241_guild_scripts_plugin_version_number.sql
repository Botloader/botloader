-- Add migration script here
ALTER TABLE guild_scripts
    ADD COLUMN plugin_version_number int;

