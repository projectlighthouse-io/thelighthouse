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
        ("GET", "/api/billing/access"),
        ("GET", "/api/billing/stripe/bought/cs_test_1"),
        ("POST", "/api/billing/stripe/checkout"),
        ("POST", "/api/billing/stripe/manage"),
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

/// Cloudflare's header, and only in the shape it is documented to take.
///
/// `XX` is its "could not tell" and `T1` is Tor; neither is a place, and
/// passing either on as one would have the frontend price for a country that
/// does not exist.
#[tokio::test]
async fn the_country_comes_from_cloudflare_and_nowhere_else() {
    let asked = |header: Option<&'static str>| async move {
        let mut request = Request::builder().uri("/api/billing/plans");

        if let Some(code) = header {
            request = request.header("CF-IPCountry", code);
        }

        let response = send(request.body(Body::empty()).unwrap()).await;
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let answer: serde_json::Value = serde_json::from_slice(&body).unwrap();

        answer.pointer("/country").cloned().unwrap()
    };

    assert_eq!(asked(Some("BD")).await, "BD");
    // Normalised, so a frontend comparing against an uppercase code matches.
    assert_eq!(asked(Some("gb")).await, "GB");

    for nowhere in [None, Some("XX"), Some("T1"), Some("nonsense"), Some("1")] {
        assert_eq!(
            asked(nowhere).await,
            serde_json::Value::Null,
            "{nowhere:?} is not a country"
        );
    }
}

#[tokio::test]
async fn what_things_cost_is_private_because_it_names_a_country() {
    // The pricing page has to render for somebody who is not signed in, so
    // this is open — but it names the country the caller asked from, which is
    // what stops it being shared.
    let response = send(
        Request::builder()
            .uri("/api/billing/plans")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    // The answer carries the caller's country, so nothing shared may
    // hold it — one country's prices served to the next is the failure.
    assert!(cache_control(&response).contains("no-store"));

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let plans: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // A json pointer, so a missing field names itself rather than panicking
    // somewhere later as a null comparison.
    let at = |path: &str| {
        plans
            .pointer(path)
            .unwrap_or_else(|| panic!("no {path} in {plans}"))
            .clone()
    };

    // `Billing::sample` sells one yearly plan, and the plans moved under a
    // key of their own when the answer grew a country and a coupon.
    assert_eq!(at("/plans/0/plan"), "yearly");
    assert_eq!(at("/plans/0/track"), "yearly");
    assert_eq!(at("/plans/0/recurring"), true);

    // No header, so nowhere in particular — and a coupon slot with nothing in
    // it, because coupons are not built.
    //
    // The coupon hangs off the plan, not the answer: it is restricted to that
    // plan's product at stripe, so "is this price reduced" is a question about
    // a plan. The field is always present, so a page does not have to tell a
    // missing key from a null one.
    assert_eq!(at("/country"), serde_json::Value::Null);
    assert_eq!(at("/plans/0/coupon"), serde_json::Value::Null);
}

#[tokio::test]
async fn a_checkout_lookup_needs_a_reader() {
    // It reads one session out of the provider and says what it was for, so
    // an anonymous caller with a guessed id must get nothing.
    let response = send(
        Request::builder()
            .uri("/api/billing/stripe/bought/cs_test_whatever")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn what_a_reader_may_read_needs_a_reader() {
    let response = send(
        Request::builder()
            .uri("/api/billing/access")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
