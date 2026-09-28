-- Ideas are uncommitted captures: they carry no status and no focus tier
-- until promoted to a project/study/path. The `idea` status stays as an
-- ordinary status for committed nodes.
ALTER TABLE nodes
    ALTER COLUMN status DROP NOT NULL,
    ALTER COLUMN status DROP DEFAULT,
    ALTER COLUMN focus DROP NOT NULL,
    ALTER COLUMN focus DROP DEFAULT;

UPDATE nodes SET status = NULL, focus = NULL WHERE kind = 'idea';

-- Ideas aren't being worked on: end whatever period was still running.
UPDATE active_periods a SET ended_at = GREATEST(now(), a.started_at)
FROM nodes n
WHERE n.id = a.node_id AND n.kind = 'idea' AND a.ended_at IS NULL;

ALTER TABLE nodes ADD CONSTRAINT nodes_idea_has_no_status CHECK (
    (kind = 'idea') = (status IS NULL) AND (kind = 'idea') = (focus IS NULL)
);
