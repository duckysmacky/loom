-- "course" was too narrow: books and general topics belong in the same
-- bucket. Renaming the enum value keeps every existing row.
ALTER TYPE node_kind RENAME VALUE 'course' TO 'study';
