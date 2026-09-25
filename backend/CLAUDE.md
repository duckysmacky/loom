# CLAUDE.md - backend

Rust/Axum API. See the root `CLAUDE.md` for commit conventions and the
domain model this implements.

## Structure

Keep handlers, the repository layer, and domain models in separate
modules - handlers parse/validate and call into the repository layer;
the repository layer is the only place that touches `sqlx` directly.

## Database

- `sqlx` with compile-time-checked queries (`query!`/`query_as!`). Check
  the `.sqlx` query cache into the repo so CI doesn't need a live
  database to build.
- After adding or changing any `query!`/`query_as!` call - including in
  `tests/*.rs`, not just `src/` - run
  `cargo sqlx prepare -- --tests` (with `DATABASE_URL` set to a live
  Postgres) and commit the updated `.sqlx/` files. Plain
  `cargo sqlx prepare` (no `-- --tests`) only scans lib/bin targets and
  will silently *delete* cache entries that test files still need, since
  it treats them as stale - CI then fails offline with "no cached data
  for this query" on the very next PR that touches `tests/`, even one
  that changes nothing else. `cargo sqlx prepare --check -- --tests`
  verifies the cache is complete without regenerating it.
- Migrations live in `migrations/`, run automatically on boot via
  `sqlx::migrate!`.
- `blocked` status and `part_of` container progress are computed in
  queries, never written to a column — don't add a `blocked` or
  container `progress` column to the schema.

## Auth

Argon2id for password hashing. Short-lived JWT access token; refresh
token rotated on use and set as an httpOnly, secure, `SameSite=Strict`
cookie - never returned in a JSON body. Rate-limit `/login` specifically.

## API shape

REST. Most endpoints are plain resource CRUD, but `/board/canvas`,
`/dashboard`, and `/nodes` are deliberately view-shaped composite
endpoints — return exactly what that view needs in one response rather
than making the frontend assemble it from several calls.

## Types

Derive `ts-rs`'s `TS` trait on every request/response struct that
crosses the API boundary, and regenerate the frontend's type files as
part of the build - the frontend should never hand-declare an interface
that duplicates a Rust struct.

