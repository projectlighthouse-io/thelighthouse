//! One function per route. No rendering decisions, no entitlement rules.

use axum::response::Response;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

use super::{
    entitlement::{Access, access},
    view::{BookDetail, BookSummary, LessonView},
};
use ohara::Locale;

use crate::{
    api::AppState,
    cache::CachePolicy,
    middleware,
    response::{self, not_found},
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
    headers: HeaderMap,
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

    // A lesson that withholds nothing is the same answer for everybody, and
    // asking who is reading would only make it uncacheable.
    if !prose.has_paid_part() {
        return response::json(
            StatusCode::OK,
            LessonView::of(book_entry, lesson_entry, &prose, true),
            CachePolicy::public_content(),
        );
    }

    // Optional, because this route serves anonymous readers too — they get the
    // free half and the contents list, which is the whole point of it being
    // one url.
    let reader = middleware::reader::optional(&state, &headers).await;

    let unlocked = match access(
        &state.db,
        &state.billing.plans,
        reader.as_ref().map(|session| session.user_id),
        book_entry,
    )
    .await
    {
        Ok(access) => access == Access::Full,
        Err(cause) => {
            // Not swallowed into "locked": a database that cannot be reached
            // means *we do not know*, and showing a paywall to somebody who
            // paid is the failure that generates a support ticket.
            tracing::error!(%book, %cause, "failed to check entitlement");
            return response::server_error();
        }
    };

    response::json(
        StatusCode::OK,
        LessonView::of(book_entry, lesson_entry, &prose, unlocked),
        if unlocked {
            // Carries the paid prose. A shared cache keys on the url alone, so
            // any positive lifetime here would hand this to the next anonymous
            // reader — see `books::mod`.
            CachePolicy::NoStore
        } else {
            // The same bytes for every unentitled reader, so still shareable,
            // but briefly: a reader who buys should not keep seeing the locked
            // copy for long.
            CachePolicy::withheld_content()
        },
    )
}
