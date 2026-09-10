//! What a reader changes about their own account.
//!
//! ```text
//!   GET    /api/settings/profile       what a reader writes about themselves
//!   PATCH  /api/settings/profile       write it
//!
//!   GET    /api/settings/tokens        the tokens this reader holds
//!   POST   /api/settings/tokens        mint one, shown once
//!   DELETE /api/settings/tokens/{id}   revoke one
//!
//!   GET    /api/settings/newsletter    whether this reader is on the list
//!   PUT    /api/settings/newsletter    join it, or leave it
//! ```
//!
//! ```text
//!   mod.rs      the routes, and this map
//!   handler.rs  one function per route, and no sql
//!   view.rs     what the page sees, and what it sends
//!   refusal.rs  why a write was refused, and its wire code
//!   kit.rs      the one call that leaves this process
//! ```
//!
//! No SQL lives here. The token queries are in `tokens::store` beside the
//! lookup that reads the same table, and the profile queries are in
//! `users::store` beside everything else that reads `users` — one module owns
//! a table, and this one owns the decisions about what a reader may do to it.
//!
//! **Name, email and avatar are not editable.** They arrive from the provider
//! on every sign-in, so a field offering to change them would be offering a
//! change that silently reverts on the next login. `username` is generated
//! once at sign-up and is not on the request type at all.
//!
//! **A token cannot mint a token.** Everything here is behind the session
//! cookie, never the bearer layer. luxctl holds a credential that lives on a
//! reader's disk and travels over the network on every run; if that credential
//! could create more, stealing one would mean permanent access that revoking
//! the stolen one does not end. Minting is something a signed-in browser does,
//! which is also where laravel had it.
//!
//! **The newsletter is one boolean.** The page it belongs to used to offer a
//! checkbox per kind of email against no column, no sender and no endpoint;
//! there is one list, at Kit, and the only thing to say about it is whether a
//! reader is on it. `users.newsletter_enabled` is this side's answer and Kit
//! holds the list — `kit.rs` has why only one direction is sent on.
//!
//! **The secret exists in one response and nowhere else.** It is generated,
//! hashed, the hash is stored, and the plaintext is returned once. Nothing
//! writes it down, so "show me that token again" has no implementation rather
//! than a refused one.

mod handler;
pub(crate) mod kit;
mod refusal;
#[cfg(test)]
mod tests;
mod view;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, patch, post, put},
};

use crate::{
    api::AppState,
    middleware::{
        csrf::require_csrf, reader::require_reader, throttle::throttle,
    },
};

/// Absolute paths, so this merges alongside the other routers rather than
/// nesting under one prefix.
///
/// **Reads and writes take different gates**, the split `notes::routes` makes:
/// every route needs a reader, and only the writes need a CSRF token and a
/// place in the write limit. Minting is a write worth rate limiting on its own
/// account — it is the one endpoint here that creates a credential.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let reads = Router::new()
        .route("/api/settings/profile", get(handler::profile))
        .route("/api/settings/tokens", get(handler::list))
        .route("/api/settings/newsletter", get(handler::newsletter));

    let writes = Router::new()
        .route("/api/settings/profile", patch(handler::edit_profile))
        .route("/api/settings/tokens", post(handler::create))
        .route("/api/settings/tokens/{id}", delete(handler::revoke))
        .route("/api/settings/newsletter", put(handler::set_newsletter))
        .route_layer(from_fn_with_state(state.clone(), throttle))
        .route_layer(from_fn(require_csrf));

    reads
        .merge(writes)
        .route_layer(from_fn_with_state(state.clone(), require_reader))
}
