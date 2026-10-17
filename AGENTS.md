# AGENTS.md

Loom: self-hosted, single-user web app tracking projects, studies and ideas as a
connected graph. One graph per account; no collaboration.

## Stack

- **Backend:** Rust, Axum, `sqlx` (compile-time-checked, no ORM), Postgres.
- **Frontend:** SvelteKit + TypeScript, `@xyflow/svelte` canvas, `adapter-static` behind its own nginx container.
- **Types:** `ts-rs` generates frontend TS types from Rust request/response structs.
- **Auth:** Argon2id password, short-lived JWT, rotated refresh token in httpOnly `Secure` cookie, rate-limited auth endpoints.
- **MCP:** optional (`MCP_ENABLED` + `PUBLIC_URL`) stateless Streamable HTTP server at `/mcp` via `rmcp`. Auth by opaque `mcp_tokens` (personal `loom_...` or OAuth 2.1 access tokens) - never web-session JWTs, and these tokens never work on `/api`.
- **Deploy:** three Compose services (backend, frontend, database), images on GHCR, behind a host nginx that terminates TLS.

## Layout

- `backend/src/handlers/` - parse/validate, map errors, call `repo/`. Thin; no `sqlx` here. `oauth.rs` holds OAuth discovery, registration, consent, token.
- `backend/src/repo/` - only place touching `sqlx`. Every fn takes `user_id` first.
- `backend/src/models/` - request/response structs (`TS`-derived).
- `backend/src/auth/`, `middleware/` - hashing/JWT/refresh rotation; `AuthUser` extractor.
- `backend/src/mcp/` - bearer middleware + `loom_*` tools. Tools call REST handlers as plain fns, never `repo/`.
- `backend/migrations/` - one file per change, auto-run via `sqlx::migrate!`.
- `frontend/src/lib/{api,graph,stores,components,types}/` - HTTP client (single-flight refresh), board logic (each with a spec), stores, components (`detail/`, `ui/`), generated types (gitignored, never hand-edited).
- `claude-plugin/` - MCP config + `loom` skill. Keep skill in sync when tools or domain rules change.
- `.github/workflows/` - `test.yml` (PRs, called by release), `release.yml` (GHCR on `master` pushes and `v*` tags; drafts release on tags).

## Domain model

- **Kinds:** `idea` (uncommitted; no status/focus/progress/checklist - DB CHECK enforces status/focus), `project` (only kind with checklist), `study` (only kind with `progress_current/total/unit`), `path` (only container; nodes join via `part_of`; nestable).
- **Promote:** change `kind` in place (same id, same edges); data the new kind can't hold is dropped. Idea -> other gets `queued`/`secondary`; demoting to idea clears both and closes the open period. The `idea` *status* is an ordinary status of committed nodes.
- **Focus:** `primary`/`secondary`/`background`, orthogonal to status; orders dashboard active list.
- **Blocked:** derived - unmet `requires` edge to a non-`done` node (idea = not done).
- **Progress (all derived):** container = done ratio of a path's `part_of` children; checklist = project items; path = all study counters + checklists inside (nested included). Distinct from a study's own counter.
- **Path rollups:** status (active > paused > queued/empty > all-archived > done), last touched, started/completed derived recursively from inner nodes, skipping ideas. Paths have no pokes, periods or writable status.
- **Poke:** append-only "worked on this" log, decoupled from `updated_at`. Last touched = `MAX(poked_at)`.
- **Active period:** `started_at -> ended_at` (null = ongoing). First opens on first activation; with "Track active periods", pause/archive closes and reactivation opens a new one. Records node focus at open; editable by hand, or (opt-in pref) focus change starts a new period. Started/Completed derived from first/last period; editing them moves period edges.
- **Stale:** last poke (or creation) older than threshold (default 14d). Active always; queued (default only if poked before) and paused (default never) follow per-browser prefs sent as `GET /api/dashboard` query params.
- **Accent:** user-assigned node color. **Unplaced:** `canvas_x/y` NULL - side panel, unless "Auto-place on canvas".
- **Edges** (directed forward from edited node): `requires` (blocking, no cycles), `part_of` (child -> path, no cycles, one path per node), `precedes` (soft order, never blocks, no cycles), `related` (soft, cycles ok).

## Core rules

- All queries scoped by `user_id` from the verified JWT via middleware; repo fns take `user_id` first. This is the security model (no roles).
- Derived state is computed at query time, never stored; node reads go through the `node_rows` view.
- Kind capabilities enforced in handlers (400s) and DB constraints - never relax one alone.
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

