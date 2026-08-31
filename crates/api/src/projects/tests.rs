//! Drives the real router, so these prove where things are mounted and which
//! gate each route is behind — not just what the handlers return.
//!
//! **No database.** The pool is connected lazily and never used: every case
//! below is one the api answers before it would query — from the catalogue,
//! from the signature, or from a missing token. A route that starts querying
//! fails here loudly rather than passing against a stub, which is the point.

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CACHE_CONTROL},
    response::Response,
};
use serde_json::Value;
use tower::ServiceExt as _;

use crate::{
    config::Config,
    testing::{signed, signed_as, signed_json},
};

fn catalog() -> ohara::catalog::Catalog {
    ohara::catalog::Catalog::load(
        ohara::fixture::content(),
        ohara::Drafts::Hidden,
    )
    .unwrap()
}

fn router() -> axum::Router {
    let db =
        sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
            .unwrap();

    crate::api::app(
        Config::sample(),
        loginwith::providers(vec![]).unwrap(),
        db,
        std::sync::Arc::new(catalog()),
    )
}

async fn send(request: Request<Body>) -> Response {
    router().oneshot(request).await.unwrap()
}

async fn body_of(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    serde_json::from_slice(&bytes).unwrap()
}

async fn json(uri: &str) -> Value {
    body_of(send(signed("GET", uri)).await).await
}

/// A json pointer, so a missing field names itself rather than panicking three
/// lines later as a null comparison.
fn at<'v>(value: &'v Value, path: &str) -> &'v Value {
    value
        .pointer(path)
        .unwrap_or_else(|| panic!("no {path} in {value}"))
}

/// The fixture's ids, read out of the catalogue rather than pasted, so
/// retuning the fixture does not mean editing assertions.
fn ids() -> (String, String) {
    let catalog = catalog();
    let snapshot = catalog.current();
    let project = snapshot.project("fixture-project").unwrap();

    (
        project.project.id.unwrap().to_string(),
        project
            .task("listen-on-port")
            .unwrap()
            .task
            .id
            .unwrap()
            .to_string(),
    )
}

// ------------------------------------------------------------------ the gate

#[tokio::test]
async fn every_route_is_a_404_without_a_signature() {
    // 404 rather than 401: a 401 confirms the endpoint is there, which is free
    // reconnaissance for anything probing the surface.
    for uri in [
        "/api/v1/ping",
        "/api/v1/health",
        "/api/v1/projects",
        "/api/v1/projects/fixture-project",
        "/api/v1/projects/fixture-project/tasks",
        "/api/v1/tasks/listen-on-port",
        "/api/v1/user",
        "/api/v1/tasks/listen-on-port/hints",
    ] {
        let response =
            send(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}

#[tokio::test]
async fn a_signature_over_one_path_does_not_open_another() {
    // The property the old body-signing scheme did not have: a captured
    // signature is good for the request it was made for and nothing else.
    let mut request = signed("GET", "/api/v1/ping");
    *request.uri_mut() = "/api/v1/projects".parse().unwrap();

    assert_eq!(send(request).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ping_answers_pong_and_nothing_may_hold_it() {
    let response = send(signed("GET", "/api/v1/ping")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .contains("no-store")
    );
    assert_eq!(
        at(&body_of(response).await, "/message"),
        &Value::from("pong")
    );
}

// ----------------------------------------------------------- the public five

#[tokio::test]
async fn the_listing_holds_the_published_projects_and_not_the_drafts() {
    let body = json("/api/v1/projects").await;

    assert_eq!(at(&body, "/total"), &Value::from(1));
    assert_eq!(at(&body, "/items/0/slug"), &Value::from("fixture-project"));
    assert_eq!(at(&body, "/items/0/task_count"), &Value::from(2));
    assert_eq!(
        at(&body, "/items/0/unlock_mode"),
        &Value::from("sequential")
    );
}

#[tokio::test]
async fn a_project_carries_its_blueprint_once_and_its_tasks_in_order() {
    let body = json("/api/v1/projects/fixture-project").await;

    assert!(
        at(&body, "/blueprint")
            .as_str()
            .unwrap()
            .contains("phase \"listen\"")
    );
    assert_eq!(at(&body, "/tasks/0/slug"), &Value::from("listen-on-port"));
    assert_eq!(at(&body, "/tasks/1/slug"), &Value::from("say-something"));

    // Once per project, never once per task — the whole point of dropping
    // `tasks.blueprint`, which held a copy of this on every row.
    for task in body.pointer("/tasks").unwrap().as_array().unwrap() {
        assert!(task.get("blueprint").is_none(), "{task}");
    }
}

#[tokio::test]
async fn a_hints_words_are_not_in_a_project_listing() {
    // The laravel project endpoint shipped every hint's text to everybody,
    // which made the unlock endpoint beside it decorative.
    let body = json("/api/v1/projects/fixture-project").await;
    let hint = at(&body, "/tasks/0/hints/0");

    assert!(hint.get("text").is_none(), "{hint}");
    assert!(hint.get("points_deduction").is_some(), "{hint}");
    assert!(!body.to_string().contains("Bind before you listen"));
}

#[tokio::test]
async fn an_anonymous_caller_gets_the_tasks_and_no_progress() {
    // Absent rather than zeroed: "no progress" and "no attempts" are different
    // statements, and a signed-out `lux project show` must not read as the
    // second.
    let body = json("/api/v1/projects/fixture-project/tasks").await;

    assert_eq!(body.as_array().unwrap().len(), 2);
    assert!(at(&body, "/0").get("progress").is_none());
}

#[tokio::test]
async fn a_project_and_a_task_answer_to_a_uuid_as_well_as_a_slug() {
    let (project_id, task_id) = ids();

    let by_id = json(&format!("/api/v1/projects/{project_id}")).await;
    assert_eq!(at(&by_id, "/slug"), &Value::from("fixture-project"));

    let task = json(&format!("/api/v1/tasks/{task_id}")).await;
    assert_eq!(at(&task, "/slug"), &Value::from("listen-on-port"));

    // ...and by slug, which is what a reader types.
    let same = json("/api/v1/tasks/listen-on-port").await;
    assert_eq!(at(&same, "/id"), at(&task, "/id"));
}

#[tokio::test]
async fn a_task_carries_its_prose_where_a_listing_does_not() {
    let alone = json("/api/v1/tasks/listen-on-port").await;
    let listed = json("/api/v1/projects/fixture-project/tasks").await;

    assert!(
        at(&alone, "/description")
            .as_str()
            .unwrap()
            .contains("bind")
    );
    assert!(at(&listed, "/0/description").is_null());
}

#[tokio::test]
async fn nothing_unpublished_is_addressable() {
    // A draft project 404s, and so does a task inside one. The second is what
    // a per-project check alone would miss: a task carries no status, so the
    // project being a draft has to be what keeps it out of the catalogue.
    for uri in [
        "/api/v1/projects/unfinished-project",
        "/api/v1/projects/unfinished-project/tasks",
        "/api/v1/tasks/a-task",
        "/api/v1/projects/no-such-project",
    ] {
        assert_eq!(
            send(signed("GET", uri)).await.status(),
            StatusCode::NOT_FOUND,
            "{uri}"
        );
    }
}

// ---------------------------------------------------- the authenticated five

#[tokio::test]
async fn the_authenticated_routes_refuse_a_caller_with_no_token() {
    // 401 and not 404: past the signature the caller is holding the client
    // secret and already knows the endpoint is there.
    let requests = [
        signed("GET", "/api/v1/user"),
        signed("GET", "/api/v1/tasks/listen-on-port/hints"),
        signed_json(
            "POST",
            "/api/v1/projects/attempts",
            &serde_json::json!({}),
        ),
        signed("POST", "/api/v1/projects/fixture-project/restart"),
    ];

    for request in requests {
        let uri = request.uri().to_string();
        let response = send(request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
        assert_eq!(
            at(&body_of(response).await, "/code"),
            &Value::from("unauthenticated")
        );
    }
}

#[tokio::test]
async fn a_token_that_is_not_even_shaped_like_one_never_reaches_a_query() {
    // Refused by the parser, so nothing touches the lazily-connected pool. A
    // malformed token that made it as far as a lookup would fail this test by
    // trying to connect.
    for token in ["nobar", "abc|secret", "42|"] {
        let response = send(signed_as("GET", "/api/v1/user", token)).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{token}");
    }
}

#[tokio::test]
async fn a_bad_token_on_a_public_route_is_refused_rather_than_ignored() {
    // Passing through would answer 200 with no progress in it, and a reader
    // whose token had expired would read that as their work having been lost.
    let response =
        send(signed_as("GET", "/api/v1/projects", "not-a-token")).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
