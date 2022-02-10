-- Add migration script here
ALTER TABLE guild_scripts
    ADD COLUMN IF NOT EXISTS contributes_commands jsonb NOT NULL DEFAULT '[]';

