CREATE TABLE IF NOT EXISTS images(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    uploaded_by bigint NOT NULL,
    plugin_id bigint REFERENCES plugins(id) ON DELETE CASCADE,
    width int NOT NULL,
    height int NOT NULL,
    bytes bytea,
    created_at timestamptz NOT NULL,
    deleted_at timestamptz
);

CREATE TABLE IF NOT EXISTS plugin_images(
    plugin_id bigint NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    image_id uuid NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL,
    description text NOT NULL,
    position int NOT NULL,
    kind int NOT NULL,
    PRIMARY KEY (plugin_id, image_id)
);

