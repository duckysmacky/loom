# Loom frontend

SvelteKit + TypeScript SPA, built with `adapter-static` and served by the
Axum backend (see the root `README.md` and `CLAUDE.md`). Visual language
follows the "Loom" design system.

## Development

```sh
# 1. backend on :8080 (from ../backend; needs Postgres, see ../.env.example)
DB_HOST=localhost cargo run

# 2. frontend dev server on :5173, proxying /api to :8080
npm install
npm run dev
```

## Scripts

- `npm run types` - regenerates `src/lib/types/` from the backend's ts-rs
  derives (runs `cargo test export_bindings`). Runs automatically before
  `build` and `check`; the directory is gitignored.
- `npm run check` - svelte-check type checking.
- `npm run lint` / `npm run format` - Prettier + ESLint.
- `npm test` - unit tests (API client token refresh, graph/timeline/filter
  helpers).
- `npm run build` - static output in `build/`.

## Layout

- `src/lib/api` - fetch client (in-memory access token, single-flight
  refresh) and typed endpoint functions.
- `src/lib/stores` - session, the shared graph cache, board filters,
  preferences, overlays, toasts.
- `src/lib/graph` - pure display/layout/timeline/filter helpers.
- `src/routes/(auth)` - login/signup; `src/routes/(app)` - everything
  behind sign-in.
