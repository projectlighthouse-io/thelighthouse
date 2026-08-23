//! A reader's own notes.
//!
//! ```text
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
//!   refusal.rs why a write was refused, and its wire code
//! ```
//!
//! Behind `require_session`, which resolves the cookie to a user id, checks the
//! CSRF token on writes and counts them against the rate limit. Every query in
//! `store` binds that id, so no parameter can name somebody else's notes — the
//! two routes taking an id in the path scope by owner and answer 404, making
//! "not yours" and "no such note" indistinguishable from outside.
//!
//! `is_public` and `parent_id` are written but nothing reads them yet. They are
//! what a note *means*, and the write is where that is decided; the lesson
//! thread that renders them is slice 3. `docs/rebuild.md` keeps comment threads
//! — what it retires is the public *activity feed*, which is not this.
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

use axum::{
    Router,
    routing::{get, patch},
};

use crate::api::AppState;

/// Absolute paths, so this merges alongside the signed routes rather than
/// nesting under the same `/api` prefix. The layer is applied where the router
/// is assembled — see `api::app`.
///
/// Caddy already routes `/api/notes*`, so the `{id}` routes need no rule of
/// their own.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/notes", get(handler::list).post(handler::create))
        .route(
            "/api/notes/{id}",
            patch(handler::update).delete(handler::remove),
        )
}
