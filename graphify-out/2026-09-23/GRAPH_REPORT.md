# Graph Report - .  (2026-09-23)

## Corpus Check
- 19 files · ~19,744 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 614 nodes · 1373 edges · 70 communities (40 shown, 30 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 19 edges (avg confidence: 0.83)
- Token cost: 0 input · 0 output

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
- AuthUser Self-Extractor Ref
- Graphify Skill Directive
- Shared Board Filter Store
- Svelte Placeholder Favicon
- robots.txt Crawl Policy
- Loom Project Overview

## God Nodes (most connected - your core abstractions)
1. `ApiError` - 22 edges
2. `send()` - 20 edges
3. `req()` - 19 edges
4. `signup()` - 17 edges
5. `send()` - 16 edges
6. `AuthError` - 16 edges
7. `app()` - 16 edges
8. `create_node()` - 16 edges
9. `AppState` - 15 edges
10. `NodeResponse` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Build plan phase sequencing` --conceptually_related_to--> `SvelteKit adapter-static served by Axum`  [INFERRED]
  CLAUDE.md → frontend/CLAUDE.md
- `SvelteKit adapter-static served by Axum` --references--> `Small atomic commit convention`  [EXTRACTED]
  frontend/CLAUDE.md → CLAUDE.md
- `Naming and language style conventions` --conceptually_related_to--> `Rust/Axum API`  [INFERRED]
  CLAUDE.md → backend/CLAUDE.md
- `Naming and language style conventions` --conceptually_related_to--> `Import ts-rs generated types (no hand-written duplicates)`  [INFERRED]
  CLAUDE.md → frontend/CLAUDE.md
- `Argon2id + JWT + rotated refresh cookie auth` --conceptually_related_to--> `user_id-scoped access control model`  [INFERRED]
  backend/CLAUDE.md → CLAUDE.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Backend-complete-before-frontend build sequencing** — claude_md_build_plan_sequencing, backend_claude_md_axum_api, frontend_claude_md_sveltekit_static_adapter [INFERRED 0.85]
- **ts-rs backend-to-frontend type generation pipeline** — backend_claude_md_ts_rs_type_generation, frontend_claude_md_ts_rs_type_import, readme_stack [EXTRACTED 1.00]
- **Docker Compose app + Postgres deployment topology** — docker_compose_app_service, docker_compose_postgres_service, readme_docker_compose_workflow [EXTRACTED 1.00]

## Communities (70 total, 30 thin omitted)

### Community 0 - "Auth Handlers + AuthUser + ApiError"
Cohesion: 0.07
Nodes (59): ApiError, AuthUser, build_refresh_cookie(), clear_refresh_cookie(), issue_tokens(), login(), logout(), map_create_user_error() (+51 more)

### Community 1 - "Topic Handlers (CRUD)"
Cohesion: 0.16
Nodes (33): create(), delete(), get(), list(), AppState, AuthUser, Json, Path (+25 more)

### Community 2 - "Node Domain Enums and DTOs"
Cohesion: 0.16
Nodes (33): AttachTopicRequest, CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView (+25 more)

### Community 3 - "ApiError Type"
Cohesion: 0.17
Nodes (26): ApiError, Error, attach_topic(), create(), delete(), detach_topic(), get(), list() (+18 more)

### Community 4 - "Node Handlers and Integration Tests"
Cohesion: 0.34
Nodes (25): app(), attach_and_detach_topic_are_idempotent(), attach_and_detach_topic_on_another_users_node_returns_404(), attach_topic_belonging_to_another_user_returns_404(), create_get_update_delete_happy_path(), create_node(), list_filters_by_status_focus_kind(), mismatched_progress_pair_returns_400() (+17 more)

### Community 5 - "Dashboard Handler"
Cohesion: 0.15
Nodes (22): get(), ApiError, AppState, AuthUser, Json, Result, State, DashboardCounts (+14 more)

### Community 6 - "Edge Domain Models"
Cohesion: 0.19
Nodes (21): CreateEdgeRequest, EdgeKind, EdgeListQuery, EdgeResponse, DateTime, Option, Utc, Uuid (+13 more)

### Community 7 - "Edge Handlers and Integration Tests"
Cohesion: 0.36
Nodes (22): app(), blocked_reflects_requires_target_status(), container_progress_reflects_children(), create_edge(), create_edge_ownership_scoping(), create_list_delete_happy_path(), create_node(), duplicate_edge_returns_409() (+14 more)

### Community 8 - "Axum Router Builder + Topic Tests"
Cohesion: 0.21
Nodes (18): build_router(), AppState, Router, app(), create_get_update_delete_happy_path(), duplicate_topic_name_returns_409(), ownership_scoping_returns_404_for_another_users_topic(), req() (+10 more)

### Community 9 - "Auth Integration Tests"
Cohesion: 0.31
Nodes (19): app(), cookie_pair(), json_request(), login_rate_limited_after_repeated_failures(), login_success_returns_tokens(), login_wrong_password_and_unknown_email_both_return_401(), logout_revokes_refresh_token(), me_rejects_missing_garbage_and_expired_bearer() (+11 more)

### Community 10 - "Dashboard Integration Tests"
Cohesion: 0.34
Nodes (19): app(), backdate_created_at(), counts_match_fixtures_exactly(), create_node(), dashboard_is_isolated_per_user(), poke_and_backdate(), req(), Body (+11 more)

### Community 11 - "Poke Handlers and Integration Tests"
Cohesion: 0.41
Nodes (19): app(), create_node(), create_poke_happy_path(), deleting_node_cascades_pokes(), get_node(), last_poked_at_reflects_max_and_null_when_never_poked(), list_nodes_agrees_with_get_node_on_last_poked_at(), list_pokes_ordered_newest_first() (+11 more)

### Community 12 - "Board Handler"
Cohesion: 0.16
Nodes (16): canvas(), ApiError, AppState, AuthUser, Json, Result, State, CanvasResponse (+8 more)

### Community 13 - "Poke Domain Models + Repository"
Cohesion: 0.23
Nodes (15): PokeResponse, DateTime, Utc, Uuid, create_poke(), last_poked_at(), list_pokes(), DateTime (+7 more)

### Community 14 - "Board Integration Tests"
Cohesion: 0.33
Nodes (15): app(), canvas_is_isolated_per_user(), canvas_returns_callers_nodes_and_edges(), create_node(), req(), Body, Option, PgPool (+7 more)

### Community 15 - "Frontend TypeScript Config"
Cohesion: 0.14
Nodes (13): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, rewriteRelativeImportExtensions (+5 more)

### Community 16 - "User Repository Layer"
Cohesion: 0.32
Nodes (11): String, Uuid, User, create_user(), find_by_email(), find_by_id(), Error, Option (+3 more)

### Community 17 - "Frontend Lint/Format Tooling"
Cohesion: 0.15
Nodes (13): eslint-config-prettier, eslint-plugin-svelte, devDependencies, eslint-config-prettier, eslint-plugin-svelte, svelte, @sveltejs/kit, @sveltejs/vite-plugin-svelte (+5 more)

### Community 18 - "Project Conventions (CLAUDE.md rules)"
Cohesion: 0.17
Nodes (12): Rust/Axum API, Handler/repository/domain layering, ts-rs TS derive type generation, Build plan phase sequencing, Naming and language style conventions, Small atomic commit convention, SvelteKit adapter-static served by Axum, Import ts-rs generated types (no hand-written duplicates) (+4 more)

### Community 19 - "JWT Issue/Verify"
Cohesion: 0.29
Nodes (11): Claims, issue_access_token(), issue_then_verify_roundtrips(), DecodingKey, EncodingKey, Error, Result, String (+3 more)

### Community 20 - "Refresh Token Repository"
Cohesion: 0.33
Nodes (11): find_valid_by_hash(), insert(), RefreshTokenRow, revoke(), Error, Option, PgPool, Result (+3 more)

### Community 21 - "Node-Topic Linking Repository"
Cohesion: 0.40
Nodes (10): attach_topic(), AttachOutcome, detach_topic(), DetachOutcome, Error, PgPool, Result, Uuid (+2 more)

### Community 22 - "Frontend package.json Scripts"
Cohesion: 0.18
Nodes (11): scripts, build, check, check:watch, dev, format, lint, prepare (+3 more)

### Community 23 - "Argon2id Password Hashing"
Cohesion: 0.29
Nodes (5): hash_password(), hash_then_verify_roundtrips(), Error, Result, String

### Community 24 - "Opaque Refresh Token Generation"
Cohesion: 0.48
Nodes (5): generate(), generate_is_random(), generate_returns_matching_raw_and_hash(), hash_token(), String

### Community 25 - "App Config Loading"
Cohesion: 0.43
Nodes (5): Config, require_env(), Result, Self, String

### Community 26 - "main.rs Boot Sequence"
Cohesion: 0.43
Nodes (6): connect_with_retries(), main(), PgPool, Result, shutdown_signal(), PgConnectOptions

### Community 27 - "PATCH Double-Option Deserialize Helper"
Cohesion: 0.29
Nodes (6): deserialize_some(), Error, Option, Result, D, T

### Community 28 - "Blueprint Light Canvas Design"
Cohesion: 0.50
Nodes (5): Computed blocked status and part_of progress, Blueprint Light design system, Canvas node kind/status badges and progress bar, @xyflow/svelte canvas view, Per-edge-kind connector styling (requires/part_of/related)

### Community 29 - "Docker Compose Services"
Cohesion: 0.50
Nodes (5): docker-compose app service, postgres pg_isready healthcheck, postgres_data volume, docker-compose postgres service, Docker Compose run workflow

### Community 30 - "Frontend package.json Metadata"
Cohesion: 0.40
Nodes (4): name, private, type, version

### Community 31 - "Auth Security Model"
Cohesion: 0.67
Nodes (3): Argon2id + JWT + rotated refresh cookie auth, user_id-scoped access control model, Security-critical path testing policy

## Knowledge Gaps
- **65 isolated node(s):** `gitignorePath`, `name`, `private`, `version`, `type` (+60 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ApiError` connect `ApiError Type` to `Auth Handlers + AuthUser + ApiError`, `Topic Handlers (CRUD)`, `Node Domain Enums and DTOs`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Why does `AuthError` connect `Auth Handlers + AuthUser + ApiError` to `Node Domain Enums and DTOs`, `ApiError Type`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Why does `build_router()` connect `Axum Router Builder + Topic Tests` to `Node Handlers and Integration Tests`, `Edge Handlers and Integration Tests`, `Auth Integration Tests`, `Dashboard Integration Tests`, `Poke Handlers and Integration Tests`, `Board Integration Tests`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **What connects `gitignorePath`, `name`, `private` to the rest of the system?**
  _65 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Auth Handlers + AuthUser + ApiError` be split into smaller, more focused modules?**
  _Cohesion score 0.07287093942054433 - nodes in this community are weakly interconnected._
- **Should `Dashboard Handler` be split into smaller, more focused modules?**
  _Cohesion score 0.14666666666666667 - nodes in this community are weakly interconnected._
- **Should `Frontend TypeScript Config` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._