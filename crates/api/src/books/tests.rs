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
        ohara::catalog::Catalog::load(
            ohara::fixture::content(),
            ohara::Drafts::Hidden,
        )
        .unwrap(),
    );

    crate::api::app(config, socials, db, catalog, crate::api::Billing::sample())
}

async fn post(uri: &str) -> Response {
    router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
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
    // Which tracks it is on, and where it falls in each. On the summary
    // because the filtering and the ordering both happen on the listing.
    assert_eq!(at(&books, "/0/tracks/go"), 2);
    assert_eq!(at(&books, "/0/tracks/rust"), 1);
    // A listing must not carry every lesson of every book.
    assert!(books.pointer("/0/chapters").is_none());
}

#[tokio::test]
async fn a_track_narrows_the_shelf_to_the_books_on_it() {
    let on_go = json("/api/books?track=go").await;
    assert_eq!(at(&on_go, "/0/slug"), "fixture-book");
    assert!(on_go.as_array().is_some_and(|books| books.len() == 1));

    // A track no book is on is an empty shelf, not the whole one and not a
    // 404: the question has an answer and the answer is nothing.
    let on_nothing = json("/api/books?track=no-such-track").await;
    assert_eq!(on_nothing.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn an_empty_track_is_a_filter_nobody_filled_in() {
    // A stray `&track=` in a url should not blank the page.
    let all = json("/api/books?track=").await;

    assert_eq!(at(&all, "/0/slug"), "fixture-book");
    assert!(all.as_array().is_some_and(|books| !books.is_empty()));
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
async fn the_contents_list_covers_the_whole_lesson() {
    // Reverses an earlier rule that the list stopped at the paywall. A reader
    // deciding whether to buy is better served by the section titles than by
    // their absence; the prose behind them is still withheld.
    let lesson = json("/api/books/fixture-book/lessons/split-lesson").await;
    let toc = at(&lesson, "/toc").as_array().unwrap().clone();

    assert_eq!(toc.len(), 3, "{toc:?}");
    assert_eq!(at(&lesson, "/toc/0/text"), "A Free Section");
    assert_eq!(at(&lesson, "/toc/1/text"), "A Paid Section");
    assert_eq!(at(&lesson, "/toc/2/text"), "Another Paid Section");

    // Which of them this reader can actually reach. Anonymous, so only the
    // first — and the page needs the flag to avoid linking at an anchor that
    // is not in the document it was given.
    assert_eq!(at(&lesson, "/toc/0/locked"), false);
    assert_eq!(at(&lesson, "/toc/1/locked"), true);
    assert_eq!(at(&lesson, "/toc/2/locked"), true);

    assert_eq!(at(&lesson, "/remaining_sections"), 2);
    assert_eq!(at(&lesson, "/unlocked"), false);
}

#[tokio::test]
async fn the_free_half_never_carries_the_paid_half() {
    let response = get("/api/books/fixture-book/lessons/split-lesson").await;

    // Shared, and therefore holdable by a cache — which is exactly why the
    // check below matters. An anonymous reader gets the free half only, and
    // that answer is the one a cache is allowed to keep.
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
async fn a_withheld_lesson_is_cached_briefly_rather_than_for_long() {
    // Still the same bytes for every unentitled reader, so still shareable —
    // but a reader who buys should not keep being served the locked copy, so
    // the window is much shorter than a lesson that withholds nothing.
    let withheld = get("/api/books/fixture-book/lessons/split-lesson").await;
    let open = get("/api/books/fixture-book/lessons/free-lesson").await;

    assert!(
        cache_control(&withheld).contains("s-maxage=60"),
        "{:?}",
        cache_control(&withheld)
    );
    assert!(
        cache_control(&open).contains("s-maxage=300"),
        "{:?}",
        cache_control(&open)
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

/// A signed POST to `/reload`, built fresh each time so one router can serve
/// several — the budget lives in the state, not the request.
fn signed_reload() -> Request<Body> {
    crate::testing::signed("POST", "/reload")
}

async fn post_signed(uri: &str) -> Response {
    router()
        .oneshot(crate::testing::signed("POST", uri))
        .await
        .unwrap()
}

#[tokio::test]
async fn reloading_needs_a_signature_and_says_nothing_without_one() {
    // 404, not 401: a 401 confirms the endpoint is there, which is free
    // reconnaissance. Caddy never routes this path either — the signature is
    // the boundary, the missing route is depth.
    assert_eq!(post("/reload").await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reloading_swaps_the_catalogue_without_restarting() {
    let response = post_signed("/reload").await;

    assert_eq!(response.status(), StatusCode::OK);
    // Nothing may hold this: it is a write, and its body is a count that is
    // only true at the instant it was produced.
    assert!(
        cache_control(&response).contains("no-store"),
        "{response:?}"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let counted: Value = serde_json::from_slice(&body).unwrap();

    // The fixture, reread from disk rather than remembered.
    assert_eq!(at(&counted, "/books"), 1);
    assert_eq!(at(&counted, "/lessons"), 2);
}

#[tokio::test]
async fn reloading_runs_out_of_budget_before_it_runs_out_of_cpu() {
    // Three a minute for the whole process. The signature is the gate; this is
    // what holds if the gate ever fails, so it must not be per caller — the
    // fourth call is refused however it arrived.
    let app = router();

    for attempt in 1..=3 {
        let response = app.clone().oneshot(signed_reload()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "reload {attempt}");
    }

    let refused = app.oneshot(signed_reload()).await.unwrap();

    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(refused.headers().contains_key("retry-after"));
}

#[tokio::test]
async fn reloading_is_a_post_because_it_changes_the_process() {
    // A GET would be fetched by anything that crawls, prefetches or
    // revalidates, and this one does work. 404 rather than 405: the signature
    // layer answers before the method does, and says as little as possible.
    assert_eq!(get("/reload").await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_contents_list_marks_which_lessons_withhold_something() {
    let book = json("/api/books/fixture-book").await;

    // The fixture's second lesson has a paid region; the first has none.
    assert_eq!(at(&book, "/chapters/0/lessons/0/has_paid_part"), false);
    assert_eq!(at(&book, "/chapters/0/lessons/1/has_paid_part"), true);

    // It says *that* something is withheld, never what. No paid heading may
    // reach a listing any more than it may reach a lesson response.
    let whole = serde_json::to_string(&book).unwrap();
    assert!(!whole.contains("A Paid Section"), "{whole}");
    assert!(!whole.contains("below the marker"), "{whole}");
}
