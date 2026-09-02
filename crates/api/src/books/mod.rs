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
//! **The free half and the paid half are separate urls, deliberately.**
//!
//! The lesson response carries no `Vary: Cookie` — only `Accept-Encoding` — so
//! a shared cache keys it on the url alone. One url whose body depended on
//! entitlement would therefore hand a paying reader's prose to the next
//! anonymous visitor for the length of its `s-maxage`. **Lowering that ttl is
//! not a fix**: it bounds how long the leak lasts rather than removing it.
//!
//! That leaves three shapes for a single url, and this is why none of them
//! won:
//!
//! - `Vary: Cookie`. Correct, and every visitor carrying any cookie —
//!   analytics, Cloudflare's own — gets a cache entry of their own. The hit
//!   rate goes to nothing, including for the anonymous readers who are the
//!   whole ranking surface.
//! - Headers chosen per response: cacheable when the body is free, `no-store`
//!   when it is not. Also correct, and one request rather than two. It is what
//!   `docs/rebuild.md` first described. What it costs is that the most cached
//!   route on the site becomes one whose cacheability depends on who called
//!   it, and stays safe only while every path through the handler sets the
//!   header right.
//! - `no-store` always. One request, and every lesson uncacheable for
//!   everybody.
//!
//! Split, the free half is unconditionally cacheable and the paid half
//! unconditionally is not, so paid prose is never in a response anything is
//! allowed to store — structurally, rather than as long as a header is right.
//! The price is one `no-store` request per lesson, paid by entitled readers
//! only.
//!
//! **A url carries a slug, never a folder number.** `07-borrowing` is how the
//! lesson is stored; `borrowing` is how it is addressed. Renumbering a book
//! then breaks no link and no indexed result — `ohara::catalog` is where the
//! two are mapped.

pub(crate) mod entitlement;
mod handler;
#[cfg(test)]
mod tests;
mod view;

use axum::{Router, routing::get};

use crate::api::AppState;

/// Absolute paths, so this merges alongside the signed routes rather than
/// nesting under the same `/api` prefix.
///
/// **No route here is gated.** All four serve anybody: the lesson route reads
/// the session cookie itself when the lesson withholds something, because it
/// has to answer for anonymous readers too — see `handler::lesson`.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/books", get(handler::list))
        .route("/api/books/{book}", get(handler::show))
        .route("/api/books/{book}/lessons/{lesson}", get(handler::lesson))
}
