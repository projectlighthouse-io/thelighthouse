//! Drives the real router, so these prove where the routes are mounted rather
//! than what the handlers would return if reached.
//!
//! No database: the pool here never connects, so a test that got past the door
//! would fail loudly. That is the point of these — every one of them asserts
//! the door is shut.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use tower::ServiceExt as _;

use crate::config::Config;

fn router() -> axum::Router {
    let config = Config::sample();
    let socials = loginwith::providers(vec![]).unwrap();
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

    crate::api::app(config, socials, db, catalog)
}

async fn send(method: &str, uri: &str) -> Response {
    router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn every_token_route_needs_a_reader() {
    // Mounted inside `require_reader` and outside the signature layer. If one
    // of these ever drifts out of the session gate, one reader's tokens are
    // listable — or mintable — by anybody who asks.
    for (method, uri) in [
        ("GET", "/api/settings/tokens"),
        ("POST", "/api/settings/tokens"),
        ("DELETE", "/api/settings/tokens/1"),
    ] {
        assert_eq!(
            send(method, uri).await.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri}"
        );
    }
}

#[tokio::test]
async fn a_bearer_token_is_not_a_way_in() {
    // The rule this module exists to keep: a credential that lives on a
    // reader's disk must not be able to mint another one. A bearer header is
    // not a session, so it is refused exactly as no header at all is.
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/tokens")
                .header("authorization", "Bearer 1|whatever")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
