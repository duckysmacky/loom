-- Sticky notes on the canvas: free text, not graph nodes. A note may sit
-- inside a path box (`path_id`), its position then relative to the box.
CREATE TABLE canvas_notes (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body text NOT NULL DEFAULT '',
    canvas_x double precision NOT NULL,
    canvas_y double precision NOT NULL,
    canvas_width double precision NOT NULL,
    canvas_height double precision NOT NULL,
    path_id uuid REFERENCES nodes(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_canvas_notes_user ON canvas_notes (user_id);

-- Annotation arrows between canvas items (notes and/or nodes). Purely
-- visual: no meaning in the graph. Each end is exactly one node or note.
CREATE TABLE canvas_arrows (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    from_node_id uuid REFERENCES nodes(id) ON DELETE CASCADE,
    from_note_id uuid REFERENCES canvas_notes(id) ON DELETE CASCADE,
    to_node_id uuid REFERENCES nodes(id) ON DELETE CASCADE,
    to_note_id uuid REFERENCES canvas_notes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT canvas_arrows_one_from CHECK (num_nonnulls(from_node_id, from_note_id) = 1),
    CONSTRAINT canvas_arrows_one_to CHECK (num_nonnulls(to_node_id, to_note_id) = 1),
    CONSTRAINT canvas_arrows_no_self_loop CHECK (
        from_node_id IS DISTINCT FROM to_node_id OR from_note_id IS DISTINCT FROM to_note_id
    )
);

CREATE INDEX idx_canvas_arrows_user ON canvas_arrows (user_id);
