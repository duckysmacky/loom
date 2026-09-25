# Graph Report - .  (2026-09-25)

## Corpus Check
- 163 files · ~59,135 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1061 nodes · 2640 edges · 67 communities (51 shown, 16 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 50 edges (avg confidence: 0.81)
- Token cost: 86,450 input · 0 output

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

## God Nodes (most connected - your core abstractions)
1. `AppState` - 51 edges
2. `$lib/graph/display` - 41 edges
3. `send()` - 32 edges
4. `req()` - 31 edges
5. `app()` - 30 edges
6. `signup()` - 30 edges
7. `$lib/stores/graph.svelte` - 29 edges
8. `send()` - 28 edges
9. `create_node()` - 28 edges
10. `$lib/types/NodeResponse` - 26 edges

## Surprising Connections (you probably didn't know these)
- `Loom README` --conceptually_related_to--> `Backlog (status = idea)`  [INFERRED]
  README.md → CLAUDE.md
- `Loom README` --conceptually_related_to--> `Blocked (derived state)`  [INFERRED]
  README.md → CLAUDE.md
- `Loom README` --conceptually_related_to--> `Focus Tier (primary/secondary/background)`  [INFERRED]
  README.md → CLAUDE.md
- `Loom README` --conceptually_related_to--> `Node Kinds (idea/project/study/path)`  [INFERRED]
  README.md → CLAUDE.md
- `Loom README` --conceptually_related_to--> `Part Of Edge (container membership)`  [INFERRED]
  README.md → CLAUDE.md

## Import Cycles
- 1-file cycle: `backend/src/handlers/extract.rs -> backend/src/handlers/extract.rs`
- 1-file cycle: `frontend/src/routes/(app)/board/organized/PathBox.svelte -> frontend/src/routes/(app)/board/organized/PathBox.svelte`

## Hyperedges (group relationships)
- **Docker Compose Deployment Pipeline** — github_workflows_release, docker_compose, readme [INFERRED 0.80]
- **CI Test & Build Flow** — github_workflows_test, github_workflows_release, backend_claude, frontend_readme [INFERRED 0.75]
- **Cross-Stack Type Contract via ts-rs** — claude_ts_rs, backend_claude, frontend_claude, frontend_readme [INFERRED 0.85]

## Communities (67 total, 16 thin omitted)

### Community 0 - "nodes.rs"
Cohesion: 0.09
Nodes (57): canvas(), ApiError, Json, Result, State, clean_title(), create(), delete() (+49 more)

### Community 1 - "fixtures.ts"
Cohesion: 0.07
Nodes (34): makeEdge(), makeNode(), nodes, CanvasPlacement, cardSize, flowDirection(), layoutCanvas(), layoutPositions() (+26 more)

### Community 2 - "Topic Handlers (CRUD)"
Cohesion: 0.09
Nodes (44): create(), delete(), list(), map_edge_error(), ApiError, AuthUser, Error, Json (+36 more)

### Community 3 - "Axum Router Builder + Topic Tests"
Cohesion: 0.16
Nodes (40): build_router(), governor_error_response(), Response, Router, app(), app_with_signup_policy(), bearer_request(), change_password_enforces_length_rules() (+32 more)

### Community 4 - "Auth Integration Tests"
Cohesion: 0.13
Nodes (37): ApiError, Result, validate_color(), create(), delete(), get(), list(), map_topic_error() (+29 more)

### Community 5 - "Frontend Lint/Format Tooling"
Cohesion: 0.05
Nodes (41): eslint, eslint-config-prettier, @eslint/js, eslint-plugin-svelte, devDependencies, eslint, eslint-config-prettier, @eslint/js (+33 more)

### Community 6 - "Node Domain Enums and DTOs"
Cohesion: 0.14
Nodes (37): AttachTopicRequest, CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView (+29 more)

### Community 7 - "Dashboard Handler"
Cohesion: 0.26
Nodes (39): app(), attach_and_detach_topic_are_idempotent(), attach_and_detach_topic_on_another_users_node_returns_404(), attach_topic_belonging_to_another_user_returns_404(), canvas_position_is_null_until_set_and_round_trips(), color_must_be_a_hex_code(), create_get_update_delete_happy_path(), create_node() (+31 more)

### Community 8 - "ApiError Type"
Cohesion: 0.17
Nodes (33): build_refresh_cookie(), change_password(), clear_refresh_cookie(), is_valid_email(), issue_tokens(), login(), logout(), map_create_user_error() (+25 more)

### Community 9 - "Node Handlers and Integration Tests"
Cohesion: 0.28
Nodes (34): a_node_sits_in_one_path_but_paths_nest(), a_path_that_changes_kind_releases_its_nodes(), app(), blocked_reflects_requires_target_status(), canvas_size_is_a_positive_pair(), container_progress_reflects_children(), create_edge(), create_edge_ownership_scoping() (+26 more)

### Community 10 - "Frontend package.json Scripts"
Cohesion: 0.06
Nodes (33): @dagrejs/dagre, dompurify, @fontsource/ibm-plex-mono, @fontsource/space-grotesk, dependencies, @dagrejs/dagre, dompurify, @fontsource/ibm-plex-mono (+25 more)

### Community 11 - "Opaque Refresh Token Generation"
Cohesion: 0.11
Nodes (20): @xyflow/svelte/dist/base.css, boardApi, checklistApi, dashboardApi, topicsApi, available, requestKindChange(), $lib/components/ui/Chip.svelte (+12 more)

### Community 12 - "Migration-on-Boot Policy"
Cohesion: 0.12
Nodes (21): ApiError, NO_RETRY_PATHS, onSessionExpired(), postRefresh(), queryString(), refreshAccessToken(), request(), send() (+13 more)

### Community 13 - "Poke Handlers and Integration Tests"
Cohesion: 0.11
Nodes (28): $lib/graph/display, ACCENT_PALETTE, accentColor(), BorderStyle, daysSince(), DEFAULT_ACCENT, fromDateInput(), incrementedProgress() (+20 more)

### Community 14 - "+page.svelte"
Cohesion: 0.14
Nodes (17): svelte/animate, svelte/elements, $lib/components/ui/Badge.svelte, $lib/components/ui/Button.svelte, $lib/components/ui/Modal.svelte, $lib/components/ui/ProgressBar.svelte, ./CanvasNode.svelte, LoomFlowEdge (+9 more)

### Community 15 - "Poke Domain Models + Repository"
Cohesion: 0.14
Nodes (25): create(), list(), ApiError, AuthUser, Json, Result, State, StatusCode (+17 more)

### Community 16 - "Adapter-Auto Dependency"
Cohesion: 0.10
Nodes (27): Backend CLAUDE.md, Loom Root CLAUDE.md, Password + Argon2id + JWT Auth, Axum Backend Framework, Backlog (status = idea), Blocked (derived state), Three-Service Docker Compose Deployment, Focus Tier (primary/secondary/background) (+19 more)

### Community 17 - "Dashboard Integration Tests"
Cohesion: 0.29
Nodes (25): add_edge(), app(), backdate_created_at(), backlog_count_and_recent_backlog_cover_every_status_idea_node(), counts_match_fixtures_exactly(), create_node(), dashboard_is_isolated_per_user(), missing_or_garbage_token_returns_401() (+17 more)

### Community 18 - "Edge Domain Models"
Cohesion: 0.16
Nodes (21): get(), ApiError, AuthUser, Json, Result, State, DashboardCounts, DashboardResponse (+13 more)

### Community 19 - "Globals Dependency"
Cohesion: 0.19
Nodes (22): ChecklistItemResponse, CreateChecklistItemRequest, DateTime, Option, String, Utc, Uuid, UpdateChecklistItemRequest (+14 more)

### Community 20 - "Edge Handlers and Integration Tests"
Cohesion: 0.36
Nodes (22): app(), create_node(), create_poke_happy_path(), deleting_node_cascades_pokes(), get_node(), last_poked_at_reflects_max_and_null_when_never_poked(), list_nodes_agrees_with_get_node_on_last_poked_at(), list_pokes_ordered_newest_first() (+14 more)

### Community 21 - "Auth Security Model"
Cohesion: 0.13
Nodes (14): edgesApi, nodesApi, GraphStore, notifyError(), applyDrop(), confirmKindDrop(), dragState, dropAtEnd() (+6 more)

### Community 22 - "send"
Cohesion: 0.36
Nodes (19): add_item(), another_users_checklist_is_invisible(), app(), blank_titles_are_rejected(), checklist_drives_node_progress(), create_node(), get_node(), items_append_in_order_and_list_back() (+11 more)

### Community 23 - "ESLint Dependency"
Cohesion: 0.12
Nodes (14): @fontsource/space-grotesk/400.css, @fontsource/space-grotesk/500.css, @fontsource/space-grotesk/600.css, @fontsource/space-grotesk/700.css, SvelteKit app.html Shell, Grouping, $lib/stores/prefs.svelte, applyAppearance() (+6 more)

### Community 24 - "Board Handler"
Cohesion: 0.33
Nodes (17): app(), create_get_update_delete_happy_path(), duplicate_topic_name_returns_409(), empty_name_returns_400(), missing_or_garbage_token_returns_401(), ownership_scoping_returns_404_for_another_users_topic(), req(), Body (+9 more)

### Community 25 - "Board Integration Tests"
Cohesion: 0.33
Nodes (16): app(), canvas_is_isolated_per_user(), canvas_returns_callers_nodes_and_edges(), create_node(), missing_or_garbage_token_returns_401(), req(), Body, Option (+8 more)

### Community 26 - "Auth Security Model"
Cohesion: 0.15
Nodes (13): svelte/easing, $lib/stores/toasts.svelte, dismiss(), notify(), Toast, ToastAction, toasts, blockedPrimaryCount (+5 more)

### Community 27 - "Frontend Vitest Example (Welcome.svelte)"
Cohesion: 0.20
Nodes (10): $lib/motion, $lib/stores/ui.svelte, overlays, ./CanvasControls.svelte, section, title, ./Sidebar.svelte, active (+2 more)

### Community 28 - "ESLint JS Config Dependency"
Cohesion: 0.26
Nodes (11): ApiJson<T>, ApiPath<T>, ApiQuery<T>, Request, Result, Self, FromRequest, FromRequestParts (+3 more)

### Community 29 - "Docker Compose Services"
Cohesion: 0.19
Nodes (10): $lib/components/ui/SegmentedControl.svelte, selected, selected, ./AccountTab.svelte, ./AppearanceTab.svelte, ./FunctionalityTab.svelte, ./TopicsTab.svelte, chosen (+2 more)

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
Cohesion: 0.26
Nodes (7): isBacklog(), FilterCriteria, nodeMatches(), anything, boardFilters, matchesBoardFilters(), ./FilterBar.svelte

### Community 35 - "main.rs Boot Sequence"
Cohesion: 0.35
Nodes (7): Config, require_env(), require_jwt_secret(), Result, Self, String, validate_jwt_secret()

### Community 36 - "Refresh Token Repository"
Cohesion: 0.40
Nodes (10): attach_topic(), AttachOutcome, detach_topic(), DetachOutcome, Error, PgPool, Result, Uuid (+2 more)

### Community 37 - "User Repository Layer"
Cohesion: 0.47
Nodes (10): create_user(), find_by_email(), find_by_id(), Error, Option, PgPool, Result, Uuid (+2 more)

### Community 38 - "Prettier Svelte Plugin"
Cohesion: 0.25
Nodes (6): ApiError, Error, From, Response, Self, String

### Community 39 - "App Config Loading"
Cohesion: 0.48
Nodes (5): generate(), generate_is_random(), generate_returns_matching_raw_and_hash(), hash_token(), String

### Community 40 - "PATCH Double-Option Deserialize Helper"
Cohesion: 0.43
Nodes (6): connect_with_retries(), main(), PgPool, Result, shutdown_signal(), PgConnectOptions

### Community 41 - "Blueprint Light Canvas Design"
Cohesion: 0.29
Nodes (6): deserialize_some(), Error, Option, Result, T, D

### Community 42 - "User Repository Layer"
Cohesion: 0.67
Nodes (3): String, Uuid, User

## Knowledge Gaps
- **126 isolated node(s):** `config`, `App`, `extends`, `./.svelte-kit/tsconfig.json`, `rewriteRelativeImportExtensions` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `nodes.rs` to `Topic Handlers (CRUD)`, `Axum Router Builder + Topic Tests`, `Auth Integration Tests`, `ApiError Type`, `Poke Domain Models + Repository`, `Edge Domain Models`, `Frontend Vitest Example (Welcome.svelte)`?**
  _High betweenness centrality (0.561) - this node is a cross-community bridge._
- **Why does `build_router()` connect `Axum Router Builder + Topic Tests` to `nodes.rs`, `Dashboard Handler`, `Node Handlers and Integration Tests`, `Dashboard Integration Tests`, `Edge Handlers and Integration Tests`, `send`, `Board Handler`, `Board Integration Tests`?**
  _High betweenness centrality (0.269) - this node is a cross-community bridge._
- **Why does `$lib/markdown` connect `Frontend package.json Scripts` to `+page.svelte`?**
  _High betweenness centrality (0.120) - this node is a cross-community bridge._
- **What connects `config`, `App`, `extends` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `nodes.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09086538461538461 - nodes in this community are weakly interconnected._
- **Should `fixtures.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.07013574660633484 - nodes in this community are weakly interconnected._
- **Should `Topic Handlers (CRUD)` be split into smaller, more focused modules?**
  _Cohesion score 0.09098639455782313 - nodes in this community are weakly interconnected._