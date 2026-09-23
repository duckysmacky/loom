-- Distinguishes a token revoked by rotation (replaying it is a theft signal)
-- from one revoked outright by logout/password change or simply expired
-- (replaying those is just a stale cookie). Existing rows default to false:
-- without the history, the non-cascading reading is the safe one.
ALTER TABLE refresh_tokens ADD COLUMN rotated boolean NOT NULL DEFAULT false;
