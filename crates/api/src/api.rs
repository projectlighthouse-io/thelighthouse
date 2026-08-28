//! What is mounted where, and behind which gates.
//!
//! `main` starts this; the gates themselves are `middleware`. Reading `app`
//! top to bottom should be enough to say what any request can reach — which is
//! why the layering lives here and not spread across the modules being mounted.
//!
//! Content is `books`. The entitlement and payment endpoints from
//! docs/rebuild.md replace the stub routes below.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::RETRY_AFTER},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use loginwith::Providers;
use ohara::catalog::Catalog;
use sqlx::postgres::PgPool;

use crate::{
    auth, bookmarks, books,
    cache::{self, CachePolicy},
    config::Config,
    db,
    limit::RateLimit,
    middleware::{rate::limit_requests, signature::require_signature},
    notes, response, telemetry,
};

/// What every handler can reach. Cheap to clone — `PgPool` and `Providers` are
/// both handles to something shared, and `Config` is a handful of strings.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) socials: Providers,
    pub(crate) db: PgPool,
    /// Shared, not cloned: an `Arc` so every clone of this state counts against
    /// the same buckets. A `RateLimit` per clone would be a limit per request.
    pub(crate) limits: Arc<RateLimit>,
    /// The limit every route inherits, keyed by reader or address. Shared for
    /// the same reason `limits` is: a per-clone counter is a per-request one.
    pub(crate) requests: Arc<RateLimit>,
    /// What a reload costs, capped for the process rather than per caller.
    /// The signature is the gate; this is what holds if the gate ever fails.
    pub(crate) reloads: Arc<RateLimit>,
    /// Shared for the same reason, and more so: reloading swaps the snapshot
    /// inside this one, and a per-clone catalogue would leave most requests
    /// reading a copy nothing ever reloads.
    pub(crate) catalog: Arc<Catalog>,
}

pub(crate) fn app(
    config: Config,
    socials: Providers,
    db: PgPool,
    catalog: Arc<Catalog>,
) -> Router {
    let state = AppState {
        config,
        socials,
        db,
        limits: Arc::new(RateLimit::note_writes()),
        requests: Arc::new(RateLimit::requests()),
        reloads: Arc::new(RateLimit::content_reloads()),
        catalog,
    };

    // luxctl's surface. Everything mounted here inherits the signature check.
    let signed = Router::new()
        .route("/ping", get(ping))
        .route("/me", get(me))
        .route_layer(from_fn_with_state(
            state.config.clone(),
            require_signature,
        ));

    // Signed like luxctl's surface, and unrouted like `/health`. Two
    // independent gates, because either alone is thin: caddy not naming a path
    // is configuration, and configuration is one edit from being wrong.
    let internal = Router::new().route("/reload", post(reload)).route_layer(
        from_fn_with_state(state.config.clone(), require_signature),
    );

    // Everything a caller off the internet can reach, under one limit.
    //
    // `RateLimiter::for('api')` from the laravel app — sixty a minute — and it
    // is the floor, not the whole story: the note writes keep their own
    // stricter budget inside this one, exactly as a laravel route carrying
    // both `throttle:api` and `throttle:notes` does.
    let public = Router::new()
        .nest("/api", signed)
        // Absolute paths, so these merge alongside `signed` rather than nesting
        // under the same prefix. Both carry their own gates — see their
        // `routes`.
        .merge(books::routes(&state))
        .merge(notes::routes(&state))
        .merge(bookmarks::routes(&state))
        // Deliberately outside the reader gate: an OAuth callback is a browser
        // navigation and cannot carry an HMAC or a session. Those routes
        // authenticate themselves — see `auth`.
        .merge(auth::routes(&state))
        .layer(from_fn_with_state(state.clone(), limit_requests));

    Router::new()
        // Outside the limit, both of them. `/health` is polled on a timer by
        // whatever is watching the container, and a probe that starts failing
        // because it probed too often is worse than no probe; `/reload` is
        // signed, unrouted, and called by a deploy rather than by traffic.
        .route("/health", get(health))
        .merge(internal)
        .merge(public)
        .layer(axum::middleware::from_fn(telemetry::trace_request))
        .with_state(state)
}

/// Unsigned on purpose. Caddy never routes to it, so it is reachable only from
/// inside the container, which is where the health probe runs.
///
/// It checks the database, because the probe's job is to answer "should this
/// container keep serving traffic" and a process that cannot reach postgres
/// cannot serve a single page. A health check that only proves the process is
/// running is the kind that stays green through an outage.
async fn health(State(state): State<AppState>) -> Response {
    if db::is_reachable(&state.db).await {
        (StatusCode::OK, "ok").into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "database unreachable")
            .into_response()
    }
}

/// The one bucket every reload counts against, whoever asked for it.
const RELOAD_BUDGET: &str = "content-reload";

/// Rereads the content repo and swaps the catalogue over, in place.
///
/// The whole point is that nothing restarts. [`Catalog::reload`] builds a new
/// snapshot and swaps one `Arc` pointer, so requests already running finish
/// against the snapshot they are holding and the next one picks up the new
/// catalogue. There is no window where the site has no content.
///
/// **A failed reload changes nothing.** The rebuild happens first and the swap
/// only follows if it parsed, so a typo in one lesson leaves the previous
/// catalogue serving and reports what was wrong. That is why this can be safe
/// to call from a deploy script: the bad case is "still serving the old
/// content", not "serving none".
///
/// # Who may call it
///
/// Two gates, and neither is trusted on its own.
///
/// It carries the same HMAC signature luxctl uses, verified here in constant
/// time — so reaching the socket is not enough, and anything else sharing the
/// container's loopback still cannot trigger it. And it is not mounted under
/// `/api/*`, so the Caddyfile never routes it and nothing outside reaches it
/// at all.
///
/// The signature is the boundary; the caddy rule is depth. Relying on caddy
/// alone would make an unauthenticated way to run work on demand out of one
/// mistaken proxy rule, and this repo's own rule is that caddy is not a
/// security boundary — it cannot verify an HMAC, so it must never be the only
/// thing standing in front of something that does work.
async fn reload(State(state): State<AppState>) -> Response {
    // Before the work, not after: the point is to not do it. One bucket for
    // the whole process — see `RateLimit::content_reloads` for why this is not
    // per caller.
    if let Some(retry_after) = state.reloads.check(RELOAD_BUDGET) {
        tracing::warn!(retry_after, "content reload refused: too many");

        let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
        if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
            response.headers_mut().insert(RETRY_AFTER, value);
        }

        return response;
    }

    match state.catalog.reload() {
        Ok(()) => {
            let snapshot = state.catalog.current();
            let books = snapshot.books().count();
            let lessons: usize =
                snapshot.books().map(|book| book.lessons().count()).sum();

            tracing::info!(books, lessons, "content reloaded");

            response::json(
                StatusCode::OK,
                serde_json::json!({ "books": books, "lessons": lessons }),
                CachePolicy::NoStore,
            )
        }
        Err(cause) => {
            // Named, unlike a 500 from a reader-facing route: the only caller
            // is a deploy script inside the container, and "which file" is the
            // entire value of the response.
            tracing::error!(
                %cause,
                "failed to reload content; keeping the previous catalogue"
            );

            response::json(
                StatusCode::UNPROCESSABLE_ENTITY,
                serde_json::json!({
                    "error": cause.to_string(),
                    "serving": "the previous catalogue, unchanged",
                }),
                CachePolicy::NoStore,
            )
        }
    }
}

/// Stands in for the reader's own state — progress, bookmarks.
///
/// `Private`, not `NoStore`: it is scoped to one reader, so nothing shared may
/// hold it, but their own browser revalidating with an `ETag` is both safe and
/// the difference between a snappy dashboard and one that refetches everything.
async fn me(headers: HeaderMap) -> Response {
    revalidating(&headers, CachePolicy::Private, "placeholder")
}

/// Stands in for the luxctl surface: eleven endpoints, all behind the layer.
///
/// Signed, so session-dependent: `NoStore`. A luxctl response is scoped to one
/// reader's progress and must not be held anywhere.
async fn ping(headers: HeaderMap) -> Response {
    revalidating(&headers, CachePolicy::NoStore, "pong")
}

/// A json body that answers 304 when the caller's copy is already current.
///
/// `response::json` with an `ETag` on top. Only worth it where a client repeats
/// the same request often enough for the saved bytes to matter, which is why
/// the note endpoints do not use it.
fn revalidating<T: serde::Serialize>(
    request_headers: &HeaderMap,
    cache_policy: CachePolicy,
    value: T,
) -> Response {
    let Ok(body) = serde_json::to_vec(&value) else {
        return crate::response::server_error();
    };

    let etag = cache::etag_for(&body);

    // 304 regardless of policy: revalidation is about not resending bytes the
    // caller already has, which is orthogonal to whether anyone may store them.
    if let Some(etag) = etag.as_ref()
        && cache::matches_if_none_match(request_headers, etag)
    {
        let mut response = cache::NOT_MODIFIED.into_response();
        cache::apply(response.headers_mut(), cache_policy, Some(etag.clone()));
        return response;
    }

    let mut response = Json(value).into_response();
    cache::apply(response.headers_mut(), cache_policy, etag);

    response
}
