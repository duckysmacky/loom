# AGENTS.md

Loom: self-hosted, single-user web app tracking projects, studies and ideas as a
connected graph. One graph per account; no collaboration.

## Stack

- **Backend:** Rust, Axum, `sqlx` (compile-time-checked, no ORM), Postgres.
- **Frontend:** SvelteKit + TypeScript, `@xyflow/svelte` canvas, `adapter-static` behind its own nginx container.
- **Types:** `ts-rs` generates frontend TS types from Rust request/response structs.
- **Auth:** Argon2id password, short-lived JWT, rotated refresh token in httpOnly `Secure` cookie, rate-limited auth endpoints.
- **MCP:** optional (`MCP_ENABLED` + `PUBLIC_URL`) stateless Streamable HTTP server at `/mcp` via `rmcp`. Auth by opaque `access_tokens` (personal `loom_...` or OAuth 2.1 access tokens) - never web-session JWTs.
- **API access tokens:** personal `loom_...` tokens (Settings > Connections; scope `read`/`read_write`, optional expiry) also work as bearer on the *data* routes of `/api` (nodes, topics, edges, checklist, periods, canvas, board, dashboard) - never on account routes (`/auth`, `/tokens`, `/oauth`). OAuth tokens work on `/mcp` only. `CORS_ORIGINS` (exact origins, no `*`) opts `/api` in to cross-origin browser calls; off when empty.
- **Deploy:** three Compose services (backend, frontend, database), images on GHCR, behind a host nginx that terminates TLS.

## Layout

- `backend/src/handlers/` - parse/validate, map errors, call `repo/`. Thin; no `sqlx` here. `oauth.rs` holds OAuth discovery, registration, consent, token.
- `backend/src/repo/` - only place touching `sqlx`. Every fn takes `user_id` first.
- `backend/src/models/` - request/response structs (`TS`-derived).
- `backend/src/auth/`, `middleware/` - hashing/JWT/refresh rotation; `AuthUser` extractor; `api_token` layer (personal token -> `AuthUser`, scope by HTTP method).
- `backend/src/mcp/` - bearer middleware + `loom_*` tools. Tools call REST handlers as plain fns, never `repo/`.
- `backend/migrations/` - one file per change, auto-run via `sqlx::migrate!`.
- `frontend/src/lib/{api,graph,stores,components,types}/` - HTTP client (single-flight refresh), board logic (each with a spec), stores, components (`detail/`, `ui/`), generated types (gitignored, never hand-edited).
- `claude-plugin/` - MCP config + `loom` skill. Keep skill in sync when tools or domain rules change.
- `.github/workflows/` - `test.yml` (PRs, called by release), `release.yml` (GHCR on `master` pushes and `v*` tags; drafts release on tags).
- `.context/` - gitignored agent/working notes (`API.md` REST reference, `PROJECT_OVERVIEW.md`, `PROMPT.md`, `IDEAS.md`, `plans/`). Local only, never committed.

## Domain model

- **Kinds:** `idea` (uncommitted; no status/focus/progress/checklist - DB CHECK enforces status/focus), `project` (only kind with checklist), `study` (only kind with `progress_current/total/unit`), `path` (only container; nodes join via `part_of`; nestable).
- **Promote:** change `kind` in place (same id, same edges); data the new kind can't hold is dropped. Idea -> other gets `queued`/`secondary`; demoting to idea clears both and closes the open period. The `idea` *status* is an ordinary status of committed nodes.
- **Archived:** boolean flag on any node (any kind/status, ideas and paths included), independent of status and periods; hides it from board/dashboard. Archiving/restoring a path does the same to everything inside it (nested paths, ideas included).
- **Focus:** `primary`/`secondary`/`background`, orthogonal to status; orders dashboard active list.
- **Blocked:** derived - unmet `requires` edge to a non-`done` node (idea = not done).
- **Progress (all derived):** container = done ratio of a path's `part_of` children; checklist = project items; path = all study counters + checklists inside (nested included). Distinct from a study's own counter.
- **Path rollups:** status (active > paused > queued/empty > done), last touched, started/completed derived recursively from inner nodes, skipping ideas. Paths have no pokes, periods or writable status.
- **Poke:** append-only "worked on this" log, decoupled from `updated_at`. Last touched = `MAX(poked_at)`.
- **Active period:** `started_at -> ended_at` (null = ongoing). First opens on first activation; with "Track active periods", pause closes and reactivation opens a new one. Records node focus at open; editable by hand, or (opt-in pref) focus change starts a new period. Started/Completed derived from first/last period; editing them moves period edges.
- **Stale:** recorded in `nodes.stale_since` by a sweep (`repo/staleness.rs`: every 10 min, plus after pokes, node PATCHes and settings saves) when the last poke (or creation) passes `stale_days` (default 14). Account settings pick kinds (project/study) and queued/paused/idea-status modes (active always; done, archived, idea kind never). Only a poke bringing the last touch back within the threshold (backdated ones too) clears it - status changes don't. Optional auto-pause fires once on entry; paths inherit per `path_stale_mode`. Dashboard also warns `stale_warn_days` ahead.
- **Accent:** user-assigned node color. **Unplaced:** `canvas_x/y` NULL - side panel, unless "Auto-place on canvas".
- **Edges** (directed forward from edited node): `requires` (blocking, no cycles), `part_of` (child -> path, no cycles, one path per node), `precedes` (soft order, never blocks, no cycles), `related` (soft, cycles ok).

## Core rules

- All queries scoped by `user_id` from the verified JWT via middleware; repo fns take `user_id` first. This is the security model (no roles).
- Derived state is computed at query time, never stored; node reads go through the `node_rows` view. One deliberate exception: `stale_since` (needs history for once-only auto-pause).
- Kind capabilities enforced in handlers (400s) and DB constraints - never relax one alone.
- Account routes (login, password, tokens, OAuth clients) must stay session-JWT-only: only data routes get the `api_token` layer in `app.rs`. A new data route goes inside `data_routes`; a new write MCP tool needs the `read_only_denied` guard (a test enforces it).
- Never edit an applied migration; add a new one.
- After changing a query: `cargo sqlx prepare` in `backend/`, commit `.sqlx/` (CI builds offline).
- Regenerate types with `npm run types` in `frontend/`. Missing type -> `#[derive(TS)]` on the Rust struct, never redeclare.
- Frontend graph store refetches the whole graph after any mutation.

## Code conventions

- Descriptive names; no single-letter vars except true loop counters.
- Rust: `rustfmt`/`clippy` clean, `snake_case`. TypeScript: `camelCase` - don't force Rust naming across the API boundary.

## Testing

Auth flows and ownership-scoping are security-critical: need real tests, not happy-path checks. Rest can be looser.
Backend `backend/tests/*.rs` use `#[sqlx::test]`, need live `DATABASE_URL`. Frontend: Vitest `*.spec.ts` beside modules.

```sh
cd backend && cargo test       # needs DATABASE_URL
cd frontend && npm run check   # type-check
cd frontend && npm test        # vitest
```

## Running

```sh
docker compose up -d database   # local: DB only
cd backend && cargo run         # .env with DB_HOST=localhost
cd frontend && npm run dev      # :5173, proxies /api to :8080
docker compose up --build       # full stack: frontend :8081, API :8080 (cp .env.example .env first)
```

## Git

- New big feature = new feature branch off the `dev` branch, with fitting prefix (`feature/`, `fix/`, `refactor/`, `docs/`, `chore/`)
- Commit in reasonable, logical commits - not one mono commit. Message: `<Verb> <what>` ("Add '/user' API endpoints"). Body only for big/complex change, 1-2 sentences.
- When asked to merge: **squash-merge** the branch to keep history short (`git merge --squash`, then one commit).
- Worktrees: `.worktrees/<branch-name>`, gitignored.

## Tooling

Use `/graphify` when working on the project and keep it updated. If anything is architecturally ambiguous, ask rather than guess.

