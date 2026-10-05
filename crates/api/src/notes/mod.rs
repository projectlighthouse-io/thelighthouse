//! A reader's own notes, and the public thread they make on a lesson.
//!
//! ```text
//!   GET    /api/books/{book}/lessons/{lesson}/comments?page=&per_page=
//!   GET    /api/notes?page=&per_page=&q=
//!   POST   /api/notes
//!   PATCH  /api/notes/{id}
//!   DELETE /api/notes/{id}
//! ```
//!
//! ```text
//!   mod.rs     the routes, and this map
//!   handler.rs one function per route, and no sql
//!   payload.rs what a reader sends, and whether it is acceptable
//!   target.rs  which lesson a note belongs to, and may it be a reply
//!   store.rs   every query, and the only thing that talks to postgres
//!   note.rs    a note, as a row and as json
//!   thread.rs  a lesson's public notes, replies hung under their roots
//!   refusal.rs why a write was refused, and its wire code
//! ```
//!
//! Every `/api/notes` route needs a reader; the writes also need a CSRF token
//! and a place in the write limit — see `routes` for which gate is mounted
//! where. Every query those routes make binds that reader's id, so no
//! parameter can name somebody else's notes; the two routes taking an id in
//! the path scope by owner and answer 404, making "not yours" and "no such
//! note" indistinguishable.
//!
//! **The thread is the one route here with no reader.** It reads only what
//! its authors made public — `is_public`, enforced in every query that serves
//! it — and it is the reader of `parent_id` too: replies are hung under their
//! roots, one level deep. `docs/rebuild.md` keeps comment threads — what it
//! retires is the public *activity feed*, which is not this.
//!
//! **Reply notifications do not port.** `CreateNote` notifies the parent's
//! author through `NoteReplied`, and `notifications` was dropped in
//! `20260822210000_drop_laravel_only_tables.sql`.
//!
//! **No drift columns.** `content_hash`, `context_before`, `context_after`,
//! `confidence_score` and `last_verified_at` exist to relocate a note when
//! lesson prose changes. Notes written now have no context to relocate
//! against; the mechanism is being reconsidered — see the plan's slice 4.

mod handler;
mod note;
mod payload;
mod refusal;
mod store;
mod target;
#[cfg(test)]
mod tests;
mod thread;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, patch, post},
};

use crate::{
    api::AppState,
    middleware::{
        csrf::require_csrf, reader::require_reader, throttle::throttle,
    },
};

/// Absolute paths, so this merges alongside the signed routes rather than
/// nesting under the same `/api` prefix. Caddy already routes `/api/notes*`, so
/// the `{id}` routes need no rule of their own.
///
/// **Reads and writes are separate routers because they need different gates.**
/// Every route here needs a reader; only the writes need a CSRF token and a
/// place in the rate limit. Splitting them says that at the mount point instead
/// of leaving a single layer to work it out from the method.
///
/// The write gates are applied *inside* `require_reader` — both read the
/// `Session` it inserts, and both refuse when it is absent.
///
/// **The thread is merged after that layer, so it is outside it.** A
/// `route_layer` covers the routes already on the router it is called on, and
/// none added later — which is the whole of what keeps the thread public. It
/// lives here rather than in `books` because this module owns the table. It
/// still sits inside the request limit every public route inherits, and needs a
/// Caddy rule of its own: `/api/books/*/lessons/*` matches one segment per `*`.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let reads = Router::new().route("/api/notes", get(handler::list));

    let writes = Router::new()
        .route("/api/notes", post(handler::create))
        .route("/api/notes/{id}", patch(handler::update))
        .route("/api/notes/{id}", delete(handler::remove))
        .route_layer(from_fn_with_state(state.clone(), throttle))
        .route_layer(from_fn(require_csrf));

    let thread = Router::new().route(
        "/api/books/{book}/lessons/{lesson}/comments",
        get(handler::thread),
    );

    reads
        .merge(writes)
        .route_layer(from_fn_with_state(state.clone(), require_reader))
        .merge(thread)
}
