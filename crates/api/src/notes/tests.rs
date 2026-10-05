//! Drives the real router, so these prove where the thread is mounted and that
//! no reader gate sits in front of it — not just what the handler returns.
//!
//! Only the paths that answer before a query are driven: the pool is lazy and
//! never connected, so a lesson that exists would reach postgres and fail
//! here. The thread's shape is tested over plain rows in `thread`.

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CACHE_CONTROL},
    response::Response,
};
use tower::ServiceExt as _;

use crate::config::Config;

fn router() -> axum::Router {
    let db =
        sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
            .unwrap();

    let catalog = std::sync::Arc::new(
        ohara::catalog::Catalog::load(
            ohara::fixture::content(),
            ohara::Drafts::Hidden,
        )
        .unwrap(),
    );

    crate::api::app(
        Config::sample(),
        loginwith::providers(vec![]).unwrap(),
        db,
        catalog,
        crate::api::Billing::sample(),
    )
}

/// No cookie, no signature: a stranger.
async fn get(uri: &str) -> Response {
    router()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn cache_control(response: &Response) -> Option<&str> {
    response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
}

#[tokio::test]
async fn the_thread_answers_a_stranger_rather_than_asking_for_a_reader() {
    let response =
        get("/api/books/no-such-book/lessons/no-such-lesson/comments").await;

    // 401 would be `require_reader`; this is the handler's own 404. The
    // header is what tells it apart from a route that was never mounted,
    // which axum answers 404 with no `Cache-Control` at all.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(cache_control(&response), Some("private, no-store"));
}

#[tokio::test]
async fn an_unmounted_neighbour_is_not_the_thread() {
    // The contrast the test above leans on: if this ever grows a
    // `Cache-Control`, that test stops proving the route is mounted.
    let response =
        get("/api/books/no-such-book/lessons/no-such-lesson/nothing").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(cache_control(&response), None);
}

#[tokio::test]
async fn a_readers_own_notes_still_need_a_reader() {
    // The thread is merged beside `/api/notes`; this is the check that the
    // merge did not lift the gate off its neighbour.
    let response = get("/api/notes").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
