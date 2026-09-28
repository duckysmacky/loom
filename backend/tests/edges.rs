use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use loom::app::build_router;
use loom::state::AppState;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test-only-secret";

fn app(pool: PgPool) -> axum::Router {
    build_router(AppState::new(pool, TEST_JWT_SECRET))
}

async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

const TEST_PEER: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    12345,
);

/// Every request needs `ConnectInfo` attached, same as
/// `into_make_service_with_connect_info` would in production - the
/// `/api/auth/*` rate limiter's `SmartIpKeyExtractor` falls back to it
/// when there's no forwarded-for header, which `oneshot` never provides.
fn req(method: &str, uri: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .extension(ConnectInfo(TEST_PEER));
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn signup(app: &axum::Router, email: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/auth/signup",
            json!({"email": email, "password": "password123"}),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["access_token"].as_str().unwrap().to_owned()
}

async fn create_node(app: &axum::Router, token: &str, title: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "project", "title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

async fn create_path(app: &axum::Router, token: &str, title: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "path", "title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

async fn create_edge(
    app: &axum::Router,
    token: &str,
    from: &str,
    to: &str,
    kind: &str,
) -> (StatusCode, Value) {
    send(
        app,
        req(
            "POST",
            "/api/edges",
            json!({"from_node_id": from, "to_node_id": to, "kind": kind}),
            Some(token),
        ),
    )
    .await
}

async fn get_node(app: &axum::Router, token: &str, id: &str) -> Value {
    send(
        app,
        req("GET", &format!("/api/nodes/{id}"), Value::Null, Some(token)),
    )
    .await
    .1
}

async fn set_status(app: &axum::Router, token: &str, id: &str, status: &str) {
    let (status_code, _) = send(
        app,
        req(
            "PATCH",
            &format!("/api/nodes/{id}"),
            json!({"status": status}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status_code, StatusCode::OK);
}

#[sqlx::test]
async fn create_list_delete_happy_path(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "edgecrud@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    let (status, edge) = create_edge(&app, &token, &a, &b, "related").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(edge["from_node_id"], a);
    assert_eq!(edge["to_node_id"], b);
    assert_eq!(edge["kind"], "related");
    let edge_id = edge["id"].as_str().unwrap().to_owned();

    let (_, all) = send(&app, req("GET", "/api/edges", Value::Null, Some(&token))).await;
    assert_eq!(all.as_array().unwrap().len(), 1);

    let (_, by_from) = send(
        &app,
        req(
            "GET",
            &format!("/api/edges?from_node_id={a}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(by_from.as_array().unwrap().len(), 1);

    let (_, by_kind_mismatch) = send(
        &app,
        req("GET", "/api/edges?kind=requires", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(by_kind_mismatch.as_array().unwrap().len(), 0);

    let (delete_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (delete_again, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(delete_again, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn create_edge_ownership_scoping(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "edgeowner@example.com").await;
    let token_b = signup(&app, "edgeintruder@example.com").await;

    let node1 = create_node(&app, &token_a, "node1").await;
    let node2 = create_node(&app, &token_b, "node2").await;

    let (status, _) = create_edge(&app, &token_a, &node1, &node2, "related").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = create_edge(&app, &token_a, &node2, &node1, "related").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn requires_cycle_rejected_two_node(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "cycle2@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    let (status, _) = create_edge(&app, &token, &a, &b, "requires").await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = create_edge(&app, &token, &b, &a, "requires").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn requires_cycle_rejected_three_node(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "cycle3@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;
    let c = create_node(&app, &token, "C").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "requires").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        create_edge(&app, &token, &b, &c, "requires").await.0,
        StatusCode::CREATED
    );

    let (status, _) = create_edge(&app, &token, &c, &a, "requires").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn part_of_cycle_rejected_two_node(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "partofcycle@example.com").await;
    let a = create_path(&app, &token, "A").await;
    let b = create_path(&app, &token, "B").await;

    let (status, _) = create_edge(&app, &token, &a, &b, "part_of").await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = create_edge(&app, &token, &b, &a, "part_of").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn duplicate_edge_returns_409(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "dupedge@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "related").await.0,
        StatusCode::CREATED
    );
    let (status, _) = create_edge(&app, &token, &a, &b, "related").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn self_loop_returns_400(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "selfloop@example.com").await;
    let a = create_node(&app, &token, "A").await;

    let (status, _) = create_edge(&app, &token, &a, &a, "related").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn blocked_reflects_requires_target_status(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "blocked@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "requires").await.0,
        StatusCode::CREATED
    );

    let node_a = get_node(&app, &token, &a).await;
    assert_eq!(node_a["blocked"], true);

    // PATCH's derived_state follow-up recomputes A even though A itself
    // wasn't the node being patched.
    set_status(&app, &token, &b, "done").await;

    let node_a = get_node(&app, &token, &a).await;
    assert_eq!(node_a["blocked"], false);
}

#[sqlx::test]
async fn precedes_cycle_rejected(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "precedescycle@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;
    let c = create_node(&app, &token, "C").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "precedes").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        create_edge(&app, &token, &b, &c, "precedes").await.0,
        StatusCode::CREATED
    );

    let (status, _) = create_edge(&app, &token, &c, &a, "precedes").await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = create_edge(&app, &token, &b, &a, "precedes").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn precedes_does_not_block(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "precedesblock@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "precedes").await.0,
        StatusCode::CREATED
    );

    assert_eq!(get_node(&app, &token, &a).await["blocked"], false);
    assert_eq!(get_node(&app, &token, &b).await["blocked"], false);
}

#[sqlx::test]
async fn requiring_an_idea_blocks(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "requiresidea@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let (status, idea) = send(
        &app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "idea", "title": "idea"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let idea = idea["id"].as_str().unwrap().to_owned();

    assert_eq!(
        create_edge(&app, &token, &a, &idea, "requires").await.0,
        StatusCode::CREATED
    );
    // An idea has no status, so it's never done: it blocks.
    assert_eq!(get_node(&app, &token, &a).await["blocked"], true);
}

#[sqlx::test]
async fn container_progress_reflects_children(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "container@example.com").await;
    let p = create_path(&app, &token, "P").await;
    let x = create_node(&app, &token, "X").await;
    let y = create_node(&app, &token, "Y").await;

    let no_children = get_node(&app, &token, &p).await;
    assert_eq!(no_children["container_progress"], Value::Null);

    assert_eq!(
        create_edge(&app, &token, &x, &p, "part_of").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        create_edge(&app, &token, &y, &p, "part_of").await.0,
        StatusCode::CREATED
    );

    let container = get_node(&app, &token, &p).await;
    assert_eq!(
        container["container_progress"],
        json!({"done": 0, "total": 2})
    );

    set_status(&app, &token, &x, "done").await;

    let container = get_node(&app, &token, &p).await;
    assert_eq!(
        container["container_progress"],
        json!({"done": 1, "total": 2})
    );
}

async fn create_node_with(app: &axum::Router, token: &str, body: Value) -> String {
    let (status, body) = send(app, req("POST", "/api/nodes", body, Some(token))).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_owned()
}

async fn add_checklist_item(app: &axum::Router, token: &str, node_id: &str, done: bool) {
    let (status, item) = send(
        app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/checklist"),
            json!({"title": "item"}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    if done {
        let item_id = item["id"].as_str().unwrap();
        let (status, _) = send(
            app,
            req(
                "PATCH",
                &format!("/api/checklist/{item_id}"),
                json!({"done": true}),
                Some(token),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
}

#[sqlx::test]
async fn path_progress_sums_inner_counters_recursively(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pathprogress@example.com").await;
    let outer = create_path(&app, &token, "P").await;
    let inner = create_path(&app, &token, "Q").await;
    let empty = create_path(&app, &token, "E").await;
    let study = create_node_with(
        &app,
        &token,
        json!({"kind": "study", "title": "S", "progress_current": 12, "progress_total": 15}),
    )
    .await;
    let project = create_node(&app, &token, "Checklist").await;
    for done in [true, true, true, false, false] {
        add_checklist_item(&app, &token, &project, done).await;
    }
    let counterless = create_node(&app, &token, "Counterless").await;
    let idea = create_node_with(&app, &token, json!({"kind": "idea", "title": "I"})).await;

    for (child, parent) in [
        (&study, &outer),
        (&inner, &outer),
        (&idea, &outer),
        (&project, &inner),
        (&counterless, &inner),
    ] {
        assert_eq!(
            create_edge(&app, &token, child, parent, "part_of").await.0,
            StatusCode::CREATED
        );
    }

    assert_eq!(
        get_node(&app, &token, &outer).await["path_progress"],
        json!({"done": 15, "total": 20})
    );
    assert_eq!(
        get_node(&app, &token, &inner).await["path_progress"],
        json!({"done": 3, "total": 5})
    );
    assert_eq!(
        get_node(&app, &token, &empty).await["path_progress"],
        Value::Null
    );
}

#[sqlx::test]
async fn container_progress_skips_ideas(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "containerideas@example.com").await;
    let path = create_path(&app, &token, "P").await;
    let done = create_node(&app, &token, "Done").await;
    let idea = create_node_with(&app, &token, json!({"kind": "idea", "title": "I"})).await;
    for child in [&done, &idea] {
        assert_eq!(
            create_edge(&app, &token, child, &path, "part_of").await.0,
            StatusCode::CREATED
        );
    }
    set_status(&app, &token, &done, "done").await;

    assert_eq!(
        get_node(&app, &token, &path).await["container_progress"],
        json!({"done": 1, "total": 1})
    );
}

#[sqlx::test]
async fn path_status_follows_what_is_inside(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pathstatus@example.com").await;

    // (statuses of the nodes inside, expected path status)
    let cases: [(&[&str], &str); 8] = [
        (&[], "queued"),
        (&["queued"], "queued"),
        (&["idea"], "queued"),
        (&["active", "done"], "active"),
        (&["paused", "done", "queued"], "paused"),
        (&["queued", "done"], "queued"),
        (&["done", "archived"], "done"),
        (&["archived", "archived"], "archived"),
    ];
    for (inside, expected) in cases {
        let path = create_path(&app, &token, "P").await;
        for status in inside {
            let child = create_node_with(
                &app,
                &token,
                json!({"kind": "project", "title": "c", "status": status}),
            )
            .await;
            create_edge(&app, &token, &child, &path, "part_of").await;
        }
        assert_eq!(
            get_node(&app, &token, &path).await["status"],
            *expected,
            "{inside:?}"
        );
    }

    // Ideas are skipped; nested paths are walked through.
    let outer = create_path(&app, &token, "Outer").await;
    let inner = create_path(&app, &token, "Inner").await;
    let done = create_node_with(
        &app,
        &token,
        json!({"kind": "project", "title": "d", "status": "done"}),
    )
    .await;
    let idea = create_node_with(&app, &token, json!({"kind": "idea", "title": "i"})).await;
    create_edge(&app, &token, &inner, &outer, "part_of").await;
    create_edge(&app, &token, &done, &outer, "part_of").await;
    create_edge(&app, &token, &idea, &outer, "part_of").await;
    assert_eq!(get_node(&app, &token, &outer).await["status"], "done");
    let active = create_node_with(
        &app,
        &token,
        json!({"kind": "study", "title": "a", "status": "active"}),
    )
    .await;
    create_edge(&app, &token, &active, &inner, "part_of").await;
    assert_eq!(get_node(&app, &token, &outer).await["status"], "active");

    // A done nested path counts as a done child.
    set_status(&app, &token, &active, "done").await;
    assert_eq!(
        get_node(&app, &token, &outer).await["container_progress"],
        json!({"done": 2, "total": 2})
    );
}

#[sqlx::test]
async fn requiring_a_path_blocks_until_everything_inside_is_done(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "requirespath@example.com").await;
    let dependent = create_node(&app, &token, "D").await;
    let path = create_path(&app, &token, "P").await;
    let step = create_node(&app, &token, "S").await;
    create_edge(&app, &token, &step, &path, "part_of").await;
    create_edge(&app, &token, &dependent, &path, "requires").await;

    assert_eq!(get_node(&app, &token, &dependent).await["blocked"], true);
    set_status(&app, &token, &step, "done").await;
    assert_eq!(get_node(&app, &token, &dependent).await["blocked"], false);
}

#[sqlx::test]
async fn status_filters_use_derived_path_status(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pathfilter@example.com").await;
    let path = create_path(&app, &token, "P").await;
    let step = create_node(&app, &token, "S").await;
    create_edge(&app, &token, &step, &path, "part_of").await;
    set_status(&app, &token, &step, "archived").await;

    let (_, archived) = send(
        &app,
        req("GET", "/api/nodes?view=archived", Value::Null, Some(&token)),
    )
    .await;
    assert!(archived.as_array().unwrap().iter().any(|n| n["id"] == path));
    let (_, queued) = send(
        &app,
        req("GET", "/api/nodes?status=queued", Value::Null, Some(&token)),
    )
    .await;
    assert!(!queued.as_array().unwrap().iter().any(|n| n["id"] == path));
}

#[sqlx::test]
async fn list_nodes_agrees_with_get_node_on_derived_fields(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "consistency@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;
    assert_eq!(
        create_edge(&app, &token, &a, &b, "requires").await.0,
        StatusCode::CREATED
    );

    let via_get = get_node(&app, &token, &a).await;
    let (_, list) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    let via_list = list
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == a)
        .unwrap();

    assert_eq!(via_get["blocked"], via_list["blocked"]);
    assert_eq!(
        via_get["container_progress"],
        via_list["container_progress"]
    );
}

#[sqlx::test]
async fn missing_or_garbage_token_returns_401(pool: PgPool) {
    let app = app(pool);

    let (missing_status, _) = send(&app, req("GET", "/api/edges", Value::Null, None)).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

    let (garbage_status, _) = send(
        &app,
        req("GET", "/api/edges", Value::Null, Some("not-a-real-token")),
    )
    .await;
    assert_eq!(garbage_status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn delete_edge_ownership_scoping(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "deleteedgeowner@example.com").await;
    let token_b = signup(&app, "deleteedgeintruder@example.com").await;
    let a = create_node(&app, &token_a, "A").await;
    let b = create_node(&app, &token_a, "B").await;

    let (_, edge) = create_edge(&app, &token_a, &a, &b, "related").await;
    let edge_id = edge["id"].as_str().unwrap();

    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Still there - user A can delete it themselves.
    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token_a),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn only_paths_can_contain_nodes(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "onlypaths@example.com").await;
    let child = create_node(&app, &token, "child").await;
    let not_a_path = create_node(&app, &token, "idea").await;

    let (status, body) = create_edge(&app, &token, &child, &not_a_path, "part_of").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "only paths can contain nodes");
}

#[sqlx::test]
async fn a_node_sits_in_one_path_but_paths_nest(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "onepath@example.com").await;
    let child = create_node(&app, &token, "child").await;
    let first = create_path(&app, &token, "first").await;
    let second = create_path(&app, &token, "second").await;
    let outer = create_path(&app, &token, "outer").await;

    assert_eq!(
        create_edge(&app, &token, &child, &first, "part_of").await.0,
        StatusCode::CREATED
    );
    let (status, body) = create_edge(&app, &token, &child, &second, "part_of").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "already inside a path");

    // A path can itself sit inside another path.
    assert_eq!(
        create_edge(&app, &token, &first, &outer, "part_of").await.0,
        StatusCode::CREATED
    );
}

#[sqlx::test]
async fn joining_or_leaving_a_path_resets_the_canvas_position(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "relative@example.com").await;
    let child = create_node(&app, &token, "child").await;
    let path = create_path(&app, &token, "path").await;
    let place = |id: &str| {
        req(
            "PATCH",
            &format!("/api/nodes/{id}"),
            json!({"canvas_x": 40, "canvas_y": 60}),
            Some(&token),
        )
    };

    send(&app, place(&child)).await;
    let (_, edge) = create_edge(&app, &token, &child, &path, "part_of").await;
    assert_eq!(
        get_node(&app, &token, &child).await["canvas_x"],
        Value::Null
    );

    send(&app, place(&child)).await;
    let edge_id = edge["id"].as_str().unwrap();
    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        get_node(&app, &token, &child).await["canvas_x"],
        Value::Null
    );
}

#[sqlx::test]
async fn a_path_that_changes_kind_releases_its_nodes(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "release@example.com").await;
    let child = create_node(&app, &token, "child").await;
    let path = create_path(&app, &token, "path").await;
    create_edge(&app, &token, &child, &path, "part_of").await;

    let (status, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{path}"),
            json!({"kind": "project"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["container_progress"], Value::Null);

    // The child is free to join another path again.
    let other = create_path(&app, &token, "other").await;
    assert_eq!(
        create_edge(&app, &token, &child, &other, "part_of").await.0,
        StatusCode::CREATED
    );
}

#[sqlx::test]
async fn canvas_size_is_a_positive_pair(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "size@example.com").await;
    let path = create_path(&app, &token, "path").await;
    let uri = format!("/api/nodes/{path}");

    let (status, body) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"canvas_width": 420.5, "canvas_height": 300}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["canvas_width"], 420.5);

    let (status, _) = send(
        &app,
        req("PATCH", &uri, json!({"canvas_width": null}), Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"canvas_width": -1, "canvas_height": 10}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
