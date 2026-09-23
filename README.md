# Loom

Self-hosted, single-user web app for tracking projects, courses, and
ideas as a connected graph. See `docs/PROJECT_OVERVIEW.md` for the
domain model.

## Stack

Rust/Axum backend (`backend/`), SvelteKit frontend (`frontend/`),
Postgres. See `backend/CLAUDE.md` and `frontend/CLAUDE.md` for
per-side conventions.

## Running with Docker

```sh
cp .env.example .env
docker compose up --build
```

Serves the API at `http://localhost:8080` (`APP_PORT` in `.env`), with
a bundled Postgres. To use an external Postgres instead, point the
`DB_*` vars in `.env` at it and run just the app:

```sh
docker compose up --build app
```

## Running locally

```sh
docker compose up -d postgres   # or point DB_HOST at an external instance
cd backend
cargo run
```

Requires `DB_HOST=localhost` (or your external host) in `.env`.

Rebuild the compile-time query cache after changing a `query!`/
`query_as!` call, with the database reachable:

```sh
cd backend
cargo sqlx prepare
```
