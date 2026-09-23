# Graph Report - loom  (2026-09-23)

## Corpus Check
- 168 files · ~42,383 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 933 nodes · 2140 edges · 85 communities (51 shown, 34 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 19 edges (avg confidence: 0.74)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8369b414`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Auth Handlers + AuthUser + ApiError
- Topic Handlers (CRUD)
- Node Domain Enums and DTOs
- ApiError Type
- Node Handlers and Integration Tests
- Dashboard Handler
- Edge Domain Models
- Edge Handlers and Integration Tests
- Axum Router Builder + Topic Tests
- Auth Integration Tests
- Dashboard Integration Tests
- Poke Handlers and Integration Tests
- Board Handler
- Poke Domain Models + Repository
- Board Integration Tests
- Frontend TypeScript Config
- User Repository Layer
- Frontend Lint/Format Tooling
- Project Conventions (CLAUDE.md rules)
- JWT Issue/Verify
- Refresh Token Repository
- Node-Topic Linking Repository
- Frontend package.json Scripts
- Argon2id Password Hashing
- Opaque Refresh Token Generation
- App Config Loading
- main.rs Boot Sequence
- PATCH Double-Option Deserialize Helper
- Blueprint Light Canvas Design
- Docker Compose Services
- Frontend package.json Metadata
- Auth Security Model
- Frontend Vitest Example (greet)
- Frontend Vitest Example (Welcome.svelte)
- sqlx Offline Query Cache
- ESLint Dependency
- ESLint JS Config Dependency
- ESLint Config File
- Globals Dependency
- Playwright Dependency
- Prettier Dependency
- Prettier Svelte Plugin
- svelte-check Dependency
- Adapter-Auto Dependency
- Node Types Dependency
- TypeScript Dependency
- TypeScript-ESLint Dependency
- Vite Dependency
- Vitest Dependency
- Vitest Browser Playwright Dependency
- Prettier Config File
- SvelteKit App Types
- SvelteKit Root Layout
- View-Shaped REST Endpoints
- Login Rate Limiting Policy
- Migration-on-Boot Policy
- Rust mod.rs (handlers)
- AuthUser Self-Extractor Ref
- Rust mod.rs (middleware)
- Rust mod.rs (models)
- Rust mod.rs (auth)
- Graphify Skill Directive
- SvelteKit Root Page
- robots.txt Crawl Policy
- Vite Config
- Loom Project Overview
- Canvas node kind/status badges and progress bar
- robots.txt (allow-all crawling)
- Loom (self-hosted graph tracker app)
- @xyflow/svelte canvas view
- Per-edge-kind connector styling (requires/part_of/related)
- Import ts-rs generated types (no hand-written duplicates)
- npm dev/build/preview workflow
- sv CLI Svelte project scaffolding
- Docker Compose run workflow
- sqlx query cache rebuild workflow
- @xyflow/svelte

## God Nodes (most connected - your core abstractions)
1. `AppState` - 44 edges
2. `AuthUser` - 33 edges
3. `send()` - 28 edges
4. `$lib/graph/display` - 28 edges
5. `send()` - 27 edges
6. `req()` - 26 edges
7. `$lib/api/endpoints` - 26 edges
8. `app()` - 25 edges
9. `signup()` - 25 edges
10. `$lib/stores/graph.svelte` - 25 edges

## Surprising Connections (you probably didn't know these)
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/board.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/dashboard.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/edges.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/nodes.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/pokes.rs → backend/src/app.rs

## Import Cycles
- 1-file cycle: `backend/src/handlers/extract.rs -> backend/src/handlers/extract.rs`

## Hyperedges (group relationships)
- **Backend-complete-before-frontend build sequencing** — claude_md_build_plan_sequencing, backend_claude_md_axum_api, frontend_claude_md_sveltekit_static_adapter [INFERRED 0.85]
- **ts-rs backend-to-frontend type generation pipeline** — backend_claude_md_ts_rs_type_generation, frontend_claude_md_ts_rs_type_import, readme_stack [EXTRACTED 1.00]
- **Docker Compose app + Postgres deployment topology** — docker_compose_app_service, docker_compose_postgres_service, readme_docker_compose_workflow [EXTRACTED 1.00]

## Communities (85 total, 34 thin omitted)

### Community 0 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.08
Nodes (60): canvas(), ApiError, Json, Result, State, get(), ApiError, Json (+52 more)

### Community 1 - "Topic Handlers (CRUD)"
Cohesion: 0.09
Nodes (41): create(), delete(), list(), map_edge_error(), ApiError, Error, Json, Result (+33 more)

### Community 2 - "Node Domain Enums and DTOs"
Cohesion: 0.14
Nodes (34): AttachTopicRequest, CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView (+26 more)

### Community 3 - "ApiError Type"
Cohesion: 0.17
Nodes (32): build_refresh_cookie(), change_password(), clear_refresh_cookie(), is_valid_email(), issue_tokens(), login(), logout(), map_create_user_error() (+24 more)

### Community 4 - "Node Handlers and Integration Tests"
Cohesion: 0.30
Nodes (28): app(), blocked_reflects_requires_target_status(), container_progress_reflects_children(), create_edge(), create_edge_ownership_scoping(), create_list_delete_happy_path(), create_node(), delete_edge_ownership_scoping() (+20 more)

### Community 5 - "Dashboard Handler"
Cohesion: 0.28
Nodes (34): app(), attach_and_detach_topic_are_idempotent(), attach_and_detach_topic_on_another_users_node_returns_404(), attach_topic_belonging_to_another_user_returns_404(), canvas_position_is_null_until_set_and_round_trips(), color_must_be_a_hex_code(), create_get_update_delete_happy_path(), create_node() (+26 more)

### Community 6 - "Edge Domain Models"
Cohesion: 0.24
Nodes (15): DashboardCounts, DashboardResponse, KindCounts, NodeResponse, Vec, StatusCounts, get_counts(), get_dashboard() (+7 more)

### Community 7 - "Edge Handlers and Integration Tests"
Cohesion: 0.36
Nodes (22): app(), create_node(), create_poke_happy_path(), deleting_node_cascades_pokes(), get_node(), last_poked_at_reflects_max_and_null_when_never_poked(), list_nodes_agrees_with_get_node_on_last_poked_at(), list_pokes_ordered_newest_first() (+14 more)

### Community 8 - "Axum Router Builder + Topic Tests"
Cohesion: 0.16
Nodes (40): build_router(), governor_error_response(), Response, Router, app(), app_with_signup_policy(), bearer_request(), change_password_enforces_length_rules() (+32 more)

### Community 9 - "Auth Integration Tests"
Cohesion: 0.24
Nodes (19): CreateTopicRequest, DateTime, Option, String, Utc, Uuid, TopicResponse, UpdateTopicRequest (+11 more)

### Community 10 - "Dashboard Integration Tests"
Cohesion: 0.29
Nodes (25): add_edge(), app(), backdate_created_at(), backlog_count_and_recent_backlog_cover_only_unpromoted_ideas(), containers_list_open_paths_with_progress(), counts_match_fixtures_exactly(), create_node(), dashboard_is_isolated_per_user() (+17 more)

### Community 11 - "Poke Handlers and Integration Tests"
Cohesion: 0.10
Nodes (24): $lib/graph/display, ACCENT_PALETTE, accentColor(), childrenOf(), daysSince(), DEFAULT_ACCENT, displayKind, MONTHS (+16 more)

### Community 12 - "Board Handler"
Cohesion: 0.33
Nodes (17): app(), create_get_update_delete_happy_path(), duplicate_topic_name_returns_409(), empty_name_returns_400(), missing_or_garbage_token_returns_401(), ownership_scoping_returns_404_for_another_users_topic(), req(), Body (+9 more)

### Community 13 - "Poke Domain Models + Repository"
Cohesion: 0.14
Nodes (24): create(), list(), ApiError, Json, Result, State, StatusCode, Uuid (+16 more)

### Community 14 - "Board Integration Tests"
Cohesion: 0.33
Nodes (16): app(), canvas_is_isolated_per_user(), canvas_returns_callers_nodes_and_edges(), create_node(), missing_or_garbage_token_returns_401(), req(), Body, Option (+8 more)

### Community 15 - "Frontend TypeScript Config"
Cohesion: 0.14
Nodes (13): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, rewriteRelativeImportExtensions (+5 more)

### Community 16 - "User Repository Layer"
Cohesion: 0.33
Nodes (12): String, Uuid, User, create_user(), find_by_email(), find_by_id(), Error, Option (+4 more)

### Community 17 - "Frontend Lint/Format Tooling"
Cohesion: 0.05
Nodes (41): eslint, eslint-config-prettier, @eslint/js, eslint-plugin-svelte, devDependencies, eslint, eslint-config-prettier, @eslint/js (+33 more)

### Community 19 - "JWT Issue/Verify"
Cohesion: 0.29
Nodes (11): Claims, issue_access_token(), issue_then_verify_roundtrips(), DecodingKey, EncodingKey, Error, Result, String (+3 more)

### Community 20 - "Refresh Token Repository"
Cohesion: 0.40
Nodes (10): attach_topic(), AttachOutcome, detach_topic(), DetachOutcome, Error, PgPool, Result, Uuid (+2 more)

### Community 21 - "Node-Topic Linking Repository"
Cohesion: 0.42
Nodes (12): consume(), ConsumeOutcome, insert(), revoke(), revoke_all_for_user(), revoke_single(), DateTime, Error (+4 more)

### Community 22 - "Frontend package.json Scripts"
Cohesion: 0.06
Nodes (32): @dagrejs/dagre, dompurify, @fontsource/space-grotesk, @fontsource/space-mono, dependencies, @dagrejs/dagre, dompurify, @fontsource/space-grotesk (+24 more)

### Community 23 - "Argon2id Password Hashing"
Cohesion: 0.32
Nodes (10): hash_password(), hash_password_blocking(), hash_then_verify_roundtrips(), Error, Result, String, verify_password(), verify_password_blocking() (+2 more)

### Community 24 - "Opaque Refresh Token Generation"
Cohesion: 0.16
Nodes (22): svelte/elements, $lib/api/endpoints, boardApi, dashboardApi, edgesApi, nodesApi, topicsApi, ./DetailConnections.svelte (+14 more)

### Community 25 - "App Config Loading"
Cohesion: 0.48
Nodes (5): generate(), generate_is_random(), generate_returns_matching_raw_and_hash(), hash_token(), String

### Community 26 - "main.rs Boot Sequence"
Cohesion: 0.35
Nodes (7): Config, require_env(), require_jwt_secret(), Result, Self, String, validate_jwt_secret()

### Community 27 - "PATCH Double-Option Deserialize Helper"
Cohesion: 0.43
Nodes (6): connect_with_retries(), main(), PgPool, Result, shutdown_signal(), PgConnectOptions

### Community 28 - "Blueprint Light Canvas Design"
Cohesion: 0.29
Nodes (6): deserialize_some(), Error, Option, Result, T, D

### Community 29 - "Docker Compose Services"
Cohesion: 0.10
Nodes (13): $lib/components/CommandPalette.svelte, $lib/components/Toaster.svelte, $lib/components/ui/Modal.svelte, fuzzyScore(), $lib/stores/ui.svelte, overlays, LoomFlowEdge, LoomFlowNode (+5 more)

### Community 31 - "Auth Security Model"
Cohesion: 0.12
Nodes (17): @xyflow/svelte/dist/base.css, GraphStore, $lib/stores/toasts.svelte, dismiss(), notify(), notifyError(), Toast, ToastAction (+9 more)

### Community 32 - "Frontend Vitest Example (greet)"
Cohesion: 0.50
Nodes (4): docker-compose app service, postgres pg_isready healthcheck, postgres_data volume, docker-compose postgres service

### Community 33 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.17
Nodes (13): $lib/components/QuickCapture.svelte, $lib/navigation, closeNode(), openNode(), withParam(), $lib/stores/prefs.svelte, applyAppearance(), darkQuery (+5 more)

### Community 35 - "ESLint Dependency"
Cohesion: 0.22
Nodes (12): shortDate(), $lib/graph/timeline, barSpan(), percentOf(), pokeOffset(), now, startOfWeek(), timelineWindow (+4 more)

### Community 36 - "ESLint JS Config Dependency"
Cohesion: 0.26
Nodes (11): ApiJson<T>, ApiPath<T>, ApiQuery<T>, FromRequestParts, Parts, Rejection, Request, Result (+3 more)

### Community 38 - "Globals Dependency"
Cohesion: 0.19
Nodes (7): $lib/components/ui/SegmentedControl.svelte, selected, $lib/components/ui/Tabs.svelte, selected, chosen, remove(), ./settings.css

### Community 39 - "Playwright Dependency"
Cohesion: 0.24
Nodes (9): $lib/components/ui/ProgressBar.svelte, $lib/graph/layout, flowDirection(), layoutPositions(), Point, blocked, dashed, dimmed (+1 more)

### Community 41 - "Prettier Svelte Plugin"
Cohesion: 0.25
Nodes (6): ApiError, Error, From, Response, Self, String

### Community 42 - "svelte-check Dependency"
Cohesion: 0.25
Nodes (6): Access control, Code conventions, Commits, Sequencing, Skills, Testing

### Community 43 - "Adapter-Auto Dependency"
Cohesion: 0.29
Nodes (6): API shape, Auth, CLAUDE.md - backend, Database, Structure, Types

### Community 44 - "Node Types Dependency"
Cohesion: 0.33
Nodes (5): Canvas, CLAUDE.md - frontend, Shared state, Styling, Types

### Community 45 - "TypeScript Dependency"
Cohesion: 0.33
Nodes (5): Deployment, Loom, Running locally, Running with Docker, Stack

### Community 46 - "TypeScript-ESLint Dependency"
Cohesion: 0.40
Nodes (4): Development, Layout, Loom frontend, Scripts

### Community 55 - "Migration-on-Boot Policy"
Cohesion: 0.09
Nodes (29): @fontsource/space-grotesk/400.css, @fontsource/space-grotesk/500.css, @fontsource/space-grotesk/600.css, @fontsource/space-grotesk/700.css, $lib/api/client, ApiError, NO_RETRY_PATHS, onSessionExpired() (+21 more)

## Knowledge Gaps
- **152 isolated node(s):** `gitignorePath`, `name`, `private`, `version`, `type` (+147 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **34 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `Auth Handlers + AuthUser + ApiError` to `Topic Handlers (CRUD)`, `ApiError Type`, `Axum Router Builder + Topic Tests`, `Poke Domain Models + Repository`, `Docker Compose Services`?**
  _High betweenness centrality (0.475) - this node is a cross-community bridge._
- **Why does `build_router()` connect `Axum Router Builder + Topic Tests` to `Auth Handlers + AuthUser + ApiError`, `Node Handlers and Integration Tests`, `Dashboard Handler`, `Edge Handlers and Integration Tests`, `Dashboard Integration Tests`, `Board Handler`, `Board Integration Tests`?**
  _High betweenness centrality (0.262) - this node is a cross-community bridge._
- **Why does `$lib/components/detail/NodeDetail.svelte` connect `Opaque Refresh Token Generation` to `Frontend Vitest Example (Welcome.svelte)`, `Globals Dependency`, `Poke Handlers and Integration Tests`, `Docker Compose Services`, `Auth Security Model`?**
  _High betweenness centrality (0.155) - this node is a cross-community bridge._
- **What connects `gitignorePath`, `name`, `private` to the rest of the system?**
  _152 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Auth Handlers + AuthUser + ApiError` be split into smaller, more focused modules?**
  _Cohesion score 0.0782608695652174 - nodes in this community are weakly interconnected._
- **Should `Topic Handlers (CRUD)` be split into smaller, more focused modules?**
  _Cohesion score 0.08888888888888889 - nodes in this community are weakly interconnected._
- **Should `Node Domain Enums and DTOs` be split into smaller, more focused modules?**
  _Cohesion score 0.14414414414414414 - nodes in this community are weakly interconnected._