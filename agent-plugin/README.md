# Loom plugin for Claude and Codex

This plugin connects your agent to a self-hosted [Loom](https://github.com/duckysmacky/loom)
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
-> Paste `github.com/duckysmacky/duckysmacky-plugin-marketplace` - this will add my personal skills
marketplace
2. Customize -> Plugins -> Search for `loom` in the search bar and then
3. Connectors -> Yours -> Find `loom` in the list and press "connect"
4. Claude opens Loom's consent page (sign in first if asked) - check the app
name and click "approve"

### Claude Code

Run install commands:

```sh
/plugin marketplace add duckysmacky/duckysmacky-plugin-marketplace
/plugin install loom@duckysmacky
```

1. When asked, enter your Loom URL, e.g. `https://loom.example.com`.
2. Run `/mcp`, select `loom` and authenticate.
3. Approve the connection in the browser window that opens.

### Codex and ChatGPT Desktop

The same directory also contains a Codex manifest. Set your instance URL in
`codex.mcp.json` in the installed plugin directory after installing from the
shared `duckysmacky/duckysmacky-plugin-marketplace` marketplace.
See the main README's [Connecting agents](../README.md#connecting-agents)
section for Codex CLI and ChatGPT Desktop installation steps.

### Other agents

```sh
npx skills add duckysmacky/loom --skill loom
```

This installs only the skill. Configure a Streamable HTTP MCP connection to
`https://loom.example.com/mcp` separately, then authenticate through OAuth or
a personal access token. Add `--global` to install the skill for all projects.

## Contents

```
agent-plugin/
├── .claude-plugin/plugin.json      # manifest: MCP server + Loom URL prompt
├── .codex-plugin/plugin.json       # Codex manifest: shared skill + MCP config
├── codex.mcp.json                  # set your instance URL after installing
└── skills/loom/
    ├── SKILL.md                    # how to work with Loom
    └── references/setup.md         # connecting & troubleshooting
```
