CREATE TYPE node_kind AS ENUM ('idea', 'project', 'course');
CREATE TYPE node_status AS ENUM ('idea', 'queued', 'active', 'paused', 'done', 'archived');
CREATE TYPE node_focus AS ENUM ('primary', 'secondary', 'background');

-- updated_at is set explicitly by the repository layer on every UPDATE, not triggered.
CREATE TABLE nodes (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind node_kind NOT NULL,
    status node_status NOT NULL DEFAULT 'idea',
    focus node_focus NOT NULL DEFAULT 'secondary',
    title text NOT NULL,
    progress_current integer,
    progress_total integer,
    color text,
    notes text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    completed_at timestamptz,
    CONSTRAINT nodes_progress_pair_nullness
        CHECK ((progress_current IS NULL) = (progress_total IS NULL)),
    CONSTRAINT nodes_progress_current_nonnegative
        CHECK (progress_current IS NULL OR progress_current >= 0),
    CONSTRAINT nodes_progress_total_positive
        CHECK (progress_total IS NULL OR progress_total > 0),
    CONSTRAINT nodes_progress_current_le_total
        CHECK (progress_current IS NULL OR progress_current <= progress_total)
);

CREATE INDEX idx_nodes_user_status ON nodes (user_id, status);
CREATE INDEX idx_nodes_user_focus ON nodes (user_id, focus);
