//! Drives the real router, so these prove where things are mounted and which
//! gates they sit behind — not just what the handlers return.
//!
//! Every route here that reads the database answers before it would query: a
//! request with no session is refused by the gate, and the webhook is refused
//! by its signature. The pool is lazy and never connected, so a route that
//! starts querying earlier fails loudly here rather than passing against a
//! stub.

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

async fn send(request: Request<Body>) -> Response {
    router().oneshot(request).await.unwrap()
}

async fn post(uri: &str, body: &'static str) -> Response {
    send(
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap(),
    )
    .await
}

fn cache_control(response: &Response) -> &str {
    response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[tokio::test]
async fn every_reader_route_needs_a_reader() {
    let uris = [
        ("GET", "/api/billing/membership"),
        ("POST", "/api/billing/stripe/checkout"),
        ("POST", "/api/billing/stripe/cancel"),
        ("POST", "/api/billing/stripe/resume"),
        ("POST", "/api/billing/stripe/swap"),
    ];

    for (method, uri) in uris {
        let response = send(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await;

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} is reachable without a session"
        );
    }
}

#[tokio::test]
async fn the_webhook_is_outside_the_reader_gate() {
    // A provider has no session and no CSRF token. If this ever answers 401
    // the endpoint has been put behind a gate no provider can pass, and every
    // delivery is being refused.
    let response = post("/webhooks/stripe", "{}").await;

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn an_unsigned_delivery_is_refused() {
    let response = post("/webhooks/stripe", r#"{"type":"ping"}"#).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_delivery_for_a_provider_nobody_registered_is_a_404() {
    // Not a 400: an unknown provider is a url that does not exist.
    let response = post("/webhooks/paypal", "{}").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn nothing_about_money_may_be_cached() {
    // `private` alone still permits a browser cache, and a shared machine is
    // enough for that to matter.
    let refused = post("/webhooks/stripe", "{}").await;
    assert!(cache_control(&refused).contains("no-store"));

    let unauthorised = send(
        Request::builder()
            .uri("/api/billing/membership")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert!(!cache_control(&unauthorised).contains("public"));
}
