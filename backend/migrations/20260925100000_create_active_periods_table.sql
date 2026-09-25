-- Optional, additive active/paused history for a node - coexists with the
-- single started_at/completed_at pair on nodes, never replaces it. Zero
-- rows is the normal case: a node that goes active -> done in one
-- continuous run needs none.
CREATE TABLE active_periods (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    started_at timestamptz NOT NULL,
    ended_at timestamptz,
    CHECK (ended_at IS NULL OR ended_at >= started_at)
);

CREATE INDEX idx_active_periods_node_started_at ON active_periods (node_id, started_at DESC);
