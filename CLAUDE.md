# CLAUDE.md

See `docs/PROJECT_OVERVIEW.md` for the product/domain model and
`docs/BUILD_PLAN.md` for the phased build order. Read both before
starting any work.

## Sequencing

Follow `docs/BUILD_PLAN.md` phase by phase, in order. Do not start
frontend phases before every backend phase is checked off - the frontend
is built against a working API, not alongside it. After finishing a
phase, check it off in `docs/BUILD_PLAN.md`, commit the changes and start
the next phase.

## Commits

Small and atomic - one logical change per commit, never a single commit
for an entire phase. Commit messages should be specific enough to read
as a changelog on their own.

For commit rules check global CLAUDE.md at `~/.claude/CLAUDE.md`.

## Code conventions

- Descriptive, meaningful names everywhere — no single-letter variables
  except genuine loop counters with no broader meaning.
- Rust: standard `rustfmt`/`clippy`-clean, `snake_case`.
- TypeScript: standard `camelCase` - don't force Rust naming style across
  the API boundary.

## Access control

Every node/edge/topic/poke query is scoped by `user_id`, extracted once
via middleware from the verified JWT. Repository-layer functions take
`user_id` as their first argument, always - this is the actual security
model (no roles/permissions system exists), so never write a query that
skips it.

## Testing

Auth flows and ownership-scoping are the security-critical paths - these
need real test coverage, not just happy-path manual checks. Everything
else can be tested more loosely at this stage.

If anything in the build plan is architecturally ambiguous, ask rather
than guessing - this is a from-scratch design, not a refactor of
existing code.

## Skills

Use the /graphify when working on the project and update it accordingly.

