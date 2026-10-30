use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use loom::app::build_router;
use loom::repo::staleness;
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

async fn create_node(app: &axum::Router, token: &str, body: Value) -> String {
    let (status, body) = send(app, req("POST", "/api/nodes", body, Some(token))).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_owned()
}

async fn get_node(app: &axum::Router, token: &str, node_id: &str) -> Value {
    let (status, body) = send(
        app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body
}

async fn patch_node(app: &axum::Router, token: &str, node_id: &str, body: Value) -> StatusCode {
    send(
        app,
        req("PATCH", &format!("/api/nodes/{node_id}"), body, Some(token)),
    )
    .await
    .0
}

async fn set_settings(app: &axum::Router, token: &str, body: Value) {
    let (status, body) = send(app, req("PATCH", "/api/settings", body, Some(token))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn poke(app: &axum::Router, token: &str, node_id: &str, body: Value) {
    let (status, _) = send(
        app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/pokes"),
            body,
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

/// A poke `days` ago, logged through the API (so the backdate path is the
/// real one).
async fn poke_days_ago(app: &axum::Router, token: &str, node_id: &str, days: i64) {
    let poked_at = chrono::Utc::now() - chrono::Duration::days(days);
    poke(app, token, node_id, json!({"poked_at": poked_at})).await;
}

async fn backdate_created_at(pool: &PgPool, node_id: &str, days: i32) {
    let node_id: uuid::Uuid = node_id.parse().unwrap();
    sqlx::query!(
        "UPDATE nodes SET created_at = now() - make_interval(days => $2) WHERE id = $1",
        node_id,
        days,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn add_edge(app: &axum::Router, token: &str, from: &str, to: &str, kind: &str) {
    let (status, _) = send(
        app,
        req(
            "POST",
            "/api/edges",
            json!({"from_node_id": from, "to_node_id": to, "kind": kind}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

async fn dashboard(app: &axum::Router, token: &str) -> Value {
    send(app, req("GET", "/api/dashboard", Value::Null, Some(token)))
        .await
        .1
}

fn titles(list: &Value) -> Vec<String> {
    let mut titles: Vec<String> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_owned())
        .collect();
    titles.sort();
    titles
}

async fn sweep(pool: &PgPool) {
    staleness::sweep_all(pool).await.unwrap();
}

fn is_stale(node: &Value) -> bool {
    !node["stale_since"].is_null()
}

#[sqlx::test]
async fn nodes_go_stale_past_the_threshold(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "stale@example.com").await;

    let never_poked_old = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "never-poked-old", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &never_poked_old, 15).await;
    let poked_long_ago = create_node(
        &app,
        &token,
        json!({"kind": "study", "title": "poked-long-ago", "status": "active"}),
    )
    .await;
    poke_days_ago(&app, &token, &poked_long_ago, 15).await;
    let recently_poked = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "recently-poked", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &recently_poked, 20).await;
    poke(&app, &token, &recently_poked, json!({})).await;
    let done = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "done", "status": "done"}),
    )
    .await;
    backdate_created_at(&pool, &done, 20).await;
    let archived = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "archived", "status": "active", "archived": true}),
    )
    .await;
    backdate_created_at(&pool, &archived, 20).await;
    let idea = create_node(&app, &token, json!({"kind": "idea", "title": "idea"})).await;
    backdate_created_at(&pool, &idea, 20).await;
    sweep(&pool).await;

    assert_eq!(
        titles(&dashboard(&app, &token).await["stale"]),
        vec!["never-poked-old", "poked-long-ago"]
    );
    // Stamped when the threshold was crossed, not when the sweep ran.
    let stale_since: chrono::DateTime<chrono::Utc> = serde_json::from_value(
        get_node(&app, &token, &never_poked_old).await["stale_since"].clone(),
    )
    .unwrap();
    let expected = chrono::Utc::now() - chrono::Duration::days(1);
    assert!((stale_since - expected).num_minutes().abs() < 5);
}

#[sqlx::test]
async fn kinds_and_status_modes_decide_who_can_go_stale(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "stalemodes@example.com").await;

    let queued_never_poked = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "queued-never-poked", "status": "queued"}),
    )
    .await;
    backdate_created_at(&pool, &queued_never_poked, 20).await;
    let queued_poked = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "queued-poked", "status": "queued"}),
    )
    .await;
    poke_days_ago(&app, &token, &queued_poked, 20).await;
    let paused_poked = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "paused-poked", "status": "paused"}),
    )
    .await;
    poke_days_ago(&app, &token, &paused_poked, 20).await;
    let idea_status = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "idea-status", "status": "idea"}),
    )
    .await;
    backdate_created_at(&pool, &idea_status, 20).await;
    let active_study = create_node(
        &app,
        &token,
        json!({"kind": "study", "title": "active-study", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &active_study, 10).await;

    // Defaults: only active nodes, after 14 days.
    sweep(&pool).await;
    assert!(titles(&dashboard(&app, &token).await["stale"]).is_empty());

    set_settings(
        &app,
        &token,
        json!({"stale_queued": "poked", "stale_paused": "always", "stale_idea_status": "always"}),
    )
    .await;
    assert_eq!(
        titles(&dashboard(&app, &token).await["stale"]),
        vec!["idea-status", "paused-poked", "queued-poked"]
    );

    // Saving settings re-sweeps: a shorter threshold pulls in the study,
    // dropping studies from the kinds lets it go again.
    set_settings(&app, &token, json!({"stale_days": 7})).await;
    assert!(titles(&dashboard(&app, &token).await["stale"]).contains(&"active-study".into()));
    set_settings(&app, &token, json!({"stale_kinds": ["project"]})).await;
    assert!(!titles(&dashboard(&app, &token).await["stale"]).contains(&"active-study".into()));

    // Ideas (the kind) and paths can't be picked as stale kinds.
    for kinds in [json!(["idea"]), json!(["path"])] {
        let (status, _) = send(
            &app,
            req(
                "PATCH",
                "/api/settings",
                json!({"stale_kinds": kinds}),
                Some(&token),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    for days in [-1, 366] {
        let (status, _) = send(
            &app,
            req(
                "PATCH",
                "/api/settings",
                json!({"stale_warn_days": days}),
                Some(&token),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[sqlx::test]
async fn auto_pause_pauses_once_and_ends_the_period_at_the_last_poke(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "autopause@example.com").await;
    set_settings(&app, &token, json!({"stale_auto_pause": true})).await;

    let node = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "drifting", "status": "active"}),
    )
    .await;
    let node_uuid: uuid::Uuid = node.parse().unwrap();
    // The period started 30 days ago; the last work was 20 days ago.
    sqlx::query!(
        "UPDATE active_periods SET started_at = now() - interval '30 days' WHERE node_id = $1",
        node_uuid,
    )
    .execute(&pool)
    .await
    .unwrap();
    poke_days_ago(&app, &token, &node, 20).await;
    sweep(&pool).await;

    let paused = get_node(&app, &token, &node).await;
    assert_eq!(paused["status"], "paused");
    assert!(is_stale(&paused));
    let (_, periods) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node}/periods"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    let ended_at: chrono::DateTime<chrono::Utc> =
        serde_json::from_value(periods[0]["ended_at"].clone()).unwrap();
    let last_poke = chrono::Utc::now() - chrono::Duration::days(20);
    assert!((ended_at - last_poke).num_minutes().abs() < 5);

    // Reactivated by hand: still stale (still untouched), but not re-paused.
    assert_eq!(
        patch_node(&app, &token, &node, json!({"status": "active"})).await,
        StatusCode::OK
    );
    sweep(&pool).await;
    let reactivated = get_node(&app, &token, &node).await;
    assert_eq!(reactivated["status"], "active");
    assert!(is_stale(&reactivated));
}

#[sqlx::test]
async fn auto_pause_can_end_the_period_at_the_stale_moment(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "stalemoment@example.com").await;
    set_settings(
        &app,
        &token,
        json!({"stale_auto_pause": true, "stale_pause_period_end": "stale_moment"}),
    )
    .await;
    let node = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "drifting", "status": "active"}),
    )
    .await;
    let node_uuid: uuid::Uuid = node.parse().unwrap();
    sqlx::query!(
        "UPDATE active_periods SET started_at = now() - interval '30 days' WHERE node_id = $1",
        node_uuid,
    )
    .execute(&pool)
    .await
    .unwrap();
    poke_days_ago(&app, &token, &node, 20).await;
    sweep(&pool).await;

    let (_, periods) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node}/periods"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    let ended_at: chrono::DateTime<chrono::Utc> =
        serde_json::from_value(periods[0]["ended_at"].clone()).unwrap();
    // Last poke (20 days ago) + 14 days.
    let stale_moment = chrono::Utc::now() - chrono::Duration::days(6);
    assert!((ended_at - stale_moment).num_minutes().abs() < 5);
}

#[sqlx::test]
async fn only_a_fresh_poke_unstales_and_resumes_with_poke_sets_active(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "unstale@example.com").await;
    set_settings(
        &app,
        &token,
        json!({"stale_auto_pause": true, "poke_sets_active": true}),
    )
    .await;
    let node = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "drifting", "status": "active"}),
    )
    .await;
    poke_days_ago(&app, &token, &node, 30).await;
    sweep(&pool).await;
    assert_eq!(get_node(&app, &token, &node).await["status"], "paused");

    // Backdated but still past the threshold: nothing changes.
    poke_days_ago(&app, &token, &node, 20).await;
    let still = get_node(&app, &token, &node).await;
    assert!(is_stale(&still));
    assert_eq!(still["status"], "paused");

    // Backdated within the threshold: as good as a live poke.
    poke_days_ago(&app, &token, &node, 1).await;
    let fresh = get_node(&app, &token, &node).await;
    assert!(!is_stale(&fresh));
    assert_eq!(fresh["status"], "active");
}

#[sqlx::test]
async fn unstaling_keeps_the_status_without_poke_sets_active(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "unstalekeep@example.com").await;
    set_settings(&app, &token, json!({"stale_auto_pause": true})).await;
    let node = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "drifting", "status": "active"}),
    )
    .await;
    poke_days_ago(&app, &token, &node, 30).await;
    sweep(&pool).await;

    poke(&app, &token, &node, json!({})).await;
    let fresh = get_node(&app, &token, &node).await;
    assert!(!is_stale(&fresh));
    assert_eq!(fresh["status"], "paused");
}

#[sqlx::test]
async fn done_and_archived_clear_staleness(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "clear@example.com").await;
    let finished = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "finished", "status": "active"}),
    )
    .await;
    let hidden = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "hidden", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &finished, 20).await;
    backdate_created_at(&pool, &hidden, 20).await;
    sweep(&pool).await;
    assert!(is_stale(&get_node(&app, &token, &finished).await));

    patch_node(&app, &token, &finished, json!({"status": "done"})).await;
    patch_node(&app, &token, &hidden, json!({"archived": true})).await;
    assert!(!is_stale(&get_node(&app, &token, &finished).await));
    assert!(!is_stale(&get_node(&app, &token, &hidden).await));
}

#[sqlx::test]
async fn dashboard_warns_before_nodes_go_stale(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "warn@example.com").await;
    let soon = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "soon", "status": "active"}),
    )
    .await;
    poke_days_ago(&app, &token, &soon, 12).await;
    let later = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "later", "status": "active"}),
    )
    .await;
    poke_days_ago(&app, &token, &later, 5).await;

    assert_eq!(
        titles(&dashboard(&app, &token).await["going_stale"]),
        vec!["soon"]
    );
    set_settings(&app, &token, json!({"stale_warn_days": 0})).await;
    assert!(titles(&dashboard(&app, &token).await["going_stale"]).is_empty());
}

#[sqlx::test]
async fn paths_inherit_staleness_per_mode(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "stalepath@example.com").await;

    let path = create_node(&app, &token, json!({"kind": "path", "title": "path"})).await;
    let drifting = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "drifting", "status": "active"}),
    )
    .await;
    let busy = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "busy", "status": "active"}),
    )
    .await;
    let finished = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "finished", "status": "done"}),
    )
    .await;
    for node in [&drifting, &busy, &finished] {
        add_edge(&app, &token, node, &path, "part_of").await;
    }
    poke_days_ago(&app, &token, &drifting, 20).await;
    poke_days_ago(&app, &token, &busy, 20).await;
    poke_days_ago(&app, &token, &finished, 20).await;

    // `any` (default): the path's last touch is 20 days old.
    sweep(&pool).await;
    assert!(is_stale(&get_node(&app, &token, &path).await));

    // `all`: every node in play is stale (the done one is ignored)...
    set_settings(&app, &token, json!({"path_stale_mode": "all"})).await;
    assert!(is_stale(&get_node(&app, &token, &path).await));
    // ...until one of them is touched.
    poke(&app, &token, &busy, json!({})).await;
    assert!(!is_stale(&get_node(&app, &token, &path).await));

    set_settings(&app, &token, json!({"path_stale_mode": "any"})).await;
    assert!(!is_stale(&get_node(&app, &token, &path).await));
    set_settings(&app, &token, json!({"path_stale_mode": "off"})).await;
    poke_days_ago(&app, &token, &busy, 30).await;
    sweep(&pool).await;
    assert!(!is_stale(&get_node(&app, &token, &path).await));
}

#[sqlx::test]
async fn a_sweep_never_touches_another_account(pool: PgPool) {
    let app = app(pool.clone());
    let owner = signup(&app, "sweepowner@example.com").await;
    let other = signup(&app, "sweepother@example.com").await;
    let theirs = create_node(
        &app,
        &other,
        json!({"kind": "project", "title": "theirs", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &theirs, 10).await;

    // The owner's settings save sweeps the owner's account only, under the
    // owner's 7-day threshold.
    set_settings(
        &app,
        &owner,
        json!({"stale_days": 7, "stale_auto_pause": true}),
    )
    .await;
    let node = get_node(&app, &other, &theirs).await;
    assert!(!is_stale(&node));
    assert_eq!(node["status"], "active");
}

#[sqlx::test]
async fn path_poke_history_rolls_up_the_nodes_inside(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "history@example.com").await;
    let intruder = signup(&app, "historyintruder@example.com").await;

    let outer = create_node(&app, &token, json!({"kind": "path", "title": "outer"})).await;
    let inner = create_node(&app, &token, json!({"kind": "path", "title": "inner"})).await;
    add_edge(&app, &token, &inner, &outer, "part_of").await;
    let direct = create_node(&app, &token, json!({"kind": "project", "title": "direct"})).await;
    add_edge(&app, &token, &direct, &outer, "part_of").await;
    let nested = create_node(&app, &token, json!({"kind": "study", "title": "nested"})).await;
    add_edge(&app, &token, &nested, &inner, "part_of").await;
    let outside = create_node(&app, &token, json!({"kind": "project", "title": "outside"})).await;
    for node in [&direct, &nested, &outside] {
        poke(&app, &token, node, json!({})).await;
    }

    let (status, history) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{outer}/pokes"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let mut node_ids: Vec<&str> = history
        .as_array()
        .unwrap()
        .iter()
        .map(|poke| poke["node_id"].as_str().unwrap())
        .collect();
    node_ids.sort();
    let mut expected = vec![direct.as_str(), nested.as_str()];
    expected.sort();
    assert_eq!(node_ids, expected);

    let (status, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{outer}/pokes"),
            Value::Null,
            Some(&intruder),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
