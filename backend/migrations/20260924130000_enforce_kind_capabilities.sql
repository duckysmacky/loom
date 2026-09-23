-- Kinds are strict: tracked progress is study-only, checklists are
-- project-only. Drop existing data that no longer fits its node's kind.
UPDATE nodes
SET progress_current = NULL, progress_total = NULL, progress_unit = NULL
WHERE kind <> 'study';

DELETE FROM checklist_items c
USING nodes n
WHERE n.id = c.node_id AND n.kind <> 'project';
