-- With the account's path status mode set to `manual`, a path's stored
-- status is used as is; otherwise it's derived from what's inside it.
CREATE OR REPLACE FUNCTION path_status(path_id uuid) RETURNS node_status LANGUAGE sql STABLE AS $$
    SELECT CASE
        WHEN u.settings->>'path_status_mode' = 'manual' THEN p.status
        ELSE (
            SELECT CASE
                WHEN bool_or(n.status = 'active') THEN 'active'
                WHEN bool_or(n.status = 'paused') THEN 'paused'
                WHEN COUNT(*) = 0 OR bool_or(n.status IN ('queued', 'idea')) THEN 'queued'
                ELSE 'done'
            END::node_status
            FROM path_leaves(path_id) AS inside(leaf_id) JOIN nodes n ON n.id = inside.leaf_id
        )
    END
    FROM nodes p JOIN users u ON u.id = p.user_id
    WHERE p.id = path_id
$$;
