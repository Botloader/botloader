-- Add migration script here
CREATE TABLE IF NOT EXISTS bucket_store (
    guild_id bigint NOT NULL,
    bucket text NOT NULL,
    key text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    expires_at timestamp with time zone,
    -- only one and atleast one of these has to be present
    value_json jsonb,
    value_float double precision,
    PRIMARY KEY (guild_id, bucket, key)
);

CREATE INDEX IF NOT EXISTS bucket_store_float_idx ON bucket_store (guild_id, bucket, value_float)
WHERE (value_float IS NOT NULL);

