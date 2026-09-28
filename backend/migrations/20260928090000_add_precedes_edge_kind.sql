-- A soft "do A before B" order between two nodes: directed, cycles rejected,
-- never blocks. Not referenced elsewhere in this file (Postgres rejects using
-- a freshly added enum value in the same transaction).
ALTER TYPE edge_kind ADD VALUE 'precedes';
