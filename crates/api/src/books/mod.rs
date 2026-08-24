//! Books and lessons, as a reader reads them.
//!
//! ```text
//!   GET /api/books                                  every published book
//!   GET /api/books/{book}                           one book, and its chapters
//!   GET /api/books/{book}/lessons/{lesson}          the free half
//!   GET /api/books/{book}/lessons/{lesson}/paid     the rest, if entitled
//! ```
//!
//! ```text
//!   mod.rs          the routes, and this map
//!   handler.rs      one function per route
//!   view.rs         what a reader sees, as json
//!   entitlement.rs  who may read the paid half
//! ```
//!
//! **The free half and the paid half are separate urls, deliberately.** One url
//! whose body depended on entitlement would have to `Vary: Cookie`, and every
//! visitor carrying any cookie — analytics, Cloudflare's own — would then get a
//! cache entry of their own. The hit rate would go to nothing, including for
//! anonymous readers. Split, the free half is unconditionally cacheable and the
//! paid half is unconditionally not.
//!
//! That also means no misconfigured cache rule can leak paid prose: it is never
//! in a response that anything is allowed to store.
//!
//! **A url carries a slug, never a folder number.** `07-borrowing` is how the
//! lesson is stored; `borrowing` is how it is addressed. Renumbering a book
//! then breaks no link and no indexed result — `ohara::catalog` is where the
//! two are mapped.

mod entitlement;
mod handler;
#[cfg(test)]
mod tests;
mod view;

use axum::{Router, middleware::from_fn_with_state, routing::get};

use crate::{api::AppState, middleware::reader::require_reader};

/// Absolute paths, so this merges alongside the signed routes rather than
/// nesting under the same `/api` prefix.
///
/// **Only the paid half is gated.** The other three are the same for everyone,
/// so a session would buy nothing and `require_reader` would only make them
/// uncacheable. The paid route needs a reader before entitlement can even be
/// asked about, so it carries the layer on its own.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/api/books", get(handler::list))
        .route("/api/books/{book}", get(handler::show))
        .route("/api/books/{book}/lessons/{lesson}", get(handler::lesson));

    let entitled = Router::new()
        .route(
            "/api/books/{book}/lessons/{lesson}/paid",
            get(handler::paid),
        )
        .route_layer(from_fn_with_state(state.clone(), require_reader));

    public.merge(entitled)
}
