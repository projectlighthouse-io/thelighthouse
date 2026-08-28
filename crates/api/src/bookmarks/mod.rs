//! Where a reader left off, one lesson at a time.
//!
//! ```text
//!   GET    /api/bookmarks/{book}/{lesson}
//!   PUT    /api/bookmarks/{book}/{lesson}
//!   DELETE /api/bookmarks/{book}/{lesson}
//! ```
//!
//! ```text
//!   mod.rs      the routes, and this map
//!   handler.rs  one function per route, and no sql
//!   payload.rs  what a reader sends, and whether it is acceptable
//!   store.rs    every query, and the only thing that talks to postgres
//!   bookmark.rs a bookmark, as a row and as json
//!   refusal.rs  why a write was refused, and its wire code
//! ```
//!
//! **A bookmark is not a short note.** `notes` accumulate — a lesson has as
//! many as a reader writes, and each one stays where it was put. There is one
//! bookmark per reader per lesson, and placing a second *moves* the first. Two
//! tables, two lifetimes, and one endpoint that tried to serve both would have
//! to be told which it was every time it was called.
//!
//! Addressed by slug pair rather than by id, unlike `/api/notes/{id}`: a reader
//! never holds a bookmark's id — they are on a lesson page, and the lesson is
//! what they mean. `{book}/{lesson}` is also what the browser already has in
//! its own url.
//!
//! The writes share `notes`' budget rather than getting one of their own. It is
//! the same reader annotating the same page, and a separate bucket would mean
//! ten notes *and* ten bookmarks a minute from a script.
//!
//! **No progress, and no "resume reading" yet.** `lesson_bookmarks` is one
//! passage, not a position in a book; the dashboard's continue-where-you-left-
//! off is `docs/rebuild.md` phase 6 and reads lesson progress, not this.

mod bookmark;
mod handler;
mod payload;
mod refusal;
mod store;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, put},
};

use crate::{
    api::AppState,
    middleware::{
        csrf::require_csrf, reader::require_reader, throttle::throttle,
    },
};

/// Absolute paths, so this merges alongside the signed routes rather than
/// nesting under the same `/api` prefix. Caddy routes `/api/bookmarks*`.
///
/// **Reads and writes are separate routers because they need different gates**,
/// exactly as in `notes`: every route needs a reader, and only the writes need
/// a CSRF token and a place in the write limit.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let reads = Router::new()
        .route("/api/bookmarks/{book}/{lesson}", get(handler::show));

    let writes = Router::new()
        .route("/api/bookmarks/{book}/{lesson}", put(handler::place))
        .route("/api/bookmarks/{book}/{lesson}", delete(handler::remove))
        .route_layer(from_fn_with_state(state.clone(), throttle))
        .route_layer(from_fn(require_csrf));

    reads
        .merge(writes)
        .route_layer(from_fn_with_state(state.clone(), require_reader))
}
