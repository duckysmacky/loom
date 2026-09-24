# Graph Report - loom  (2026-09-24)

## Corpus Check
- 200 files · ~55,052 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1063 nodes · 2555 edges · 95 communities (61 shown, 34 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 27 edges (avg confidence: 0.7)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ee7822ec`
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
- send
- fixtures.ts
- $lib/stores/filters.svelte
- +page.svelte
- nodes.rs
- update
- ApiPath
- +page.svelte
- create
- $lib/navigation

## God Nodes (most connected - your core abstractions)
1. `AppState` - 49 edges
2. `$lib/graph/display` - 39 edges
3. `AuthUser` - 38 edges
4. `send()` - 31 edges
5. `$lib/api/endpoints` - 31 edges
6. `req()` - 30 edges
7. `app()` - 29 edges
8. `signup()` - 29 edges
9. `send()` - 28 edges
10. `$lib/stores/graph.svelte` - 28 edges

## Surprising Connections (you probably didn't know these)
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/board.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/checklist.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/dashboard.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/edges.rs → backend/src/app.rs
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/nodes.rs → backend/src/app.rs

## Import Cycles
- 1-file cycle: `backend/src/handlers/extract.rs -> backend/src/handlers/extract.rs`
- 1-file cycle: `frontend/src/routes/(app)/board/organized/PathBox.svelte -> frontend/src/routes/(app)/board/organized/PathBox.svelte`

## Hyperedges (group relationships)
- **Backend-complete-before-frontend build sequencing** — claude_md_build_plan_sequencing, backend_claude_md_axum_api, frontend_claude_md_sveltekit_static_adapter [INFERRED 0.85]
- **Docker Compose app + Postgres deployment topology** — docker_compose_app_service, docker_compose_postgres_service, readme_docker_compose_workflow [EXTRACTED 1.00]

## Communities (95 total, 34 thin omitted)

### Community 0 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.12
Nodes (22): canvas(), ApiError, Json, Result, State, get(), ApiError, Json (+14 more)

### Community 1 - "Topic Handlers (CRUD)"
Cohesion: 0.13
Nodes (31): CanvasResponse, NodeResponse, Vec, CreateEdgeRequest, EdgeKind, EdgeListQuery, EdgeResponse, DateTime (+23 more)

### Community 2 - "Node Domain Enums and DTOs"
Cohesion: 0.14
Nodes (34): AttachTopicRequest, CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView (+26 more)

### Community 3 - "ApiError Type"
Cohesion: 0.17
Nodes (32): build_refresh_cookie(), change_password(), clear_refresh_cookie(), is_valid_email(), issue_tokens(), login(), logout(), map_create_user_error() (+24 more)

### Community 4 - "Node Handlers and Integration Tests"
Cohesion: 0.28
Nodes (34): a_node_sits_in_one_path_but_paths_nest(), a_path_that_changes_kind_releases_its_nodes(), app(), blocked_reflects_requires_target_status(), canvas_size_is_a_positive_pair(), container_progress_reflects_children(), create_edge(), create_edge_ownership_scoping() (+26 more)

### Community 5 - "Dashboard Handler"
Cohesion: 0.26
Nodes (38): app(), attach_and_detach_topic_are_idempotent(), attach_and_detach_topic_on_another_users_node_returns_404(), attach_topic_belonging_to_another_user_returns_404(), canvas_position_is_null_until_set_and_round_trips(), color_must_be_a_hex_code(), create_get_update_delete_happy_path(), create_node() (+30 more)

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
Nodes (25): add_edge(), app(), backdate_created_at(), backlog_count_and_recent_backlog_cover_every_status_idea_node(), counts_match_fixtures_exactly(), create_node(), dashboard_is_isolated_per_user(), missing_or_garbage_token_returns_401() (+17 more)

### Community 11 - "Poke Handlers and Integration Tests"
Cohesion: 0.17
Nodes (19): $lib/graph/display, ACCENT_PALETTE, accentColor(), borderStyle, daysSince(), DEFAULT_ACCENT, fromDateInput(), incrementedProgress() (+11 more)

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
Nodes (32): @dagrejs/dagre, dompurify, @fontsource/ibm-plex-mono, @fontsource/space-grotesk, dependencies, @dagrejs/dagre, dompurify, @fontsource/ibm-plex-mono (+24 more)

### Community 23 - "Argon2id Password Hashing"
Cohesion: 0.32
Nodes (10): hash_password(), hash_password_blocking(), hash_then_verify_roundtrips(), Error, Result, String, verify_password(), verify_password_blocking() (+2 more)

### Community 24 - "Opaque Refresh Token Generation"
Cohesion: 0.14
Nodes (19): $lib/api/endpoints, boardApi, checklistApi, dashboardApi, edgesApi, nodesApi, topicsApi, ./DetailChecklist.svelte (+11 more)

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
Cohesion: 0.17
Nodes (7): $lib/components/ui/SegmentedControl.svelte, selected, $lib/components/ui/Tabs.svelte, selected, chosen, ./settings.css, $app/navigation

### Community 31 - "Auth Security Model"
Cohesion: 0.13
Nodes (14): GraphStore, $lib/stores/toasts.svelte, dismiss(), notify(), notifyError(), Toast, ToastAction, toasts (+6 more)

### Community 32 - "Frontend Vitest Example (greet)"
Cohesion: 0.50
Nodes (4): docker-compose app service, postgres pg_isready healthcheck, postgres_data volume, docker-compose postgres service

### Community 33 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.17
Nodes (12): svelte/easing, $lib/components/CommandPalette.svelte, $lib/components/Toaster.svelte, $lib/components/ui/AnimatedNumber.svelte, fuzzyScore(), $lib/motion, $lib/stores/ui.svelte, overlays (+4 more)

### Community 35 - "ESLint Dependency"
Cohesion: 0.12
Nodes (17): KIND_GLYPH, TIER_COLOR, $lib/graph/grouping, Bucket, FOCUS_BUCKETS, Grouping, groupNodes(), KIND_BUCKETS (+9 more)

### Community 36 - "ESLint JS Config Dependency"
Cohesion: 0.26
Nodes (11): ApiJson<T>, ApiPath<T>, ApiQuery<T>, FromRequestParts, Parts, Rejection, Request, Result (+3 more)

### Community 38 - "Globals Dependency"
Cohesion: 0.19
Nodes (22): ChecklistItemResponse, CreateChecklistItemRequest, DateTime, Option, String, Utc, Uuid, UpdateChecklistItemRequest (+14 more)

### Community 39 - "Playwright Dependency"
Cohesion: 0.19
Nodes (13): svelte/animate, $lib/components/NodeCard.svelte, $lib/components/ui/Badge.svelte, $lib/components/ui/ProgressBar.svelte, $lib/graph/order, childrenOf(), isShown(), ordered (+5 more)

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
Cohesion: 0.40
Nodes (4): Deployment, Loom, Running locally, Running with Docker

### Community 46 - "TypeScript-ESLint Dependency"
Cohesion: 0.40
Nodes (4): Development, Layout, Loom frontend, Scripts

### Community 55 - "Migration-on-Boot Policy"
Cohesion: 0.09
Nodes (29): @fontsource/space-grotesk/400.css, @fontsource/space-grotesk/500.css, @fontsource/space-grotesk/600.css, @fontsource/space-grotesk/700.css, $lib/api/client, ApiError, NO_RETRY_PATHS, onSessionExpired() (+21 more)

### Community 85 - "send"
Cohesion: 0.36
Nodes (19): add_item(), another_users_checklist_is_invisible(), app(), blank_titles_are_rejected(), checklist_drives_node_progress(), create_node(), get_node(), items_append_in_order_and_list_back() (+11 more)

### Community 86 - "fixtures.ts"
Cohesion: 0.07
Nodes (34): makeEdge(), makeNode(), nodes, $lib/graph/layout, CanvasPlacement, cardSize, flowDirection(), layoutCanvas() (+26 more)

### Community 87 - "$lib/stores/filters.svelte"
Cohesion: 0.33
Nodes (7): isBacklog(), FilterCriteria, nodeMatches(), anything, $lib/stores/filters.svelte, boardFilters, matchesBoardFilters()

### Community 88 - "+page.svelte"
Cohesion: 0.24
Nodes (3): @xyflow/svelte/dist/base.css, active, ./types

### Community 89 - "nodes.rs"
Cohesion: 0.28
Nodes (20): attach_topic(), clean_progress_unit(), create(), delete(), detach_topic(), get(), list(), map_node_error() (+12 more)

### Community 90 - "update"
Cohesion: 0.24
Nodes (17): ApiError, Result, validate_color(), create(), delete(), get(), list(), map_topic_error() (+9 more)

### Community 91 - "ApiPath"
Cohesion: 0.30
Nodes (16): clean_title(), create(), delete(), list(), ApiError, Json, Result, State (+8 more)

### Community 92 - "+page.svelte"
Cohesion: 0.19
Nodes (11): svelte/elements, $lib/components/PromoteDialog.svelte, $lib/components/QuickCapture.svelte, $lib/components/ui/Button.svelte, $lib/components/ui/Modal.svelte, LoomFlowEdge, LoomFlowNode, $lib/types/EdgeKind (+3 more)

### Community 93 - "create"
Cohesion: 0.29
Nodes (12): create(), delete(), list(), map_edge_error(), ApiError, Error, Json, Result (+4 more)

### Community 94 - "$lib/navigation"
Cohesion: 0.83
Nodes (4): $lib/navigation, closeNode(), openNode(), withParam()

## Knowledge Gaps
- **157 isolated node(s):** `gitignorePath`, `name`, `private`, `version`, `type` (+152 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **34 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `Auth Handlers + AuthUser + ApiError` to `ApiError Type`, `Axum Router Builder + Topic Tests`, `Poke Domain Models + Repository`, `+page.svelte`, `nodes.rs`, `update`, `ApiPath`, `create`?**
  _High betweenness centrality (0.539) - this node is a cross-community bridge._
- **Why does `build_router()` connect `Axum Router Builder + Topic Tests` to `Auth Handlers + AuthUser + ApiError`, `Node Handlers and Integration Tests`, `Dashboard Handler`, `Edge Handlers and Integration Tests`, `Dashboard Integration Tests`, `Board Handler`, `Board Integration Tests`, `send`?**
  _High betweenness centrality (0.302) - this node is a cross-community bridge._
- **Why does `$lib/components/detail/NodeDetail.svelte` connect `Opaque Refresh Token Generation` to `Frontend Vitest Example (Welcome.svelte)`, `Playwright Dependency`, `Poke Handlers and Integration Tests`, `+page.svelte`, `+page.svelte`, `Docker Compose Services`, `$lib/navigation`, `Auth Security Model`?**
  _High betweenness centrality (0.184) - this node is a cross-community bridge._
- **What connects `gitignorePath`, `name`, `private` to the rest of the system?**
  _157 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Auth Handlers + AuthUser + ApiError` be split into smaller, more focused modules?**
  _Cohesion score 0.1164021164021164 - nodes in this community are weakly interconnected._
- **Should `Topic Handlers (CRUD)` be split into smaller, more focused modules?**
  _Cohesion score 0.12605042016806722 - nodes in this community are weakly interconnected._
- **Should `Node Domain Enums and DTOs` be split into smaller, more focused modules?**
  _Cohesion score 0.14414414414414414 - nodes in this community are weakly interconnected._