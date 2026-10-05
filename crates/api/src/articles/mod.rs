//! The blog: markdown a reader wrote.
//!
//! ```text
//!   GET    /api/articles?page=&per_page=&q=&topic=&author=  every live article
//!   GET    /api/articles/{slug}                         one, if it is live
//!
//!   GET    /api/articles/mine     this author's, taken down ones too
//!   POST   /api/articles          publish one
//!   PATCH  /api/articles/{slug}   rewrite one of your own
//!   DELETE /api/articles/{slug}   delete one of your own
//! ```
//!
//! ```text
//!   mod.rs      the routes, and this map
//!   handler.rs  one function per route, and no sql
//!   payload.rs  what an author sends, and whether it is acceptable
//!   store.rs    every query, and the only thing that talks to postgres
//!   article.rs  an article, as a row and as json
//!   refusal.rs  why a write was refused, and its wire code
//! ```
//!
//! **An article has a title, its topics and a body, and that is all.** Each
//! topic is one of `payload::CATEGORIES` — a hardcoded list, refused if it is
//! not on it — and `?topic=rust` narrows the listing by one of them. An
//! article carries between one and `payload::MAX_TOPICS` of them, because a
//! piece about running go in a container is about both and should not have to
//! pick; they live in `article_topics`.
//!
//! **Publishing is not approval.** An article is live the moment it is
//! written, and there is no pending state, no queue and no screen to sit in
//! front of.
//!
//! **Taking one down is an `UPDATE`, not an endpoint.** `taken_down_at` and
//! `taken_down_reason` are set by hand, in psql:
//!
//! ```sql
//! UPDATE articles
//!    SET taken_down_at = now(),
//!        taken_down_reason = 'Reposted without attribution.'
//!  WHERE slug = '…';
//! ```
//!
//! …and cleared by setting both back to null. There is no route for it
//! because there is one person who does it, rarely, and an endpoint would mean
//! a permission model, a gate and a screen for something one statement already
//! does. Every read path filters on those columns — see `store` — so the row
//! stops being served within the cache window either way.
//!
//! **A takedown carries a reason, and the author is the only one who reads
//! it.** Everyone else gets the 404 a missing article gets, so a takedown
//! cannot be confirmed from outside. The author sees theirs on
//! `/api/articles/mine` and on `/api/articles?author=<their username>`, which
//! are the only two routes that answer about an article that is not live.
//!
//! **`?author=` is the one read that looks at the session.** It takes a
//! username, and when it names the reader who is asking, the listing answers
//! in the author's own shape — taken down articles included, takedown reasons
//! and all. `handler::shelf` is that decision, in one place and testable
//! without a database; the query cannot make it, because `store::own` binds
//! the id it is given rather than one off the query string.
//!
//! **So the cache policy is chosen per answer, not per route.** The public
//! listing is still the same bytes for everybody and the edge may still hold
//! it — the split `books` argues for at length. The author's own view is
//! `no-store`, which is what makes the branch safe without `Vary: Cookie`: a
//! shared cache is never handed the answer that depended on who asked. What a
//! *frontend* must not do is render that view into a page it then lets the
//! edge keep — see `web/app/pages/blog/index.vue`, which fetches it in the
//! browser for exactly this reason.
//!
//! **`body` is markdown as typed, and is never rendered here.** It goes out on
//! the wire as the author wrote it and Nuxt renders it, which is where every
//! other markdown on the site is rendered — see `docs/rebuild.md` on route
//! prefixes. That makes the renderer's output a frontend concern, and it makes
//! **sanitising the result the frontend's job**: this is reader-supplied
//! markdown, so raw html in it must not reach a page unescaped. Storing
//! rendered html here instead would put reader-supplied markup in the database
//! and make every renderer fix a backfill.
//!
//! **Writes share the note write budget** rather than getting one of their
//! own. It is the same reader typing into the same site, and a second knob
//! with no separate reason to be turned is a second knob to get wrong.

mod article;
mod handler;
#[cfg(test)]
mod ownership_tests;
mod payload;
mod refusal;
mod store;
#[cfg(test)]
mod tests;

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

/// Absolute paths, so this merges alongside the other routers rather than
/// nesting under one prefix.
///
/// Three routers because there are three different gates: the anonymous reads
/// take none, the author's own listing takes a reader, and the writes take a
/// reader and a CSRF token and a place in the write limit — the split
/// `notes::routes` makes.
///
/// **`require_csrf` is on the writes only, never on a `GET`.** It has no
/// method check of its own — it refuses anything without the header — so
/// mounting it over a read would make that read impossible rather than safe.
///
/// **Every article is addressed by its slug, on the writes too.** Not because
/// an id would leak anything — it is the same row either way — but because
/// axum will not take `{id}` and `{slug}` at the same position across merged
/// routers, and picking one is better than the alternative it leaves: one url
/// shape for reading and another for writing, on the same resource.
///
/// `/api/articles/mine` and `/api/articles/{slug}` are different routers and
/// the same shape of path. Axum matches the literal segment first, so `mine`
/// is never read as a slug — and no article can be minted with that slug
/// anyway, see `payload::slugify`.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/api/articles", get(handler::list))
        .route("/api/articles/{slug}", get(handler::show));

    let own = Router::new()
        .route("/api/articles/mine", get(handler::mine))
        .route_layer(from_fn_with_state(state.clone(), require_reader));

    let writes = Router::new()
        .route("/api/articles", post(handler::create))
        .route("/api/articles/{slug}", patch(handler::update))
        .route("/api/articles/{slug}", delete(handler::remove))
        .route("/api/articles/{slug}/archive", post(handler::archive))
        .route("/api/articles/{slug}/archive", delete(handler::unarchive))
        .route_layer(from_fn_with_state(state.clone(), throttle))
        .route_layer(from_fn(require_csrf))
        .route_layer(from_fn_with_state(state.clone(), require_reader));

    public.merge(own).merge(writes)
}
