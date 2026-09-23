-- Paths are the only containers, and a node sits in at most one path (paths
-- may nest). Bring existing data in line, then enforce it.

-- 1. Anything already used as a container becomes a path.
UPDATE nodes SET kind = 'path'
WHERE kind <> 'path'
  AND id IN (SELECT to_node_id FROM edges WHERE kind = 'part_of');

-- Paths have no tracked progress or checklist of their own.
UPDATE nodes
SET progress_current = NULL, progress_total = NULL, progress_unit = NULL
WHERE kind = 'path';
DELETE FROM checklist_items c USING nodes n WHERE n.id = c.node_id AND n.kind = 'path';

-- 2. Keep only the oldest part_of edge per child.
DELETE FROM edges e
USING edges older
WHERE e.kind = 'part_of' AND older.kind = 'part_of'
  AND e.from_node_id = older.from_node_id
  AND (older.created_at, older.id) < (e.created_at, e.id);

-- 3. One path per node.
CREATE UNIQUE INDEX edges_one_path_per_node ON edges (from_node_id) WHERE kind = 'part_of';

-- Children are positioned relative to their path on the canvas, so any
-- position saved while they were top-level is meaningless now.
UPDATE nodes SET canvas_x = NULL, canvas_y = NULL
WHERE id IN (SELECT from_node_id FROM edges WHERE kind = 'part_of');

-- 4. A path's box size on the canvas (null = sized to fit its children).
ALTER TABLE nodes
    ADD COLUMN canvas_width double precision,
    ADD COLUMN canvas_height double precision,
    ADD CONSTRAINT nodes_canvas_size_pair
        CHECK ((canvas_width IS NULL) = (canvas_height IS NULL)),
    ADD CONSTRAINT nodes_canvas_size_positive
        CHECK (canvas_width IS NULL OR (canvas_width > 0 AND canvas_height > 0));
