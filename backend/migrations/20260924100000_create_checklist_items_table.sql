-- Optional task checklist on any node. Ownership is inherited from the node
-- (queries join nodes.user_id); done/total drives the node's progress.
CREATE TABLE checklist_items (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    title text NOT NULL,
    done boolean NOT NULL DEFAULT false,
    position integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_checklist_items_node_position ON checklist_items (node_id, position);
