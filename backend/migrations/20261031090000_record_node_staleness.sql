-- Staleness is recorded, not derived: a sweep stamps `stale_since` when a
-- node goes untouched past the account's threshold and clears it once the
-- node is touched again (or stops qualifying). Recording it lets auto-pause
-- fire once, on entry, and keeps an auto-paused node marked stale.
ALTER TABLE nodes ADD COLUMN stale_since timestamptz;

-- A `StaleMode` setting (`always` / `poked` / `never`) applied to a node.
CREATE FUNCTION stale_mode_allows(mode text, poked boolean) RETURNS boolean
LANGUAGE sql IMMUTABLE AS $$
    SELECT mode = 'always' OR (mode = 'poked' AND poked)
$$;

CREATE OR REPLACE VIEW node_rows AS
SELECT
    n.id, n.user_id, n.kind,
    -- A path's status comes from what's inside it; its stored one is unused.
    CASE WHEN n.kind = 'path' THEN rollup.status ELSE n.status END AS status,
    n.focus, n.title,
    n.progress_current, n.progress_total, n.progress_unit,
    n.color, n.notes, n.created_at, n.updated_at,
    -- A path has no periods of its own: it started when the first thing
    -- inside it did, and completed when the last thing inside it ended.
    CASE WHEN n.kind = 'path' THEN rollup.started_at
         ELSE (SELECT MIN(a.started_at) FROM active_periods a WHERE a.node_id = n.id)
    END AS started_at,
    CASE
        WHEN n.kind = 'path' THEN CASE WHEN rollup.status = 'done' THEN rollup.ended_at END
        WHEN n.status = 'done' THEN (
            SELECT a.ended_at FROM active_periods a WHERE a.node_id = n.id
            ORDER BY a.started_at DESC, a.id DESC LIMIT 1
        )
    END AS completed_at,
    n.canvas_x, n.canvas_y, n.canvas_width, n.canvas_height, n.sort_order,
    ARRAY(SELECT nt.topic_id FROM node_topics nt WHERE nt.node_id = n.id) AS topic_ids,
    EXISTS (
        SELECT 1 FROM edges e JOIN nodes req ON req.id = e.to_node_id
        WHERE e.from_node_id = n.id AND e.kind = 'requires'
          AND (CASE WHEN req.kind = 'path' THEN path_status(req.id) ELSE req.status END)
              IS DISTINCT FROM 'done'
    ) AS blocked,
    -- Ideas inside a path don't count towards it.
    (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
     WHERE pe.to_node_id = n.id AND pe.kind = 'part_of' AND child.kind <> 'idea')
        AS container_total,
    (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
     WHERE pe.to_node_id = n.id AND pe.kind = 'part_of'
       AND (CASE WHEN child.kind = 'path' THEN path_status(child.id) ELSE child.status END)
           = 'done')
        AS container_done,
    (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = n.id) AS checklist_total,
    (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = n.id AND ci.done)
        AS checklist_done,
    -- A path is touched whenever anything inside it is.
    CASE WHEN n.kind = 'path' THEN rollup.last_poked_at
         ELSE (SELECT MAX(p.poked_at) FROM pokes p WHERE p.node_id = n.id) END AS last_poked_at,
    -- A path's overall progress: every study counter and project checklist
    -- inside it, summed.
    COALESCE(rollup.progress_done, 0) AS path_progress_done,
    COALESCE(rollup.progress_total, 0) AS path_progress_total,
    n.archived,
    n.stale_since
FROM nodes n
LEFT JOIN LATERAL (
    SELECT
        path_status(n.id) AS status,
        MIN((SELECT MIN(a.started_at) FROM active_periods a WHERE a.node_id = leaf.id))
            AS started_at,
        MAX((SELECT MAX(a.ended_at) FROM active_periods a WHERE a.node_id = leaf.id))
            AS ended_at,
        SUM(COALESCE(leaf.progress_current, 0)
            + (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = leaf.id AND ci.done)
        )::bigint AS progress_done,
        SUM(COALESCE(leaf.progress_total, 0)
            + (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = leaf.id)
        )::bigint AS progress_total,
        MAX((SELECT MAX(p.poked_at) FROM pokes p WHERE p.node_id = leaf.id)) AS last_poked_at
    FROM path_leaves(n.id) AS inside(leaf_id) JOIN nodes leaf ON leaf.id = inside.leaf_id
    WHERE n.kind = 'path'
) rollup ON true;
