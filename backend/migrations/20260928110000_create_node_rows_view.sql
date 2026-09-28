-- One row per node with every derived column (blocked, progress, last
-- touched, dates), so the node-reading queries share one definition instead
-- of three copies of the same SELECT. Changed later with CREATE OR REPLACE
-- VIEW in a new migration - same columns in the same order, new ones only
-- appended.

-- What a path rolls up from: every node inside it, through nested paths,
-- except ideas (uncommitted) and the nested paths themselves.
CREATE FUNCTION path_leaves(path_id uuid) RETURNS SETOF uuid LANGUAGE sql STABLE AS $$
    WITH RECURSIVE inside(node_id) AS (
        SELECT from_node_id FROM edges WHERE to_node_id = path_id AND kind = 'part_of'
        UNION
        SELECT e.from_node_id FROM edges e JOIN inside i ON e.to_node_id = i.node_id
        WHERE e.kind = 'part_of'
    )
    SELECT n.id FROM inside JOIN nodes n ON n.id = inside.node_id
    WHERE n.kind NOT IN ('idea', 'path')
$$;

-- ponytail: path rollups recurse per path row on every read; fine for a
-- single-user graph, switch to a maintained closure table if it gets slow.
CREATE VIEW node_rows AS
SELECT
    n.id, n.user_id, n.kind, n.status, n.focus, n.title,
    n.progress_current, n.progress_total, n.progress_unit,
    n.color, n.notes, n.created_at, n.updated_at,
    (SELECT MIN(a.started_at) FROM active_periods a WHERE a.node_id = n.id) AS started_at,
    CASE WHEN n.status = 'done' THEN (
        SELECT a.ended_at FROM active_periods a WHERE a.node_id = n.id
        ORDER BY a.started_at DESC, a.id DESC LIMIT 1
    ) END AS completed_at,
    n.canvas_x, n.canvas_y, n.canvas_width, n.canvas_height, n.sort_order,
    ARRAY(SELECT nt.topic_id FROM node_topics nt WHERE nt.node_id = n.id) AS topic_ids,
    EXISTS (
        SELECT 1 FROM edges e JOIN nodes req ON req.id = e.to_node_id
        WHERE e.from_node_id = n.id AND e.kind = 'requires'
          AND req.status IS DISTINCT FROM 'done'
    ) AS blocked,
    -- Ideas inside a path don't count towards it.
    (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
     WHERE pe.to_node_id = n.id AND pe.kind = 'part_of' AND child.kind <> 'idea')
        AS container_total,
    (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
     WHERE pe.to_node_id = n.id AND pe.kind = 'part_of' AND child.status = 'done')
        AS container_done,
    (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = n.id) AS checklist_total,
    (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = n.id AND ci.done)
        AS checklist_done,
    (SELECT MAX(p.poked_at) FROM pokes p WHERE p.node_id = n.id) AS last_poked_at,
    -- A path's overall progress: every study counter and project checklist
    -- inside it, summed.
    COALESCE(rollup.progress_done, 0) AS path_progress_done,
    COALESCE(rollup.progress_total, 0) AS path_progress_total
FROM nodes n
LEFT JOIN LATERAL (
    SELECT
        SUM(COALESCE(leaf.progress_current, 0)
            + (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = leaf.id AND ci.done)
        )::bigint AS progress_done,
        SUM(COALESCE(leaf.progress_total, 0)
            + (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = leaf.id)
        )::bigint AS progress_total
    FROM path_leaves(n.id) AS inside(leaf_id) JOIN nodes leaf ON leaf.id = inside.leaf_id
    WHERE n.kind = 'path'
) rollup ON true;
