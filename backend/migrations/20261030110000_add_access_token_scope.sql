-- What a personal token may do. Existing tokens and OAuth-issued ones keep
-- full access, so nothing that works today stops working.
CREATE TYPE token_scope AS ENUM ('read', 'read_write');

ALTER TABLE access_tokens
    ADD COLUMN scope token_scope NOT NULL DEFAULT 'read_write';
