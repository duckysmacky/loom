# CLAUDE.md - frontend

SvelteKit + TypeScript, built with `adapter-static` and served by the
Axum backend as static files - not deployed as its own server. See the
root `CLAUDE.md` for commit conventions and `docs/PROJECT_OVERVIEW.md`
for the domain model.

## Types

Import types from the `ts-rs`-generated files — never hand-write an
interface that duplicates a Rust struct on the backend. If a type is
missing, that's a sign the backend struct needs the `TS` derive, not
that it should be redeclared here.

## Canvas

`@xyflow/svelte` for the canvas view. Custom node and edge components
should follow the visual language from the Claude Design mockups: kind
badges (course/project/idea/path), status badges (including a distinct
treatment for `blocked`), a progress bar on nodes with
`progress_current`/`progress_total`, and per-edge-kind connector styling
— `requires` as a strong directional line, `part_of` as
nesting/containment, `related` as a lighter dotted line.

## Shared state

Canvas, organized, and timeline views share one filter/search state (a
Svelte store), not three independent ones — switching views must not
reset what's filtered.

## Styling

Use the "Loom" design system throughout the whole project for frontend
development. It is the main design system for the application.

