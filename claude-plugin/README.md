# Loom plugin for Claude

This plugin connects Claude to a self-hosted [Loom](https://github.com/duckysmacky/loom)
instance. It includes:

- **the `loom` MCP server config**: `<your Loom URL>/mcp`, signed in through Loom's own OAuth
  consent page. There's no token to paste.
- **the `loom` skill**: Loom's domain model, the tool set, and recipes such as building a
  complete learning path in one call.

The instance must have its MCP server switched on (`MCP_ENABLED=true` and `PUBLIC_URL`). See
the main README's **MCP server** section.

## Install 

### Claude Web and Desktop

1. Customize -> Plugins -> Add -> Add marketplace -> Add from a repository
-> Paste `github.com/duckysmacky/skills` - this will add my personal skills
marketplace
2. Customize -> Plugins -> Search for `loom` in the search bar and then
3. Connectors -> Yours -> Find `loom` in the list and press "connect"
4. Claude opens Loom's consent page (sign in first if asked) - check the app
name and click "approve"

### Claude Code

Run install commands:

```sh
/plugin marketplace add duckysmacky/skills
/plugin install loom@duckysmacky
```

1. When asked, enter your Loom URL, e.g. `https://loom.example.com`.
2. Run `/mcp`, select `loom` and authenticate.
3. Approve the connection in the browser window that opens.

## Contents

```
claude-plugin/
├── .claude-plugin/plugin.json      # manifest: MCP server + Loom URL prompt
└── skills/loom/
    ├── SKILL.md                    # how to work with Loom
    └── references/setup.md         # connecting & troubleshooting
```
