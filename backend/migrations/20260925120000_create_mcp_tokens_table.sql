-- Every credential the /mcp endpoint accepts: personal tokens the user
-- creates in Settings, plus the access/refresh tokens the OAuth flow issues
-- to connector clients. Opaque random values; only the SHA-256 hash is
-- stored. Never accepted by the REST API, which stays JWT-only.
CREATE TABLE mcp_tokens (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind text NOT NULL CHECK (kind IN ('personal', 'access', 'refresh')),
    -- User-chosen label for personal tokens; NULL for OAuth-issued ones.
    name text,
    -- The OAuth client an access/refresh token was issued to; NULL for
    -- personal tokens.
    client_id text,
    token_hash text NOT NULL UNIQUE,
    expires_at timestamptz,
    last_used_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK ((kind = 'personal') = (name IS NOT NULL))
);

CREATE INDEX idx_mcp_tokens_user_id ON mcp_tokens (user_id);
