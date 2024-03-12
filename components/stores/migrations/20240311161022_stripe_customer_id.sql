-- Add migration script here
ALTER TABLE user_meta
    ADD COLUMN stripe_customer_id TEXT;

