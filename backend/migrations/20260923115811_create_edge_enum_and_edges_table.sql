CREATE TYPE edge_kind AS ENUM ('requires', 'part_of', 'related');

CREATE TABLE edges (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    from_node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    to_node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    kind edge_kind NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT edges_no_self_loop CHECK (from_node_id <> to_node_id),
    CONSTRAINT edges_unique_from_to_kind UNIQUE (from_node_id, to_node_id, kind)
);

CREATE INDEX idx_edges_from_kind ON edges (from_node_id, kind);
CREATE INDEX idx_edges_to_kind ON edges (to_node_id, kind);
