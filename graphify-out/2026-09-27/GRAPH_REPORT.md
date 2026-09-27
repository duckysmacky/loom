# Graph Report - loom  (2026-09-25)

## Corpus Check
- 259 files · ~79,717 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1497 nodes · 4046 edges · 102 communities (65 shown, 37 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 65 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `235c4d39`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- nodes.rs
- fixtures.ts
- Topic Handlers (CRUD)
- Axum Router Builder + Topic Tests
- Auth Integration Tests
- Frontend Lint/Format Tooling
- Node Domain Enums and DTOs
- Dashboard Handler
- ApiError Type
- Node Handlers and Integration Tests
- Frontend package.json Scripts
- Opaque Refresh Token Generation
- Migration-on-Boot Policy
- Poke Handlers and Integration Tests
- +page.svelte
- Poke Domain Models + Repository
- Adapter-Auto Dependency
- Dashboard Integration Tests
- Edge Domain Models
- Globals Dependency
- Edge Handlers and Integration Tests
- Auth Security Model
- send
- ESLint Dependency
- Board Handler
- Board Integration Tests
- Auth Security Model
- Frontend Vitest Example (Welcome.svelte)
- ESLint JS Config Dependency
- Docker Compose Services
- Frontend TypeScript Config
- Node-Topic Linking Repository
- JWT Issue/Verify
- Argon2id Password Hashing
- $lib/stores/filters.svelte
- main.rs Boot Sequence
- Refresh Token Repository
- User Repository Layer
- Prettier Svelte Plugin
- App Config Loading
- PATCH Double-Option Deserialize Helper
- Blueprint Light Canvas Design
- User Repository Layer
- Frontend Vitest Example (Welcome.svelte)
- Prettier Dependency
- View-Shaped REST Endpoints
- Login Rate Limiting Policy
- Vite Dependency
- ESLint JS Config Dependency
- ESLint JS Config Dependency
- ESLint JS Config Dependency
- Auth Handlers + AuthUser + ApiError
- Auth Handlers + AuthUser + ApiError
- Accent Color Concept
- Checklist Progress Concept
- Container Progress Concept
- App Favicon
- robots.txt (allow-all crawling)
- @xyflow/svelte
- Loom
- CLAUDE.md
- Loom plugin for Claude
- CLAUDE.md - frontend
- package.json
- Loom frontend
- dompurify
- eslint-config-prettier
- globals
- playwright
- prettier
- prettier-plugin-svelte
- svelte-check
- @types/node
- typescript
- vite
- vitest-browser-svelte
- Password + Argon2id + JWT Auth
- Axum Backend Framework
- Backlog (status = idea)
- Blocked (derived state)
- Focus Tier (primary/secondary/background)
- Node Kinds (idea/project/study/path)
- Part Of Edge (container membership)
- Poke (append-only touch log)
- Promote (in-place kind change)
- Related Edge (soft, cycles allowed)
- Requires Edge (hard, blocking)
- Stale (14-day poke threshold)
- SvelteKit Frontend
- ts-rs Type Generation
- @xyflow/svelte Canvas
- Loom Design System
- Board Views (Dashboard/Organized/Canvas/Timeline)

## God Nodes (most connected - your core abstractions)
1. `AppState` - 85 edges
2. `AuthUser` - 62 edges
3. `$lib/api/endpoints` - 47 edges
4. `$lib/graph/display` - 44 edges
5. `ApiPath` - 39 edges
6. `app()` - 37 edges
7. `signup()` - 37 edges
8. `create_node()` - 35 edges
9. `send()` - 34 edges
10. `req()` - 33 edges

## Surprising Connections (you probably didn't know these)
- `Docker Compose Config` --shares_data_with--> `Release Workflow`  [INFERRED]
  docker-compose.yml → .github/workflows/release.yml
- `Release Workflow` --conceptually_related_to--> `Three-Service Docker Compose Deployment`  [INFERRED]
  .github/workflows/release.yml → CLAUDE.md
- `Test Workflow` --shares_data_with--> `sqlx Compile-Time-Checked Queries`  [INFERRED]
  .github/workflows/test.yml → CLAUDE.md
- `Docker Compose Config` --implements--> `Three-Service Docker Compose Deployment`  [INFERRED]
  docker-compose.yml → CLAUDE.md
- `app()` --calls--> `build_router()`  [INFERRED]
  backend/tests/active_periods.rs → backend/src/app.rs

## Import Cycles
- 1-file cycle: `backend/src/handlers/extract.rs -> backend/src/handlers/extract.rs`
- 1-file cycle: `frontend/src/routes/(app)/board/organized/PathBox.svelte -> frontend/src/routes/(app)/board/organized/PathBox.svelte`

## Hyperedges (group relationships)
- **Docker Compose Deployment Pipeline** — github_workflows_release, docker_compose, readme [INFERRED 0.80]
- **CI Test & Build Flow** — github_workflows_test, github_workflows_release, backend_claude, frontend_readme [INFERRED 0.75]
- **Cross-Stack Type Contract via ts-rs** — claude_ts_rs, backend_claude, frontend_claude, frontend_readme [INFERRED 0.85]

## Communities (102 total, 37 thin omitted)

### Community 0 - "nodes.rs"
Cohesion: 0.05
Nodes (89): canvas(), ApiError, Json, Result, State, timeline(), get(), ApiError (+81 more)

### Community 1 - "fixtures.ts"
Cohesion: 0.13
Nodes (20): makeEdge(), makeNode(), makePeriod(), nodes, $lib/graph/layout, CanvasPlacement, cardSize, flowDirection() (+12 more)

### Community 2 - "Topic Handlers (CRUD)"
Cohesion: 0.08
Nodes (49): create(), delete(), list(), map_edge_error(), ApiError, Error, Json, Result (+41 more)

### Community 3 - "Axum Router Builder + Topic Tests"
Cohesion: 0.23
Nodes (34): app(), app_with_signup_policy(), bearer_request(), change_password_enforces_length_rules(), change_password_replaces_the_login_credential(), change_password_requires_a_bearer_token(), change_password_signs_out_other_sessions_but_keeps_this_one(), change_password_with_wrong_current_password_is_400_and_changes_nothing() (+26 more)

### Community 4 - "Auth Integration Tests"
Cohesion: 0.13
Nodes (36): ApiError, Result, validate_color(), create(), delete(), get(), list(), map_topic_error() (+28 more)

### Community 5 - "Frontend Lint/Format Tooling"
Cohesion: 0.10
Nodes (21): eslint, @eslint/js, eslint-plugin-svelte, devDependencies, eslint, @eslint/js, eslint-plugin-svelte, svelte (+13 more)

### Community 6 - "Node Domain Enums and DTOs"
Cohesion: 0.14
Nodes (37): AttachTopicRequest, CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView (+29 more)

### Community 7 - "Dashboard Handler"
Cohesion: 0.21
Nodes (49): app(), archived_from_active_closes_period_but_leaves_completed_at_null(), attach_and_detach_topic_are_idempotent(), attach_and_detach_topic_on_another_users_node_returns_404(), attach_topic_belonging_to_another_user_returns_404(), canvas_position_is_null_until_set_and_round_trips(), color_must_be_a_hex_code(), continuous_active_to_done_creates_zero_periods() (+41 more)

### Community 8 - "ApiError Type"
Cohesion: 0.17
Nodes (32): build_refresh_cookie(), change_password(), clear_refresh_cookie(), is_valid_email(), issue_tokens(), login(), logout(), map_create_user_error() (+24 more)

### Community 9 - "Node Handlers and Integration Tests"
Cohesion: 0.28
Nodes (34): a_node_sits_in_one_path_but_paths_nest(), a_path_that_changes_kind_releases_its_nodes(), app(), blocked_reflects_requires_target_status(), canvas_size_is_a_positive_pair(), container_progress_reflects_children(), create_edge(), create_edge_ownership_scoping() (+26 more)

### Community 10 - "Frontend package.json Scripts"
Cohesion: 0.14
Nodes (14): scripts, build, check, check:watch, dev, format, lint, prebuild (+6 more)

### Community 11 - "Opaque Refresh Token Generation"
Cohesion: 0.09
Nodes (27): @xyflow/svelte/dist/base.css, svelte/elements, $lib/api/endpoints, boardApi, checklistApi, dashboardApi, mcpApi, nodesApi (+19 more)

### Community 12 - "Migration-on-Boot Policy"
Cohesion: 0.12
Nodes (23): $lib/api/client, ApiError, NO_RETRY_PATHS, onSessionExpired(), postRefresh(), queryString(), refreshAccessToken(), request() (+15 more)

### Community 13 - "Poke Handlers and Integration Tests"
Cohesion: 0.10
Nodes (29): requestKindChange(), $lib/graph/display, ACCENT_PALETTE, accentColor(), borderStyle, daysSince(), DEFAULT_ACCENT, fromDateInput() (+21 more)

### Community 14 - "+page.svelte"
Cohesion: 0.14
Nodes (17): svelte/animate, $lib/components/NodeCard.svelte, $lib/components/ui/Badge.svelte, $lib/components/ui/ProgressBar.svelte, isBacklog(), FilterCriteria, nodeMatches(), anything (+9 more)

### Community 15 - "Poke Domain Models + Repository"
Cohesion: 0.09
Nodes (40): create(), list(), ApiError, Json, Result, State, StatusCode, Uuid (+32 more)

### Community 16 - "Adapter-Auto Dependency"
Cohesion: 0.18
Nodes (11): API shape, Auth, CLAUDE.md - backend, Database, Structure, Types, Three-Service Docker Compose Deployment, sqlx Compile-Time-Checked Queries (+3 more)

### Community 17 - "Dashboard Integration Tests"
Cohesion: 0.29
Nodes (25): add_edge(), app(), backdate_created_at(), backlog_count_and_recent_backlog_cover_every_status_idea_node(), counts_match_fixtures_exactly(), create_node(), dashboard_is_isolated_per_user(), missing_or_garbage_token_returns_401() (+17 more)

### Community 18 - "Edge Domain Models"
Cohesion: 0.14
Nodes (43): ApiJson, ApiPath, ApiQuery, T, AddChecklistItemsParams, auth(), ChecklistItemIdParams, compact() (+35 more)

### Community 19 - "Globals Dependency"
Cohesion: 0.14
Nodes (34): clean_title(), create(), delete(), list(), ApiError, Json, Result, State (+26 more)

### Community 20 - "Edge Handlers and Integration Tests"
Cohesion: 0.36
Nodes (22): app(), create_node(), create_poke_happy_path(), deleting_node_cascades_pokes(), get_node(), last_poked_at_reflects_max_and_null_when_never_poked(), list_nodes_agrees_with_get_node_on_last_poked_at(), list_pokes_ordered_newest_first() (+14 more)

### Community 21 - "Auth Security Model"
Cohesion: 0.13
Nodes (14): edgesApi, moveBefore(), GraphStore, notifyError(), applyDrop(), confirmKindDrop(), dragState, dropAtEnd() (+6 more)

### Community 22 - "send"
Cohesion: 0.36
Nodes (19): add_item(), another_users_checklist_is_invisible(), app(), blank_titles_are_rejected(), checklist_drives_node_progress(), create_node(), get_node(), items_append_in_order_and_list_back() (+11 more)

### Community 23 - "ESLint Dependency"
Cohesion: 0.08
Nodes (22): @fontsource/space-grotesk/400.css, @fontsource/space-grotesk/500.css, @fontsource/space-grotesk/600.css, @fontsource/space-grotesk/700.css, SvelteKit app.html Shell, $lib/components/ui/AccentPicker.svelte, chosen, $lib/components/ui/SegmentedControl.svelte (+14 more)

### Community 24 - "Board Handler"
Cohesion: 0.33
Nodes (17): app(), create_get_update_delete_happy_path(), duplicate_topic_name_returns_409(), empty_name_returns_400(), missing_or_garbage_token_returns_401(), ownership_scoping_returns_404_for_another_users_topic(), req(), Body (+9 more)

### Community 25 - "Board Integration Tests"
Cohesion: 0.34
Nodes (17): app(), canvas_is_isolated_per_user(), canvas_returns_callers_nodes_and_edges(), create_node(), missing_or_garbage_token_returns_401(), req(), Body, Option (+9 more)

### Community 26 - "Auth Security Model"
Cohesion: 0.22
Nodes (8): dismiss(), notify(), blockedPrimaryCount, captureIdea(), poke(), tiles, remove(), $lib/types/DashboardResponse

### Community 27 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.08
Nodes (31): svelte/easing, $lib/components/CommandPalette.svelte, ./DetailActivePeriods.svelte, ./DetailChecklist.svelte, $lib/components/detail/NodeDetail.svelte, $lib/components/PromoteDialog.svelte, $lib/components/QuickCapture.svelte, $lib/components/Toaster.svelte (+23 more)

### Community 28 - "ESLint JS Config Dependency"
Cohesion: 0.26
Nodes (11): ApiJson<T>, ApiPath<T>, ApiQuery<T>, FromRequestParts, Parts, Rejection, Request, Result (+3 more)

### Community 29 - "Docker Compose Services"
Cohesion: 0.14
Nodes (34): create(), delete(), list(), map_period_error(), ApiError, Error, Json, Result (+26 more)

### Community 30 - "Frontend TypeScript Config"
Cohesion: 0.14
Nodes (13): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, rewriteRelativeImportExtensions (+5 more)

### Community 31 - "Node-Topic Linking Repository"
Cohesion: 0.42
Nodes (12): consume(), ConsumeOutcome, insert(), revoke(), revoke_all_for_user(), revoke_single(), DateTime, Error (+4 more)

### Community 32 - "JWT Issue/Verify"
Cohesion: 0.29
Nodes (11): Claims, issue_access_token(), issue_then_verify_roundtrips(), DecodingKey, EncodingKey, Error, Result, String (+3 more)

### Community 33 - "Argon2id Password Hashing"
Cohesion: 0.32
Nodes (10): hash_password(), hash_password_blocking(), hash_then_verify_roundtrips(), Error, Result, String, verify_password(), verify_password_blocking() (+2 more)

### Community 34 - "$lib/stores/filters.svelte"
Cohesion: 0.25
Nodes (34): app(), approve(), authorize_body(), authorize_rejects_unregistered_redirect_and_missing_pkce(), builder(), cannot_see_or_revoke_another_users_clients(), challenge(), code_is_bound_to_client_and_redirect_uri() (+26 more)

### Community 35 - "main.rs Boot Sequence"
Cohesion: 0.29
Nodes (10): Config, require_env(), require_jwt_secret(), require_mcp_public_url(), Option, Result, Self, String (+2 more)

### Community 36 - "Refresh Token Repository"
Cohesion: 0.40
Nodes (10): attach_topic(), AttachOutcome, detach_topic(), DetachOutcome, Error, PgPool, Result, Uuid (+2 more)

### Community 37 - "User Repository Layer"
Cohesion: 0.33
Nodes (12): String, Uuid, User, create_user(), find_by_email(), find_by_id(), Error, Option (+4 more)

### Community 38 - "Prettier Svelte Plugin"
Cohesion: 0.21
Nodes (8): ApiError, Error, From, IntoResponse, Response, Self, StatusCode, String

### Community 39 - "App Config Loading"
Cohesion: 0.09
Nodes (41): generate(), generate_is_random(), generate_returns_matching_raw_and_hash(), hash_token(), String, create(), delete(), info() (+33 more)

### Community 40 - "PATCH Double-Option Deserialize Helper"
Cohesion: 0.43
Nodes (6): connect_with_retries(), main(), PgPool, Result, shutdown_signal(), PgConnectOptions

### Community 41 - "Blueprint Light Canvas Design"
Cohesion: 0.29
Nodes (6): deserialize_some(), Error, Option, Result, T, D

### Community 42 - "User Repository Layer"
Cohesion: 0.12
Nodes (23): shortDate(), $lib/graph/timeline, barSpan(), clampZoom(), nodeBarSpans(), periodSpans(), pokeOffsets(), rangeWidth() (+15 more)

### Community 43 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.06
Nodes (30): About, AI agents (MCP), Backlog & promoting, Backups, Configuration, Connecting Claude, Connecting other agents, Connections (+22 more)

### Community 50 - "ESLint JS Config Dependency"
Cohesion: 0.14
Nodes (28): AuthorizeDecision, AuthorizePreview, AuthorizeRedirect, AuthorizeRequest, ConnectedClientResponse, RegisterClientRequest, DateTime, Option (+20 more)

### Community 51 - "ESLint JS Config Dependency"
Cohesion: 0.26
Nodes (26): app(), call(), cannot_touch_another_users_nodes(), create_subgraph_builds_a_learning_path(), disabled_by_default(), error_text(), failing_subgraph_leaves_nothing_behind(), is_error() (+18 more)

### Community 52 - "ESLint JS Config Dependency"
Cohesion: 0.19
Nodes (23): build_router(), governor_error_response(), Response, Router, app(), cannot_see_or_delete_another_users_tokens(), create_list_delete_happy_path(), info_reports_disabled_by_default() (+15 more)

### Community 56 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.36
Nodes (18): add_period(), another_users_periods_are_invisible(), app(), create_node(), create_rejects_ended_before_started(), periods_are_created_listed_updated_and_deleted(), req(), Body (+10 more)

### Community 57 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.15
Nodes (13): @dagrejs/dagre, @fontsource/ibm-plex-mono, @fontsource/jetbrains-mono, @fontsource/space-grotesk, dependencies, @dagrejs/dagre, @fontsource/ibm-plex-mono, @fontsource/jetbrains-mono (+5 more)

### Community 67 - "Loom"
Cohesion: 0.17
Nodes (10): Client side, Connecting to Loom, Server side (instance owner), Troubleshooting, Domain model, Loom, Other common requests, Recipe: build a learning path (+2 more)

### Community 68 - "CLAUDE.md"
Cohesion: 0.20
Nodes (8): Code conventions, Core rules, Domain model & terminology, Repo layout, Running the project, Skills, Stack, Testing

### Community 69 - "Loom plugin for Claude"
Cohesion: 0.33
Nodes (5): Claude.ai, Desktop and Cowork, Contents, Install (Claude Code), Loom plugin for Claude, Marketplace entry

### Community 70 - "CLAUDE.md - frontend"
Cohesion: 0.33
Nodes (5): Canvas, CLAUDE.md - frontend, Shared state, Styling, Types

### Community 71 - "package.json"
Cohesion: 0.40
Nodes (4): name, private, type, version

### Community 72 - "Loom frontend"
Cohesion: 0.40
Nodes (4): Development, Layout, Loom frontend, Scripts

### Community 73 - "dompurify"
Cohesion: 0.50
Nodes (3): dompurify, dompurify, renderMarkdown()

## Knowledge Gaps
- **188 isolated node(s):** `gitignorePath`, `name`, `private`, `version`, `type` (+183 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **37 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `nodes.rs` to `Topic Handlers (CRUD)`, `Auth Integration Tests`, `App Config Loading`, `ApiError Type`, `Poke Domain Models + Repository`, `Edge Domain Models`, `Globals Dependency`, `ESLint JS Config Dependency`, `Frontend Vitest Example (Welcome.svelte)`, `Docker Compose Services`?**
  _High betweenness centrality (0.524) - this node is a cross-community bridge._
- **Why does `build_router()` connect `ESLint JS Config Dependency` to `nodes.rs`, `$lib/stores/filters.svelte`, `Axum Router Builder + Topic Tests`, `Dashboard Handler`, `Node Handlers and Integration Tests`, `Dashboard Integration Tests`, `ESLint JS Config Dependency`, `Edge Handlers and Integration Tests`, `send`, `Auth Handlers + AuthUser + ApiError`, `Board Integration Tests`, `Board Handler`?**
  _High betweenness centrality (0.279) - this node is a cross-community bridge._
- **Why does `$lib/components/detail/NodeDetail.svelte` connect `Frontend Vitest Example (Welcome.svelte)` to `Opaque Refresh Token Generation`, `Poke Handlers and Integration Tests`, `+page.svelte`, `ESLint Dependency`?**
  _High betweenness centrality (0.134) - this node is a cross-community bridge._
- **Are the 15 inferred relationships involving `ApiPath` (e.g. with `.add_checklist_items()` and `.create_node()`) actually correct?**
  _`ApiPath` has 15 INFERRED edges - model-reasoned connections that need verification._
- **What connects `gitignorePath`, `name`, `private` to the rest of the system?**
  _188 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `nodes.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.052837938760268856 - nodes in this community are weakly interconnected._
- **Should `fixtures.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12962962962962962 - nodes in this community are weakly interconnected._