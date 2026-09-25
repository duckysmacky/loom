-- OAuth 2.1 for MCP connector clients (Claude, Cowork, other MCP clients):
-- clients register themselves (RFC 7591 dynamic registration), the user
-- approves them on Loom's consent page, and they swap a one-time,
-- PKCE-bound code for access/refresh tokens stored in mcp_tokens.
CREATE TABLE oauth_clients (
    client_id text PRIMARY KEY,
    client_name text NOT NULL,
    redirect_uris text[] NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Short-lived and single-use: a code row is deleted the moment it's
-- exchanged. Only its hash is stored.
CREATE TABLE oauth_codes (
    code_hash text PRIMARY KEY,
    client_id text NOT NULL REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redirect_uri text NOT NULL,
    code_challenge text NOT NULL,
    expires_at timestamptz NOT NULL
);

ALTER TABLE mcp_tokens
    ADD CONSTRAINT mcp_tokens_client_id_fkey
        FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
    ADD CONSTRAINT mcp_tokens_client_matches_kind
        CHECK ((kind = 'personal') = (client_id IS NULL));

CREATE INDEX idx_mcp_tokens_client_id ON mcp_tokens (client_id);
