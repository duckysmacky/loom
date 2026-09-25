-- Manual card order for the Organized view. NULL means "not manually
-- ordered yet" - those nodes fall back to the existing auto order, after
-- any ranked ones.
ALTER TABLE nodes ADD COLUMN sort_order integer;
