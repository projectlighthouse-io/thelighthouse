//! What is mounted where, and behind which gates.
//!
//! `main` starts this; the gates themselves are `middleware`. Reading `app`
//! top to bottom should be enough to say what any request can reach — which is
//! why the layering lives here and not spread across the modules being mounted.
//!
//! The content, entitlement and payment endpoints from docs/rebuild.md replace
//! the stub routes below.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
};
use loginwith::Providers;
use sqlx::postgres::PgPool;

use crate::{
    auth,
    cache::{self, CachePolicy},
    config::Config,
    db,
    limit::RateLimit,
    middleware::signature::require_signature,
    notes, telemetry,
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
}

pub(crate) fn app(config: Config, socials: Providers, db: PgPool) -> Router {
    let state = AppState {
        config,
        socials,
        db,
        limits: Arc::new(RateLimit::note_writes()),
    };

    // luxctl's surface. Everything mounted here inherits the signature check.
    let signed = Router::new()
        .route("/ping", get(ping))
        .route("/books", get(books))
        .route("/me", get(me))
        .route_layer(from_fn_with_state(
            state.config.clone(),
            require_signature,
        ));

    Router::new()
        .route("/health", get(health))
        .nest("/api", signed)
        // Absolute paths, so these merge alongside `signed` rather than nesting
        // under the same prefix. Notes carries its own gates — see its `routes`.
        .merge(notes::routes(&state))
        // Deliberately outside every gate: an OAuth callback is a browser
        // navigation and cannot carry an HMAC or a session. Those four routes
        // authenticate themselves — see `auth`.
        .merge(auth::routes())
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

/// Stands in for the content the frontend reads over loopback.
///
/// Shared, because a book listing is the same for everyone — this is the only
/// kind of response a CDN may hold. Anything that consults a session or an
/// entitlement must not use this policy.
async fn books(headers: HeaderMap) -> Response {
    revalidating(&headers, CachePolicy::public_content(), ["placeholder"])
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
