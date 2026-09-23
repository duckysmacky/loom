CREATE TABLE pokes (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    poked_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_pokes_node_poked_at ON pokes (node_id, poked_at DESC);
