-- Add migration script here
ALTER TABLE joined_guilds
    ADD COLUMN IF NOT EXISTS left_at TIMESTAMP WITH TIME ZONE;

