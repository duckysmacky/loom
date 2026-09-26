-- Active periods become the single source of truth for a node's dates:
-- started_at is now the first period's start and completed_at the last
-- period's end while the node is done, both derived in queries. Fold the
-- old columns into periods, then drop them.

-- 1. Started but no periods yet: one period covering the old span (open
--    when it was never completed).
INSERT INTO active_periods (node_id, started_at, ended_at)
SELECT n.id, n.started_at,
    CASE WHEN n.completed_at IS NOT NULL THEN GREATEST(n.completed_at, n.started_at) END
FROM nodes n
WHERE n.started_at IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM active_periods a WHERE a.node_id = n.id);

-- 2. Started before the earliest custom period: keep that gap as its own
--    period, from the old start up to where the custom periods begin.
INSERT INTO active_periods (node_id, started_at, ended_at)
SELECT n.id, n.started_at, earliest.started_at
FROM nodes n
JOIN LATERAL (
    SELECT MIN(a.started_at) AS started_at FROM active_periods a WHERE a.node_id = n.id
) earliest ON true
WHERE n.started_at IS NOT NULL AND earliest.started_at > n.started_at;

-- 3. Done nodes: the last period must end at the completion date. An open
--    one is closed there; one that already closed earlier (a pause) keeps
--    its record and gets a completion-day period after it.
UPDATE active_periods p
SET ended_at = GREATEST(last.completed_at, last.started_at)
FROM (
    SELECT DISTINCT ON (a.node_id) a.id, a.started_at, a.ended_at, n.completed_at
    FROM active_periods a
    JOIN nodes n ON n.id = a.node_id
    WHERE n.status = 'done' AND n.completed_at IS NOT NULL
    ORDER BY a.node_id, a.started_at DESC, a.id DESC
) last
WHERE p.id = last.id AND last.ended_at IS NULL;

INSERT INTO active_periods (node_id, started_at, ended_at)
SELECT last.node_id, last.completed_at, last.completed_at
FROM (
    SELECT DISTINCT ON (a.node_id) a.node_id, a.ended_at, n.completed_at
    FROM active_periods a
    JOIN nodes n ON n.id = a.node_id
    WHERE n.status = 'done' AND n.completed_at IS NOT NULL
    ORDER BY a.node_id, a.started_at DESC, a.id DESC
) last
WHERE last.ended_at < last.completed_at;

-- 4. Done nodes that never started have no period, so their completion
--    date is dropped: a node that never began has no "to" date.
ALTER TABLE nodes DROP COLUMN started_at, DROP COLUMN completed_at;
