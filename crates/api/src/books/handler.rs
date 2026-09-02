//! One function per route. No rendering decisions, no entitlement rules.

use axum::response::Response;
use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use super::{
    entitlement::{Access, access},
    view::{BookDetail, BookSummary, LessonView, PaidView},
};
use ohara::{Locale, body};

use crate::{
    api::AppState,
    cache::CachePolicy,
    response::{self, not_found},
    session::Session,
};

/// What narrows [`list`]. Absent means the whole shelf.
#[derive(Debug, Deserialize)]
pub(crate) struct Shelf {
    /// A track name — `go`, `rust`, `systems`. Not an enum: the tracks are
    /// content, named in `book.yaml`, and an enum here would mean a rust change
    /// every time somebody starts one. An unknown name is simply a track no
    /// book is on.
    track: Option<String>,
}

/// Every published book, or one track of them.
///
/// **A track is a reading order, so asking for one orders the answer by it.**
/// Go Fundamentals before Go Intermediate, whatever order the catalogue walks
/// the shelf in. Without a track the catalogue's own order stands, which is the
/// only order a mixed shelf has.
///
/// A track nobody is on answers `[]` rather than 404 or the whole shelf: the
/// question "which books are on this track" has an empty answer, and neither
/// pretending the track exists nor pretending the filter was not asked for is
/// an improvement on saying so.
///
/// `Shared`: the same bytes for a subscriber, a stranger and a crawler, which
/// is what lets the edge hold it. The query string is part of the url, so two
/// tracks are two cache entries rather than one that keeps being overwritten.
pub(crate) async fn list(
    State(state): State<AppState>,
    Query(shelf): Query<Shelf>,
) -> Response {
    let snapshot = state.catalog.current();
    let mut entries: Vec<_> = snapshot.books().collect();

    // Empty is absent: `?track=` is a filter nobody filled in, and answering
    // nothing to it would be a blank page for a stray `&track=` in a url.
    if let Some(track) = shelf.track.as_deref().filter(|name| !name.is_empty())
    {
        entries.retain(|entry| entry.book.tracks.contains_key(track));
        entries.sort_by_key(|entry| {
            entry.book.tracks.get(track).copied().unwrap_or(i32::MAX)
        });
    }

    let books: Vec<BookSummary<'_>> =
        entries.into_iter().map(BookSummary::of).collect();

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

    let prose = match state.catalog.body(&book, &lesson, Locale::default()) {
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
    // Behind `require_reader`, which put this here — see `books::routes`.
    Extension(session): Extension<Session>,
    Path((book, lesson)): Path<(String, String)>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(book_entry) = snapshot.book(&book) else {
        return not_found();
    };

    match access(&state.db, Some(session.user_id), book_entry).await {
        Ok(Access::Full) => {}
        Ok(Access::FreeOnly) => return not_found(),
        Err(cause) => {
            tracing::error!(%book, %cause, "failed to check entitlement");
            return response::server_error();
        }
    }

    let prose = match state.catalog.body(&book, &lesson, Locale::default()) {
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
            toc: body::headings(&paid),
        },
        CachePolicy::NoStore,
    )
}
