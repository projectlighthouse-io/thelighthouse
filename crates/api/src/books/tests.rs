//! Drives the real router, so these prove where things are mounted and what
//! each response allows a cache to do — not just what the handlers return.

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CACHE_CONTROL},
    response::Response,
};
use serde_json::Value;
use tower::ServiceExt as _;

use crate::config::Config;

fn router() -> axum::Router {
    let config = Config::sample();

    let socials = loginwith::providers(vec![]).unwrap();

    // Lazy: no route here queries. One that starts to will fail loudly rather
    // than pass against a stub.
    let db =
        sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
            .unwrap();

    let catalog = std::sync::Arc::new(
        ohara::catalog::Catalog::load(ohara::fixture::content()).unwrap(),
    );

    crate::api::app(config, socials, db, catalog)
}

async fn get(uri: &str) -> Response {
    router()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn json(uri: &str) -> Value {
    let response = get(uri).await;
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    serde_json::from_slice(&body).unwrap()
}

/// A json pointer, so a missing field names itself rather than panicking
/// somewhere three lines later as a null comparison.
fn at<'v>(value: &'v Value, path: &str) -> &'v Value {
    value
        .pointer(path)
        .unwrap_or_else(|| panic!("no {path} in {value}"))
}

fn cache_control(response: &Response) -> &str {
    response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[tokio::test]
async fn the_listing_is_shared_so_the_edge_may_hold_it() {
    let response = get("/api/books").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        cache_control(&response).contains("s-maxage"),
        "{response:?}"
    );
    assert!(cache_control(&response).contains("public"));
}

#[tokio::test]
async fn a_listing_carries_enough_to_render_a_card() {
    let books = json("/api/books").await;

    assert_eq!(at(&books, "/0/slug"), "fixture-book");
    assert_eq!(at(&books, "/0/price/amount"), 2900);
    assert_eq!(at(&books, "/0/price/currency"), "usd");
    assert_eq!(at(&books, "/0/lesson_count"), 2);
    assert_eq!(at(&books, "/0/first_lesson"), "free-lesson");
    // A listing must not carry every lesson of every book.
    assert!(books.pointer("/0/chapters").is_none());
}

#[tokio::test]
async fn a_book_groups_its_lessons_into_chapters() {
    let book = json("/api/books/fixture-book").await;

    assert_eq!(at(&book, "/chapters/0/id"), 1);
    assert_eq!(at(&book, "/chapters/0/title"), "The Only Chapter");
    // Reading order, which is chapter order then position within it.
    assert_eq!(at(&book, "/chapters/0/lessons/0/slug"), "free-lesson");
    assert_eq!(at(&book, "/chapters/0/lessons/1/slug"), "split-lesson");
    assert!(book.pointer("/chapters/0/lessons/2").is_none());
}

#[tokio::test]
async fn an_unknown_book_is_a_404() {
    assert_eq!(get("/api/books/nope").await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_lesson_is_addressed_by_slug_and_never_by_folder() {
    assert_eq!(
        get("/api/books/fixture-book/lessons/free-lesson")
            .await
            .status(),
        StatusCode::OK
    );

    // The folder is how it is stored, not how it is addressed. Renumbering a
    // book must not be able to break a url.
    assert_eq!(
        get("/api/books/fixture-book/lessons/01-free-lesson")
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_lesson_arrives_rendered_with_its_neighbours() {
    let lesson = json("/api/books/fixture-book/lessons/free-lesson").await;

    assert_eq!(at(&lesson, "/slug"), "free-lesson");
    assert!(at(&lesson, "/html").as_str().unwrap().contains("<p>"));
    assert_eq!(at(&lesson, "/book/slug"), "fixture-book");
    assert!(at(&lesson, "/previous").is_null());
    // A neighbour carries its title, so the "next up" link needs no second
    // request to render.
    assert_eq!(at(&lesson, "/next/slug"), "split-lesson");
    assert_eq!(at(&lesson, "/next/title"), "A Split Lesson");
}

#[tokio::test]
async fn a_lesson_says_where_it_sits_in_the_book() {
    let lesson = json("/api/books/fixture-book/lessons/split-lesson").await;

    assert_eq!(at(&lesson, "/position"), 2);
    assert_eq!(at(&lesson, "/total"), 2);
    assert_eq!(at(&lesson, "/percent"), 100);
    assert!(at(&lesson, "/read_minutes").as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn the_contents_list_covers_the_free_half_only() {
    let lesson = json("/api/books/fixture-book/lessons/split-lesson").await;
    let toc = at(&lesson, "/toc").as_array().unwrap().clone();

    // One heading above the marker, two below it.
    assert_eq!(toc.len(), 1, "{toc:?}");
    assert_eq!(at(&lesson, "/toc/0/text"), "A Free Section");
    assert_eq!(at(&lesson, "/toc/0/id"), "a-free-section");

    // A count, never the titles: "2 more sections" is a reason to buy, and
    // "A Paid Section" is a spoiler. Neither paid heading may appear anywhere
    // in this response.
    assert_eq!(at(&lesson, "/remaining_sections"), 2);

    let whole = serde_json::to_string(&lesson).unwrap();
    assert!(!whole.contains("A Paid Section"), "{whole}");
    assert!(!whole.contains("Another Paid Section"), "{whole}");
}

#[tokio::test]
async fn the_free_half_never_carries_the_paid_half() {
    let response = get("/api/books/fixture-book/lessons/split-lesson").await;

    // Shared, and therefore holdable by a cache — which is exactly why the
    // check below matters.
    assert!(cache_control(&response).contains("s-maxage"));

    let lesson = json("/api/books/fixture-book/lessons/split-lesson").await;
    let html = at(&lesson, "/html").as_str().unwrap().to_owned();

    assert!(html.contains("above the marker"));
    assert!(!html.contains("below the marker"), "{html}");
    assert_eq!(at(&lesson, "/has_paid_part"), true);
}

#[tokio::test]
async fn a_lesson_with_no_paywall_says_nothing_is_withheld() {
    let lesson = json("/api/books/fixture-book/lessons/free-lesson").await;

    assert_eq!(at(&lesson, "/has_paid_part"), false);
}

#[tokio::test]
async fn the_paid_half_needs_a_reader_before_anything_else() {
    // 401 and not 404: the browser already knows this url exists, because the
    // free half told it there was more.
    assert_eq!(
        get("/api/books/fixture-book/lessons/split-lesson/paid")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn seo_falls_back_to_what_a_reader_sees() {
    let lesson = json("/api/books/fixture-book/lessons/free-lesson").await;

    // The fixture lesson has no `meta_title`, so the title stands in rather
    // than the field being null.
    assert_eq!(at(&lesson, "/seo/meta_title"), "A Free Lesson");
    assert_eq!(
        at(&lesson, "/seo/meta_description"),
        "no paywall marker, so it is wholly free"
    );
}
