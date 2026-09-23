-- Optional label for what progress_current/progress_total count
-- ("videos", "chapters") - shown as "15 / 30 videos".
ALTER TABLE nodes ADD COLUMN progress_unit text;
