# Connecting to Loom

The Loom MCP server is part of the Loom backend. It is **off by default**, and the person who
runs the Loom instance has to switch it on. There is one server per instance, at
`<PUBLIC_URL>/mcp`, e.g. `https://loom.example.com/mcp`.

## Server side (instance owner)

1. In the server's `.env`, set:

   ```sh
   MCP_ENABLED=true
   PUBLIC_URL=https://loom.example.com   # the public origin, no path, no trailing slash
   ```

2. Restart the stack with `docker compose up -d`.
3. Check the server. `curl -i https://loom.example.com/mcp -X POST` should answer
   `401 Unauthorized` with a `WWW-Authenticate: Bearer resource_metadata=...` header.
   - A 404 means MCP is still off, or the reverse proxy doesn't forward `/mcp`.
4. In Loom, open **Settings → Connections**. It shows the server URL and lets you create access
   tokens and see connected apps.

The project README covers the reverse-proxy (nginx) settings in full.

## Client side

Pick one of these.

- **Claude (claude.ai, Desktop, Cowork): custom connector.**
  1. Go to Settings → Connectors → Add custom connector and paste `https://loom.example.com/mcp`.
  2. Claude opens Loom's sign-in and consent page. Sign in and choose **Approve**.
  3. To use this skill there too, zip the `skills/loom` folder and upload it in Claude's skill
     settings.
- **Claude Code: this plugin.**
  1. Install the plugin and enter the Loom URL when asked.
  2. Run `/mcp`, pick `loom` and authenticate. A browser opens the same consent page.
- **Claude Code: manual setup with OAuth.**
  1. Run `claude mcp add --transport http loom https://loom.example.com/mcp`.
  2. Run `/mcp` to sign in.
- **Claude Code: manual setup with a token** (headless machines, CI).
  1. Create a token in Settings → Connections.
  2. Run
     `claude mcp add --transport http loom https://loom.example.com/mcp --header "Authorization: Bearer loom_..."`.
- **Any other MCP client** that supports Streamable HTTP: use the same URL, with either OAuth or
  `Authorization: Bearer <token>`.

## Troubleshooting

| Symptom | Cause / fix |
| --- | --- |
| `404` on `/mcp` | `MCP_ENABLED` isn't `true`, the backend wasn't restarted, or the proxy doesn't pass `/mcp` through. |
| `403 Forbidden` on `/mcp` | The `Host` header doesn't match `PUBLIC_URL`. Set `proxy_set_header Host $host;` in the host nginx, and check `PUBLIC_URL` is the address clients actually use. |
| `401` with a token | The token was revoked or mistyped. Personal tokens start with `loom_`. OAuth tokens expire hourly and refresh automatically; if refreshing fails, reconnect. |
| Consent page says "unknown client" | The connector was registered against another instance, or the database was reset. Remove the connector and add it again. |
| "redirect_uri ... must be https" during connect | Loom only accepts https redirect URIs, or http on localhost. |
| Tools time out or hang behind a proxy | Turn off response buffering for `/mcp` (`proxy_buffering off;`) and raise `proxy_read_timeout`. |
| An app has access it shouldn't | Settings → Connections: revoke the token or disconnect the app. |
