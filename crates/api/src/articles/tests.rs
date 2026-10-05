//! Drives the real router, so these prove where things are mounted and which
//! gate each route sits behind — not just what the handlers return.
//!
//! No database. The pool is lazily connected to a name nothing serves, so a
//! request that reaches a handler fails loudly rather than passing against a
//! stub. That is what makes these tests about *the gates*: every case here is
//! one that must be refused before any query runs.

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CACHE_CONTROL},
    response::Response,
};
use tower::ServiceExt as _;

use crate::{
    articles::{
        handler::{Shelf, shelf},
        payload::AuthorFilter,
    },
    config::Config,
    middleware::csrf::CSRF_HEADER,
};

fn router() -> axum::Router {
    let db =
        sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
            .unwrap();

    crate::api::app(
        Config::sample(),
        loginwith::providers(vec![]).unwrap(),
        db,
        std::sync::Arc::new(
            ohara::catalog::Catalog::load(
                ohara::fixture::content(),
                ohara::Drafts::Hidden,
            )
            .unwrap(),
        ),
        crate::api::Billing::sample(),
    )
}

/// No session cookie and no CSRF token — an anonymous browser.
async fn anonymous(method: &str, uri: &str) -> Response {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    router().oneshot(request).await.unwrap()
}

/// Anonymous, but carrying a CSRF header — so a route that answered would be
/// answering on the strength of the header alone.
async fn with_csrf_header(method: &str, uri: &str) -> Response {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header(CSRF_HEADER, "not-a-real-token")
        .body(Body::from("{}"))
        .unwrap();

    router().oneshot(request).await.unwrap()
}

fn cache_control(response: &Response) -> &str {
    response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

/// The reading routes are mounted and open — they get as far as the database,
/// which is not there, rather than being refused at a gate.
///
/// A 401 or a 403 here would mean the blog had quietly become signed-in-only.
#[tokio::test]
async fn the_blog_is_readable_without_signing_in() {
    for uri in ["/api/articles", "/api/articles/some-slug-a3f19c"] {
        let status = anonymous("GET", uri).await.status();

        assert_eq!(
            status,
            StatusCode::INTERNAL_SERVER_ERROR,
            "{uri} did not reach a handler"
        );
    }
}

/// The one property `books` argues for at length: a response that depended on
/// who asked must never say `public`. These do not depend on who asked, and
/// they are the only two article routes that may say it.
#[tokio::test]
async fn a_read_that_fails_still_says_nothing_may_store_it() {
    // A 500 carries `no-store` whatever the route's own policy is — the point
    // here is that the refusal is not cacheable either.
    let response = anonymous("GET", "/api/articles").await;

    assert!(
        cache_control(&response).contains("no-store"),
        "{response:?}"
    );
}

#[tokio::test]
async fn every_write_needs_a_session() {
    for (method, uri) in [
        ("POST", "/api/articles"),
        ("PATCH", "/api/articles/a-post-a3f19c"),
        ("DELETE", "/api/articles/a-post-a3f19c"),
        ("POST", "/api/articles/a-post-a3f19c/archive"),
        ("DELETE", "/api/articles/a-post-a3f19c/archive"),
    ] {
        assert_eq!(
            anonymous(method, uri).await.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} was reachable without a session"
        );

        // And a CSRF header on its own buys nothing: the reader gate is
        // outside it, so this must still be a 401 rather than a 403.
        assert_eq!(
            with_csrf_header(method, uri).await.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} answered a csrf header without a session"
        );
    }
}

#[tokio::test]
async fn the_authors_own_shelf_needs_a_session() {
    assert_eq!(
        anonymous("GET", "/api/articles/mine").await.status(),
        StatusCode::UNAUTHORIZED
    );
}

/// `mine` is a literal segment beside `{slug}`, and it must not be read as a
/// slug — the two are in different routers behind different gates.
///
/// The 401 is the proof: had it matched `{slug}`, the open read route would
/// have taken it and answered 500 from the missing database.
#[tokio::test]
async fn mine_is_a_route_and_not_a_slug() {
    let status = anonymous("GET", "/api/articles/mine").await.status();

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

/// The route exists but is not part of luxctl's signed surface, and a bearer
/// token cannot reach it either — the whole prefix is a browser's.
#[tokio::test]
async fn nothing_here_answers_a_bearer_token() {
    let request = Request::builder()
        .method("POST")
        .uri("/api/articles")
        .header("content-type", "application/json")
        .header("authorization", "Bearer whatever")
        .body(Body::from("{}"))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// The filter is refused before anything is queried, so this is the one read
/// case that answers without a database — which is also the proof that a topic
/// nobody defined is not silently dropped.
#[tokio::test]
async fn a_listing_filtered_by_an_unknown_topic_is_refused_not_ignored() {
    let response = anonymous("GET", "/api/articles?topic=php").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let code = body.pointer("/code").expect("no code in the refusal");

    assert_eq!(code, "article_category_unknown");
}

/// `?author=` is not a fixed list, so it cannot be refused the way a topic is
/// — it reaches the database like any other listing.
#[tokio::test]
async fn a_listing_filtered_by_an_author_reaches_the_handler() {
    for uri in [
        "/api/articles?author=ada",
        "/api/articles?author=nobody-has-this-name",
        "/api/articles?author=ada&topic=rust",
    ] {
        assert_eq!(
            anonymous("GET", uri).await.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{uri} did not reach a handler"
        );
    }
}

/// An empty `?author=` is the same as not asking, exactly as an empty
/// `?topic=` is — the "all authors" option in a `<select>` submits one.
#[test]
fn an_empty_author_filter_is_not_a_filter() {
    let filter = |author: Option<&str>| AuthorFilter {
        author: author.map(str::to_owned),
    };

    assert_eq!(filter(None).author(), None);
    assert_eq!(filter(Some("")).author(), None);
    assert_eq!(filter(Some("   ")).author(), None);
    assert_eq!(filter(Some(" ada ")).author(), Some("ada"));
}

/// The one check that decides whether a taken down article is visible, and the
/// reason it is a function rather than a condition inside a query.
#[test]
fn only_an_author_reading_their_own_shelf_gets_the_private_listing() {
    assert_eq!(shelf(Some(7), Some(7)), Shelf::Own(7));
}

#[test]
fn asking_about_somebody_else_is_the_public_listing() {
    // Signed in as 7, asking about 9. The commonest case that must not leak.
    assert_eq!(shelf(Some(9), Some(7)), Shelf::Public);
}

#[test]
fn asking_about_an_author_without_a_session_is_the_public_listing() {
    assert_eq!(shelf(Some(7), None), Shelf::Public);
}

#[test]
fn a_listing_with_no_author_filter_is_public_however_you_signed_in() {
    assert_eq!(shelf(None, Some(7)), Shelf::Public);
    assert_eq!(shelf(None, None), Shelf::Public);
}

/// A topic that is one of ours gets past the filter and on to the database,
/// which is the same 500 the unfiltered listing gets.
#[tokio::test]
async fn a_listing_filtered_by_a_known_topic_reaches_the_handler() {
    for category in crate::articles::payload::CATEGORIES {
        let uri = format!("/api/articles?topic={category}");

        assert_eq!(
            anonymous("GET", &uri).await.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{category} was refused"
        );
    }
}
