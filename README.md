# Loom

A flexible manager for everything you're building, studying, and dreaming up.

![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
![version](https://img.shields.io/badge/version-1.0.0-blue)

## Table of contents

- [About](#about)
- [Features](#features)
  - [Nodes](#nodes)
  - [Connections](#connections)
  - [Topics](#topics)
  - [Pokes & staleness](#pokes--staleness)
  - [Views](#views)
  - [Backlog & promoting](#backlog--promoting)
  - [Quick capture, command palette, and detail panel](#quick-capture-command-palette-and-detail-panel)
  - [Settings](#settings)
  - [AI agents (MCP)](#ai-agents-mcp)
- [Installation](#installation)
  - [Requirements](#requirements)
  - [Quick start](#quick-start)
  - [Configuration](#configuration)
  - [Self-hosting on a VPS](#self-hosting-on-a-vps)
  - [Updating](#updating)
  - [Using an external Postgres](#using-an-external-postgres)
  - [Backups](#backups)
- [MCP server (optional)](#mcp-server-optional)
  - [Enabling it](#enabling-it)
  - [Reverse proxy](#reverse-proxy)
  - [Connecting Claude](#connecting-claude)
  - [Connecting other agents](#connecting-other-agents)
  - [What agents can do](#what-agents-can-do)
  - [Security](#security)
- [Development](#development)
- [License](#license)

## About

Loom is a single-user web app for tracking projects, studies, and ideas as
one connected graph, built to fight the context-switching overload of
juggling several parallel projects, courses, and half-formed ideas at
once.

It isn't a kanban board or a task manager, on purpose:

- **Nodes are things you pursue, not tasks to check off.** A project, a
  course, or a raw idea - each with its own progress, not a checklist
  item lost among a hundred others.
- **Dependencies decide what's blocked.** Draw a `requires` edge between
  two nodes and the dependent one is automatically marked blocked until
  the other is done - no manual "waiting on" label to keep in sync.
- **Working on something is a deliberate signal, not an edit.** "Poking"
  a node logs that you touched it today, separate from whatever field you
  changed - so "last touched" actually means something.
- **Staleness nudges you back.** Anything active that hasn't been poked
  in two weeks quietly surfaces on the dashboard instead of getting
  lost.
- **Ideas get promoted, not recreated.** Write something down as a loose
  idea, then commit to it later in place - same node, same connections,
  now tracked for real.

## Features

### Nodes

Every node has a **kind**, and each kind can do something the others
can't:

| Kind | For | Can track |
|---|---|---|
| **Idea** | A loosely captured, uncommitted thought | - |
| **Project** | Something being actively built | A checklist |
| **Study** | A course, book, or topic | A progress counter (e.g. "15 of 30 chapters") |
| **Path** | A learning path or roadmap | Other nodes nested inside it (paths can nest too) |

Promoting an idea to project/study/path changes its kind in place - same
node, same connections - and drops only what the new kind genuinely can't
hold (leaving a project behind drops its checklist, for instance).

Every node also has a **status** (idea -> queued -> active -> paused -> done,
plus archived) and a **focus** tier (primary / secondary / background,
independent of status) that decides what surfaces on the dashboard, plus
notes, an accent color, and free-form topics.

### Connections

Nodes connect with directed edges:

- **Requires** - a hard dependency. The dependent node shows as
  **blocked** until the required one is done.
- **Inside a path** - nests a node inside a path/roadmap container. A
  node lives in only one path at a time.
- **Related** - a soft, non-blocking association.

### Topics

Free-form tags for filtering, coloring, and grouping nodes - kept
separate from the dependency graph so tagging never clutters what
actually blocks what.

### Pokes & staleness

"Poke" a node to log that you worked on it today. It's how "last
touched" is tracked, independent of any field edit. Anything active or
queued that hasn't been poked in 14 days shows up on the dashboard as
going stale, with a poke button right there.

### Views

- **Dashboard** - your primary-focus nodes (actionable first, blocked
  after), what's going stale, your backlog, and your paths, at a glance.
- **Organized board** - a card grid, grouped by focus, kind, status,
  or tag.
- **Canvas board** - a draggable node graph: position and path 
  membership persist, path boxes resize, and dragging between handles
  creates connections.
- **Timeline board** - bars from started to completed (or to now),
  with a marker for the last time you poked each one.

All three board views share one filter/search bar (kind, status, focus,
topic, search), so switching views never resets what you're looking at.

### Backlog & promoting

Anything with status "idea" - whatever its kind - lives in the Backlog
tab of the Nodes page, out of the way of the board. Promoting it opens a
dialog to set its real kind, status, and focus in one step.

### Quick capture, command palette, and detail panel

- Press **N** anywhere to quick-capture a new node (title, kind, focus,
  and an option to start it immediately).
- Press **Ctrl+K** for a command palette: fuzzy node search plus
  navigation, capture, and sign-out commands.
- Click any node to open its detail panel as a deep-linkable slide-over
  - notes, progress, connections, checklist, pokes, and history, all in
  one place.

### Settings

Account (email, password), appearance (theme, density), topics & tags,
functionality defaults (default board view, quick-capture defaults,
poke-from-cards), and connections (MCP server URL, access tokens,
connected apps).

### AI agents (MCP)

An optional, built-in [MCP](https://modelcontextprotocol.io) server lets
AI agents - Claude (chat, Cowork, Code) or any other MCP client - read
and edit your graph. Ask an agent to "build me a learning path for
async Rust" and it creates the path, its study and project nodes with
notes, checklists and progress units, and the `part_of`/`requires` edges
between them, in one go. Connectors sign in through Loom's own consent
page; a Claude plugin with a Loom skill is included. Off by default - see
[MCP server (optional)](#mcp-server-optional).

## Installation

### Requirements

- Docker and Docker Compose
- A domain and reverse proxy (nginx, Caddy, etc.) if self-hosting
  publicly - see [Self-hosting on a VPS](#self-hosting-on-a-vps)

### Quick start

```sh
git clone https://github.com/duckysmacky/loom.git
cd loom
cp .env.example .env
```

Edit `.env`: at minimum, set `JWT_SECRET` to a real secret
(`openssl rand -hex 32`) and change `DB_PASSWORD`.

```sh
docker compose up -d
```

Open `http://localhost:8081` to access the app

### Configuration

All variables live in `.env` (see `.env.example`):

| Variable | Default | Description |
|---|---|---|
| `DB_HOST` | `database` | Postgres host - `database` for the bundled container, or an external host |
| `DB_PORT` | `5432` | Postgres port |
| `DB_NAME` | `loom` | Postgres database name |
| `DB_USER` | `loom` | Postgres user |
| `DB_PASSWORD` | - | Postgres password - change this |
| `DATABASE_URL` | - | Only used for compile-time query checks; keep in sync with the `DB_*` vars above |
| `BACKEND_PORT` | `8080` | Host port the API is published on |
| `FRONTEND_PORT` | `8081` | Host port the web UI is published on |
| `LOOM_VERSION` | `latest` | Image tag `docker compose up` pulls - pin to a release (e.g. `1.0.0`) for a stable deploy |
| `BIND_ADDR` | `0.0.0.0:8080` | Address the backend binds to inside its container - only change if running the binary directly, outside Docker |
| `RUST_LOG` | `info` | Backend log level |
| `JWT_SECRET` | - | Signs/verifies JWT access tokens. Generate with `openssl rand -hex 32`. Must be at least 32 bytes - the app refuses to start otherwise |
| `ALLOW_SIGNUP` | `true` | Whether `POST /api/auth/signup` accepts new accounts. Leave `true` for the first run, then set `false` once your account exists - single-user app, no reason to leave account creation open |
| `MCP_ENABLED` | `false` | Turns on the [MCP server](#mcp-server-optional) for AI agents |
| `PUBLIC_URL` | - | Public origin Loom is served at, e.g. `https://loom.example.com` (no path, no trailing slash). Required when `MCP_ENABLED=true` |

### Self-hosting on a VPS

Both compose ports are bound to `127.0.0.1` - they're meant to sit behind
a reverse proxy, not be reachable directly. The frontend container
already proxies `/api/` to the backend internally, so your reverse proxy
only needs one upstream. An nginx example:

```nginx
location / {
    proxy_pass http://127.0.0.1:8081;
    proxy_set_header X-Real-IP $remote_addr;
}
```

The `/api/auth/*` rate limiter reads the client IP from
`X-Forwarded-For`/`X-Real-IP`, so this block must set one of those -
otherwise every request behind the proxy collapses into one shared
rate-limit bucket. nginx forwards that header through to the backend
unless something overrides it, which nothing here does.

**HTTPS is required** - the refresh-token cookie is marked `Secure`, so
login won't work over plain HTTP through a public proxy. Get a
certificate with [certbot](https://certbot.eff.org/) (or your proxy's
built-in ACME support) and point your reverse proxy at 443.

### Updating

```sh
docker compose pull
docker compose up -d
```

Pin `LOOM_VERSION` in `.env` to a specific release tag if you'd rather
update deliberately than track `latest`.

### Using an external Postgres

Point the `DB_*` vars at your instance and run just the app services:

```sh
docker compose up -d backend frontend
```

### Backups

```sh
docker compose exec database pg_dump -U loom loom > backup.sql
```

## MCP server (optional)

Loom can expose your graph to AI agents over the
[Model Context Protocol](https://modelcontextprotocol.io). It's part of
the backend - no extra container - and stays switched off unless you
enable it. Skip this section if you don't use AI agents.

### Enabling it

Add to `.env`:

```sh
MCP_ENABLED=true
PUBLIC_URL=https://loom.example.com   # exactly the address you open Loom at
```

and restart: `docker compose up -d`. The server is then at
`https://loom.example.com/mcp`, and **Settings → Connections** in Loom
shows that URL, lets you create access tokens, and lists connected apps.

`PUBLIC_URL` matters: it's the OAuth issuer agents sign in against, and
the MCP endpoint only answers requests whose `Host` is that domain (or
`localhost`/`127.0.0.1`).

Check it's up - this should return `401` with a `WWW-Authenticate`
header, not `404`:

```sh
curl -i -X POST https://loom.example.com/mcp
```

### Reverse proxy

Everything MCP-related (`/mcp`, `/.well-known/oauth-*`, `/oauth/authorize`,
`/api/oauth/*`) goes through the same frontend container as the rest of
the app, so the single upstream from
[Self-hosting on a VPS](#self-hosting-on-a-vps) still covers it. Two
additions are recommended:

- `proxy_set_header Host $host;` - so the backend sees your real domain.
- a `location = /mcp` block with buffering off and a long read timeout,
  since MCP responses can stream.

A complete host nginx config:

```nginx
server {
    listen 443 ssl;
    server_name loom.example.com;
    # ssl_certificate / ssl_certificate_key - e.g. managed by certbot

    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location = /mcp {
        proxy_pass http://127.0.0.1:8081;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_buffering off;
        proxy_read_timeout 1h;
    }
}
```

HTTPS is required in practice: Claude's connectors only talk to `https`
servers. If you already route `/.well-known/` somewhere else (e.g. a
certbot `/.well-known/acme-challenge/` block), make sure
`/.well-known/oauth-*` still reaches Loom.

### Connecting Claude

**Claude (claude.ai, Desktop, Cowork)** - add Loom as a custom connector:

1. Settings → Connectors → **Add custom connector**.
2. URL: `https://loom.example.com/mcp`. Leave the OAuth fields empty -
   Claude registers itself.
3. Claude opens Loom's consent page (sign in first if asked) - check
   the app name and click **Approve**.

**Claude Code** - install the bundled plugin, which adds the server and
the `loom` skill (domain model, tool guide, learning-path recipe):

```sh
/plugin marketplace add duckysmacky/skills
/plugin install loom@duckysmacky     # asks for your Loom URL
```

then run `/mcp`, pick `loom`, and authenticate in the browser. Or add
just the server by hand:

```sh
claude mcp add --transport http loom https://loom.example.com/mcp
# then /mcp inside Claude Code to sign in
```

On a headless machine, use an access token instead (Settings →
Connections → Access tokens):

```sh
claude mcp add --transport http loom https://loom.example.com/mcp \
  --header "Authorization: Bearer loom_..."
```

The skill also works in claude.ai/Cowork: zip
[`claude-plugin/skills/loom`](claude-plugin/skills/loom) and upload it in
Claude's skill settings. See [`claude-plugin/`](claude-plugin) for
details.

### Connecting other agents

Any MCP client that speaks Streamable HTTP works. Point it at
`https://loom.example.com/mcp` and either let it do the OAuth flow
(discovery via `/.well-known/oauth-protected-resource/mcp`, dynamic
client registration, PKCE) or send a personal access token as
`Authorization: Bearer loom_...`.

To poke at it by hand, the MCP Inspector works well:

```sh
npx @modelcontextprotocol/inspector
```

### What agents can do

The tools map one-to-one onto what the web app can do - same
validation, same rules:

| Area | Tools |
|---|---|
| Read | `loom_get_overview`, `loom_get_graph`, `loom_list_nodes`, `loom_get_node` |
| Nodes | `loom_create_node`, `loom_update_node` (incl. promoting), `loom_delete_node`, `loom_poke_node` |
| Build in bulk | `loom_create_subgraph` - many nodes plus their edges in one all-or-nothing call |
| Edges | `loom_create_edge`, `loom_delete_edge` |
| Checklists | `loom_add_checklist_items`, `loom_update_checklist_item`, `loom_delete_checklist_item` |
| Topics | `loom_list_topics`, `loom_create_topic`, `loom_update_topic`, `loom_delete_topic`, `loom_attach_topic`, `loom_detach_topic` |

### Security

- Anything you connect gets **full access to your graph** - it can
  delete as well as create. Only approve apps you recognise.
- Access tokens and app connections are separate from your login
  session and can be revoked any time in **Settings → Connections**;
  changing your password does not revoke them.
- Tokens are stored hashed. OAuth access tokens expire after an hour
  and are refreshed by the client; personal tokens last until revoked.
- The OAuth consent page only redirects to `https` URLs (or `http` on
  localhost) that the app registered.
- Turning `MCP_ENABLED` off removes the endpoint and the OAuth routes
  entirely; existing tokens stop working until it's turned back on.

## Development

```sh
docker compose up -d database        # just the database
cd backend && cargo run              # needs .env with DB_HOST=localhost
cd frontend && npm run dev           # proxies /api (and /mcp) to :8080
```

```sh
cd backend && cargo test             # needs a live DATABASE_URL
cd frontend && npm run check && npm test
```

See `CLAUDE.md` for conventions, domain model, and repo layout.

## License

[AGPL-3.0](LICENSE)
