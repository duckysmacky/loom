-- Each active period records the focus tier the node had while it ran, so
-- focus moves can be read back over time.
ALTER TABLE active_periods ADD COLUMN focus node_focus;

-- A currently-active node's periods take its present focus; everything else
-- (done, paused, queued, archived) has no better record than the default.
UPDATE active_periods a
SET focus = COALESCE(n.focus, 'secondary')
FROM nodes n
WHERE n.id = a.node_id AND n.status = 'active';

UPDATE active_periods SET focus = 'secondary' WHERE focus IS NULL;

ALTER TABLE active_periods ALTER COLUMN focus SET NOT NULL;
