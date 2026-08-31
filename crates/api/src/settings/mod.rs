//! What a reader changes about their own account.
//!
//! ```text
//!   GET    /api/settings/tokens        the tokens this reader holds
//!   POST   /api/settings/tokens        mint one, shown once
//!   DELETE /api/settings/tokens/{id}   revoke one
//! ```
//!
//! ```text
//!   mod.rs      the routes, and this map
//!   handler.rs  one function per route, and no sql
//!   view.rs     what the page sees, and what it sends
//!   refusal.rs  why a write was refused, and its wire code
//! ```
//!
//! The queries live in `tokens::store`, beside the lookup that reads the same
//! table — one module owns `personal_access_tokens`, and this one owns the
//! decisions about it.
//!
//! **A token cannot mint a token.** Everything here is behind the session
//! cookie, never the bearer layer. luxctl holds a credential that lives on a
//! reader's disk and travels over the network on every run; if that credential
//! could create more, stealing one would mean permanent access that revoking
//! the stolen one does not end. Minting is something a signed-in browser does,
//! which is also where laravel had it.
//!
//! **The secret exists in one response and nowhere else.** It is generated,
//! hashed, the hash is stored, and the plaintext is returned once. Nothing
//! writes it down, so "show me that token again" has no implementation rather
//! than a refused one.

mod handler;
mod refusal;
#[cfg(test)]
mod tests;
mod view;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post},
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
    let reads = Router::new().route("/api/settings/tokens", get(handler::list));

    let writes = Router::new()
        .route("/api/settings/tokens", post(handler::create))
        .route("/api/settings/tokens/{id}", delete(handler::revoke))
        .route_layer(from_fn_with_state(state.clone(), throttle))
        .route_layer(from_fn(require_csrf));

    reads
        .merge(writes)
        .route_layer(from_fn_with_state(state.clone(), require_reader))
}
