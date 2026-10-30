# Graph Report - loom  (2026-10-30)

## Corpus Check
- 304 files · ~108,411 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1779 nodes · 5247 edges · 115 communities (76 shown, 39 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 85 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b79f84b7`
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
- canvas.rs
- Frontend Vitest Example (Welcome.svelte)
- Prettier Dependency
- View-Shaped REST Endpoints
- Login Rate Limiting Policy
- Vite Dependency
- TopicResponse
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
- oauth.rs
- state.rs
- AppState
- AuthUser
- create
- mcp_tokens.rs
- extract.rs
- types.ts
- @sveltejs/kit
- typescript-eslint
- @vitest/browser-playwright
- @lucide/svelte

## God Nodes (most connected - your core abstractions)
1. `AppState` - 98 edges
2. `AuthUser` - 69 edges
3. `$lib/graph/display` - 55 edges
4. `$lib/api/endpoints` - 54 edges
5. `app()` - 48 edges
6. `send()` - 48 edges
7. `signup()` - 48 edges
8. `req()` - 47 edges
9. `ApiPath` - 46 edges
10. `$lib/stores/graph.svelte` - 41 edges

## Surprising Connections (you probably didn't know these)
- `Docker Compose Config` --shares_data_with--> `Release Workflow`  [INFERRED]
  docker-compose.yml → .github/workflows/release.yml
- `Release Workflow` --conceptually_related_to--> `Three-Service Docker Compose Deployment`  [INFERRED]
  .github/workflows/release.yml → CLAUDE.md
- `Test Workflow` --shares_data_with--> `sqlx Compile-Time-Checked Queries`  [INFERRED]
  .github/workflows/test.yml → CLAUDE.md
- `Docker Compose Config` --implements--> `Three-Service Docker Compose Deployment`  [INFERRED]
  docker-compose.yml → CLAUDE.md
- `build_router()` --calls--> `patch()`  [INFERRED]
  backend/src/app.rs → backend/tests/nodes.rs

## Import Cycles
- 1-file cycle: `backend/src/handlers/extract.rs -> backend/src/handlers/extract.rs`
- 1-file cycle: `frontend/src/routes/(app)/board/organized/PathBox.svelte -> frontend/src/routes/(app)/board/organized/PathBox.svelte`

## Hyperedges (group relationships)
- **Docker Compose Deployment Pipeline** — github_workflows_release, docker_compose, readme [INFERRED 0.80]
- **CI Test & Build Flow** — github_workflows_test, github_workflows_release, backend_claude, frontend_readme [INFERRED 0.75]
- **Cross-Stack Type Contract via ts-rs** — claude_ts_rs, backend_claude, frontend_claude, frontend_readme [INFERRED 0.85]

## Communities (115 total, 39 thin omitted)

### Community 0 - "nodes.rs"
Cohesion: 0.28
Nodes (23): attach_topic(), clean_progress_unit(), clear_order(), create(), delete(), detach_topic(), get(), list() (+15 more)

### Community 1 - "fixtures.ts"
Cohesion: 0.12
Nodes (19): edgesApi, nodesApi, makeEdge(), $lib/graph/order, dependencyOrder(), manualOrder(), moveBefore(), notifyError() (+11 more)

### Community 2 - "Topic Handlers (CRUD)"
Cohesion: 0.07
Nodes (53): create(), delete(), list(), map_edge_error(), ApiError, Error, Json, Result (+45 more)

### Community 3 - "Axum Router Builder + Topic Tests"
Cohesion: 0.23
Nodes (34): app(), app_with_signup_policy(), bearer_request(), change_password_enforces_length_rules(), change_password_replaces_the_login_credential(), change_password_requires_a_bearer_token(), change_password_signs_out_other_sessions_but_keeps_this_one(), change_password_with_wrong_current_password_is_400_and_changes_nothing() (+26 more)

### Community 4 - "Auth Integration Tests"
Cohesion: 0.17
Nodes (18): DashboardCounts, DashboardQuery, DashboardResponse, KindCounts, NodeResponse, Option, Vec, StaleMode (+10 more)

### Community 5 - "Frontend Lint/Format Tooling"
Cohesion: 0.09
Nodes (23): eslint, @eslint/js, eslint-plugin-svelte, devDependencies, eslint, @eslint/js, eslint-plugin-svelte, playwright (+15 more)

### Community 6 - "Node Domain Enums and DTOs"
Cohesion: 0.07
Nodes (81): create(), delete(), list(), map_period_error(), ApiError, Error, Json, Result (+73 more)

### Community 7 - "Dashboard Handler"
Cohesion: 0.18
Nodes (64): app(), archiving_keeps_status_and_period_and_works_on_any_kind(), attach_and_detach_topic_are_idempotent(), attach_and_detach_topic_on_another_users_node_returns_404(), attach_topic_belonging_to_another_user_returns_404(), canvas_position_is_null_until_set_and_round_trips(), color_must_be_a_hex_code(), create_get_update_delete_happy_path() (+56 more)

### Community 8 - "ApiError Type"
Cohesion: 0.17
Nodes (32): build_refresh_cookie(), change_password(), clear_refresh_cookie(), is_valid_email(), issue_tokens(), login(), logout(), map_create_user_error() (+24 more)

### Community 9 - "Node Handlers and Integration Tests"
Cohesion: 0.23
Nodes (46): a_node_sits_in_one_path_but_paths_nest(), a_path_that_changes_kind_releases_its_nodes(), add_checklist_item(), app(), archiving_a_path_archives_and_restores_everything_inside(), blocked_reflects_requires_target_status(), canvas_size_is_a_positive_pair(), container_progress_reflects_children() (+38 more)

### Community 10 - "Frontend package.json Scripts"
Cohesion: 0.14
Nodes (14): scripts, build, check, check:watch, dev, format, lint, prebuild (+6 more)

### Community 11 - "Opaque Refresh Token Generation"
Cohesion: 0.12
Nodes (36): create(), delete(), list(), ApiError, Json, Result, State, StatusCode (+28 more)

### Community 12 - "Migration-on-Boot Policy"
Cohesion: 0.06
Nodes (45): @fontsource/space-grotesk/400.css, @fontsource/space-grotesk/500.css, @fontsource/space-grotesk/600.css, @fontsource/space-grotesk/700.css, $lib/api/client, ApiError, NO_RETRY_PATHS, onSessionExpired() (+37 more)

### Community 13 - "Poke Handlers and Integration Tests"
Cohesion: 0.13
Nodes (25): show(), $lib/graph/display, ACCENT_PALETTE, accentColor(), borderStyle, canPoke(), combineDateTime(), daysSince() (+17 more)

### Community 14 - "+page.svelte"
Cohesion: 0.21
Nodes (12): svelte/animate, $lib/components/canvas/CanvasNode.svelte, dimmed, path, $lib/components/canvas/CanvasPath.svelte, $lib/components/canvas/SideHandles.svelte, $lib/components/NodeCard.svelte, path (+4 more)

### Community 15 - "Poke Domain Models + Repository"
Cohesion: 0.12
Nodes (28): create(), list(), ApiError, Json, Option, Result, State, StatusCode (+20 more)

### Community 16 - "Adapter-Auto Dependency"
Cohesion: 0.18
Nodes (11): API shape, Auth, CLAUDE.md - backend, Database, Structure, Types, Three-Service Docker Compose Deployment, sqlx Compile-Time-Checked Queries (+3 more)

### Community 17 - "Dashboard Integration Tests"
Cohesion: 0.30
Nodes (27): active_list_ranks_by_tier_then_unblocked(), add_edge(), app(), backdate_created_at(), counts_match_fixtures_exactly(), create_node(), dashboard_is_isolated_per_user(), ideas_count_and_recent_ideas_cover_every_idea_kind_node() (+19 more)

### Community 18 - "Edge Domain Models"
Cohesion: 0.12
Nodes (54): ApiJson, ApiPath, ApiQuery, T, AddChecklistItemsParams, ArrowIdParams, auth(), ChecklistItemIdParams (+46 more)

### Community 19 - "Globals Dependency"
Cohesion: 0.19
Nodes (20): ChecklistItemResponse, CreateChecklistItemRequest, DateTime, Option, String, Utc, Uuid, UpdateChecklistItemRequest (+12 more)

### Community 20 - "Edge Handlers and Integration Tests"
Cohesion: 0.29
Nodes (31): app(), backdated_poke_keeps_its_time_and_never_lowers_last_poked_at(), backdated_poke_on_another_users_node_returns_404(), create_kind(), create_node(), create_poke_happy_path(), deleting_node_cascades_pokes(), future_poke_returns_400() (+23 more)

### Community 21 - "Auth Security Model"
Cohesion: 0.09
Nodes (23): svelte/easing, save(), $lib/components/ui/AnimatedNumber.svelte, $lib/navigation, closeNode(), openNode(), withParam(), dismiss() (+15 more)

### Community 22 - "send"
Cohesion: 0.36
Nodes (19): add_item(), another_users_checklist_is_invisible(), app(), blank_titles_are_rejected(), checklist_drives_node_progress(), create_node(), get_node(), items_append_in_order_and_list_back() (+11 more)

### Community 23 - "ESLint Dependency"
Cohesion: 0.07
Nodes (28): SvelteKit app.html Shell, $lib/components/ui/AccentPicker.svelte, chosen, $lib/components/ui/Tabs.svelte, selected, isIdea(), FilterCriteria, nodeMatches() (+20 more)

### Community 24 - "Board Handler"
Cohesion: 0.33
Nodes (17): app(), create_get_update_delete_happy_path(), duplicate_topic_name_returns_409(), empty_name_returns_400(), missing_or_garbage_token_returns_401(), ownership_scoping_returns_404_for_another_users_topic(), req(), Body (+9 more)

### Community 25 - "Board Integration Tests"
Cohesion: 0.34
Nodes (17): app(), canvas_is_isolated_per_user(), canvas_returns_callers_nodes_and_edges(), create_node(), missing_or_garbage_token_returns_401(), req(), Body, Option (+9 more)

### Community 26 - "Auth Security Model"
Cohesion: 0.08
Nodes (41): @xyflow/svelte/dist/base.css, $lib/components/canvas/CanvasEdge.svelte, label, $lib/components/canvas/flow, toFlowEdges(), toFlowNodes(), $lib/components/canvas/types, LoomArrowEdge (+33 more)

### Community 27 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.14
Nodes (41): check_path(), check_position(), check_size(), create_arrow(), create_note(), delete_arrow(), delete_note(), ApiError (+33 more)

### Community 28 - "ESLint JS Config Dependency"
Cohesion: 0.26
Nodes (11): ApiJson<T>, ApiPath<T>, ApiQuery<T>, FromRequestParts, Parts, Rejection, Request, Result (+3 more)

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
Cohesion: 0.24
Nodes (13): Config, parse_cors_origins(), require_env(), require_jwt_secret(), require_mcp_public_url(), Option, Result, Self (+5 more)

### Community 36 - "Refresh Token Repository"
Cohesion: 0.44
Nodes (8): attach_topic(), AttachOutcome, detach_topic(), DetachOutcome, Error, PgPool, Result, Uuid

### Community 37 - "User Repository Layer"
Cohesion: 0.33
Nodes (12): String, Uuid, User, create_user(), find_by_email(), find_by_id(), Error, Option (+4 more)

### Community 38 - "Prettier Svelte Plugin"
Cohesion: 0.21
Nodes (8): ApiError, Error, From, IntoResponse, Response, Self, StatusCode, String

### Community 39 - "App Config Loading"
Cohesion: 0.31
Nodes (21): app(), cannot_see_or_delete_another_users_tokens(), create_list_delete_happy_path(), expired_personal_token_stays_listed_and_survives_oauth_cleanup(), info_reports_disabled_by_default(), info_reports_url_when_enabled(), rejects_blank_name(), rejects_out_of_range_expiry_and_unknown_scope() (+13 more)

### Community 40 - "PATCH Double-Option Deserialize Helper"
Cohesion: 0.43
Nodes (6): connect_with_retries(), main(), PgPool, Result, shutdown_signal(), PgConnectOptions

### Community 41 - "Blueprint Light Canvas Design"
Cohesion: 0.29
Nodes (6): deserialize_some(), Error, Option, Result, T, D

### Community 42 - "canvas.rs"
Cohesion: 0.36
Nodes (21): app(), arrow_needs_one_distinct_item_per_end(), canvas(), create_node(), create_note(), deleting_an_end_removes_its_arrows(), note_crud_round_trip(), note_rejects_tiny_size_and_non_path_parent() (+13 more)

### Community 43 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.06
Nodes (32): About, Backups, Claude Chat and Cowork (as a custom connector), Claude Chat and Cowork (as a plugin, recommended), Claude Code, Configuration, Connecting Claude, Connecting other agents (+24 more)

### Community 50 - "TopicResponse"
Cohesion: 0.13
Nodes (36): ApiError, Result, validate_color(), create(), delete(), get(), list(), map_topic_error() (+28 more)

### Community 51 - "ESLint JS Config Dependency"
Cohesion: 0.24
Nodes (33): app(), call(), cannot_touch_another_users_nodes(), create_subgraph_builds_a_learning_path(), disabled_by_default(), error_text(), failing_subgraph_leaves_nothing_behind(), failing_subgraph_placement_rolls_back() (+25 more)

### Community 52 - "ESLint JS Config Dependency"
Cohesion: 0.09
Nodes (47): build_router(), governor_error_response(), Response, Router, generate(), generate_is_random(), generate_returns_matching_raw_and_hash(), hash_token() (+39 more)

### Community 56 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.37
Nodes (19): add_period(), another_users_periods_are_invisible(), app(), create_node(), create_rejects_ended_before_started(), period_focus_defaults_to_the_node_and_is_editable(), periods_are_created_listed_updated_and_deleted(), req() (+11 more)

### Community 57 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.13
Nodes (15): @dagrejs/dagre, @fontsource/ibm-plex-mono, @fontsource/jetbrains-mono, @fontsource/space-grotesk, dependencies, @dagrejs/dagre, @fontsource/ibm-plex-mono, @fontsource/jetbrains-mono (+7 more)

### Community 67 - "Loom"
Cohesion: 0.14
Nodes (12): Client side, Connecting to Loom, Server side (instance owner), Troubleshooting, Domain model, Loom, Notes and arrows on the canvas, Other common requests (+4 more)

### Community 68 - "CLAUDE.md"
Cohesion: 0.18
Nodes (10): AGENTS.md, Code conventions, Core rules, Domain model, Git, Layout, Running, Stack (+2 more)

### Community 69 - "Loom plugin for Claude"
Cohesion: 0.33
Nodes (5): Claude Code, Claude Web and Desktop, Contents, Install, Loom plugin for Claude

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

### Community 75 - "globals"
Cohesion: 0.28
Nodes (17): consume_code(), ConsumedCode, create_client(), create_code(), find_client(), list_connected(), OAuthClient, revoke_client() (+9 more)

### Community 76 - "playwright"
Cohesion: 0.25
Nodes (6): FromRequestParts, Parts, Rejection, Result, Self, Uuid

### Community 77 - "prettier"
Cohesion: 0.16
Nodes (14): get(), ApiError, Json, Result, State, AppState, DecodingKey, EncodingKey (+6 more)

### Community 79 - "svelte-check"
Cohesion: 0.15
Nodes (12): TIER_COLOR, $lib/graph/grouping, Bucket, FOCUS_BUCKETS, Grouping, groupNodes(), IDEA_BUCKET, KIND_BUCKETS (+4 more)

### Community 82 - "vite"
Cohesion: 0.41
Nodes (12): clean_title(), create(), delete(), list(), ApiError, Json, Result, State (+4 more)

### Community 102 - "oauth.rs"
Cohesion: 0.13
Nodes (35): authorization_server_metadata(), decide(), exchange_code(), exchange_refresh_token(), is_allowed_redirect_uri(), is_this_resource(), list_clients(), mcp_resource() (+27 more)

### Community 103 - "state.rs"
Cohesion: 0.09
Nodes (44): svelte/elements, $lib/components/CommandPalette.svelte, ./DetailActivePeriods.svelte, ./DetailChecklist.svelte, ./DetailConnections.svelte, ./DetailContains.svelte, candidates, ./DetailNotes.svelte (+36 more)

### Community 104 - "AppState"
Cohesion: 0.10
Nodes (26): shortDate(), anything, makeNode(), makePeriod(), $lib/graph/timeline, barSpan(), clampZoom(), nodeBarSpans() (+18 more)

### Community 105 - "AuthUser"
Cohesion: 0.27
Nodes (11): AuthorizeDecision, AuthorizePreview, AuthorizeRedirect, AuthorizeRequest, ConnectedClientResponse, RegisterClientRequest, DateTime, Option (+3 more)

### Community 107 - "create"
Cohesion: 0.44
Nodes (11): app(), cors_is_off_unless_configured(), get_with_origin(), other_origins_get_no_cors_headers(), preflight(), preflight_from_an_allowed_origin_succeeds_without_auth(), HeaderMap, PgPool (+3 more)

### Community 108 - "mcp_tokens.rs"
Cohesion: 0.52
Nodes (6): canvas(), ApiError, Json, Result, State, timeline()

## Knowledge Gaps
- **217 isolated node(s):** `gitignorePath`, `name`, `private`, `version`, `type` (+212 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **39 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `prettier` to `nodes.rs`, `Topic Handlers (CRUD)`, `Node Domain Enums and DTOs`, `oauth.rs`, `ApiError Type`, `state.rs`, `Opaque Refresh Token Generation`, `mcp_tokens.rs`, `playwright`, `Poke Domain Models + Repository`, `vite`, `TopicResponse`, `ESLint JS Config Dependency`, `Edge Domain Models`, `Frontend Vitest Example (Welcome.svelte)`?**
  _High betweenness centrality (0.534) - this node is a cross-community bridge._
- **Why does `build_router()` connect `ESLint JS Config Dependency` to `$lib/stores/filters.svelte`, `Axum Router Builder + Topic Tests`, `App Config Loading`, `Dashboard Handler`, `Node Handlers and Integration Tests`, `canvas.rs`, `create`, `prettier`, `Dashboard Integration Tests`, `ESLint JS Config Dependency`, `Edge Handlers and Integration Tests`, `send`, `Auth Handlers + AuthUser + ApiError`, `Board Integration Tests`, `Board Handler`?**
  _High betweenness centrality (0.276) - this node is a cross-community bridge._
- **Why does `$lib/components/detail/NodeDetail.svelte` connect `state.rs` to `Migration-on-Boot Policy`, `Poke Handlers and Integration Tests`, `+page.svelte`, `Auth Security Model`, `ESLint Dependency`?**
  _High betweenness centrality (0.122) - this node is a cross-community bridge._
- **What connects `gitignorePath`, `name`, `private` to the rest of the system?**
  _217 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `fixtures.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.11666666666666667 - nodes in this community are weakly interconnected._
- **Should `Topic Handlers (CRUD)` be split into smaller, more focused modules?**
  _Cohesion score 0.06666666666666667 - nodes in this community are weakly interconnected._
- **Should `Frontend Lint/Format Tooling` be split into smaller, more focused modules?**
  _Cohesion score 0.08695652173913043 - nodes in this community are weakly interconnected._