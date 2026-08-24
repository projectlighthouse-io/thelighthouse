//! One function per route. No rendering decisions, no entitlement rules.

use axum::response::Response;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use super::{
    entitlement::may_read_paid,
    view::{BookDetail, BookSummary, LessonView, PaidView},
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    ohara::body,
    response::{self, not_found},
};

/// Every published book.
///
/// `Shared`: the same bytes for a subscriber, a stranger and a crawler, which
/// is what lets the edge hold it.
pub(crate) async fn list(State(state): State<AppState>) -> Response {
    let snapshot = state.catalog.current();
    let books: Vec<BookSummary<'_>> =
        snapshot.books().map(BookSummary::of).collect();

    response::json(StatusCode::OK, books, CachePolicy::public_content())
}

/// One book, with its chapters and their lessons.
pub(crate) async fn show(
    State(state): State<AppState>,
    Path(book): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(entry) = snapshot.book(&book) else {
        return not_found();
    };

    response::json(
        StatusCode::OK,
        BookDetail::of(entry),
        CachePolicy::public_content(),
    )
}

/// A lesson's free half, rendered.
///
/// Never the paid half, whoever is asking — that is what makes this response
/// safe to cache at the edge, and it is why the paid half has a url of its own
/// rather than a branch inside this one.
pub(crate) async fn lesson(
    State(state): State<AppState>,
    Path((book, lesson)): Path<(String, String)>,
) -> Response {
    let snapshot = state.catalog.current();

    let (Some(book_entry), Some(lesson_entry)) =
        (snapshot.book(&book), snapshot.lesson(&book, &lesson))
    else {
        return not_found();
    };

    let prose = match state.catalog.body(&book, &lesson) {
        Ok(Some(prose)) => prose,
        Ok(None) => return not_found(),
        Err(cause) => {
            tracing::error!(%book, %lesson, %cause, "failed to read lesson");
            return response::server_error();
        }
    };

    response::json(
        StatusCode::OK,
        LessonView::of(book_entry, lesson_entry, &prose),
        CachePolicy::public_content(),
    )
}

/// A lesson's paid half, for a reader entitled to it.
///
/// `NoStore`, and behind `require_reader`. 404 rather than 403 when a reader is
/// not entitled: whether the lesson has a paid half at all is not something an
/// unentitled reader needs confirmed, and the free half already said whether
/// there is more to buy.
pub(crate) async fn paid(
    State(state): State<AppState>,
    Path((book, lesson)): Path<(String, String)>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(book_entry) = snapshot.book(&book) else {
        return not_found();
    };

    if !may_read_paid(book_entry) {
        return not_found();
    }

    let prose = match state.catalog.body(&book, &lesson) {
        Ok(Some(prose)) => prose,
        Ok(None) => return not_found(),
        Err(cause) => {
            tracing::error!(%book, %lesson, %cause, "failed to read lesson");
            return response::server_error();
        }
    };

    let Some(paid) = prose.paid.filter(|half| !half.is_empty()) else {
        return not_found();
    };

    response::json(
        StatusCode::OK,
        PaidView {
            html: body::render(&paid),
        },
        CachePolicy::NoStore,
    )
}
