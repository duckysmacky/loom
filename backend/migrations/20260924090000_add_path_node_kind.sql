-- A path (learning path / roadmap) is its own kind rather than being inferred
-- from having part_of children. Existing rows keep their kind.
ALTER TYPE node_kind ADD VALUE 'path';
