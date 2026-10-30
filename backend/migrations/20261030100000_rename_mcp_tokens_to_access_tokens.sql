-- The token table now backs the REST API as well as /mcp, so the MCP
-- name no longer fits. Pure rename: rows are untouched.
ALTER TABLE mcp_tokens RENAME TO access_tokens;

ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_pkey TO access_tokens_pkey;
ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_token_hash_key TO access_tokens_token_hash_key;
ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_kind_check TO access_tokens_kind_check;
ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_check TO access_tokens_name_matches_kind;
ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_user_id_fkey TO access_tokens_user_id_fkey;
ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_client_id_fkey TO access_tokens_client_id_fkey;
ALTER TABLE access_tokens RENAME CONSTRAINT mcp_tokens_client_matches_kind TO access_tokens_client_matches_kind;

ALTER INDEX idx_mcp_tokens_user_id RENAME TO idx_access_tokens_user_id;
ALTER INDEX idx_mcp_tokens_client_id RENAME TO idx_access_tokens_client_id;
