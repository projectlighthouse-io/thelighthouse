//! The HTTP surface: routes, the signature boundary, and response shaping.
//!
//! Everything that decides what a request is allowed to do, or what a caller is
//! allowed to keep, lives here. `main` only starts it. That split is the point —
//! the signature layer and the cache policy are the two things worth being able
//! to read end to end without boot code in between.
//!
//! What is real here is the shape: it binds loopback only, verifies the luxctl
//! signature in a layer rather than per handler, and answers the health probe.
//! The content, entitlement and payment endpoints from docs/rebuild.md replace
//! the stub routes below.

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use hmac::{Hmac, Mac};
use loginwith::Providers;
use sha2::Sha256;
use sqlx::postgres::PgPool;
use subtle::ConstantTimeEq;

use crate::{
    auth,
    cache::{self, CachePolicy},
    config::Config,
    db, telemetry,
};

type HmacSha256 = Hmac<Sha256>;

/// A signed body larger than this is refused outright. The whole body has to be
/// buffered to compute the HMAC over it, so without a ceiling an unauthenticated
/// caller can make the process allocate as much as it likes.
const MAX_SIGNED_BODY: usize = 1024 * 1024;

/// What every handler can reach. Cheap to clone — `PgPool` and `Providers` are
/// both handles to something shared, and `Config` is a handful of strings.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) socials: Providers,
    pub(crate) db: PgPool,
}

pub(crate) fn app(config: Config, socials: Providers, db: PgPool) -> Router {
    // Everything mounted here inherits the signature check.
    let signed = Router::new()
        .route("/ping", get(ping))
        .route("/books", get(books))
        .route("/me", get(me))
        .route_layer(middleware::from_fn_with_state(
            config.clone(),
            require_signature,
        ));

    Router::new()
        .route("/health", get(health))
        .nest("/api", signed)
        // Merged rather than nested under the same `/api`, and deliberately
        // outside the signature layer: an OAuth callback is a browser
        // navigation and cannot carry an HMAC. Those four routes authenticate
        // themselves — see `auth`.
        .merge(auth::routes())
        .layer(middleware::from_fn(telemetry::trace_request))
        .with_state(AppState {
            config,
            socials,
            db,
        })
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn signature_matches(secret: &str, body: &[u8], provided: &str) -> bool {
    let Some(provided) = hex_decode(provided) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);

    // Constant time. A byte-by-byte compare leaks the correct prefix through
    // timing, which is enough to forge a signature given enough attempts.
    mac.finalize().into_bytes().ct_eq(&provided).into()
}

/// Rejects anything without a valid luxctl signature.
///
/// This is a layer, not a call inside each handler, and that is the point: a
/// new endpoint under `/api` is protected because of where it is mounted, not
/// because someone remembered. Forgetting is no longer possible — you would
/// have to remove the layer to expose something.
///
/// Caddy checks only that the header exists. It cannot verify an HMAC, so this
/// is the real boundary and must never trust the proxy's judgement.
async fn require_signature(State(config): State<Config>, request: Request, next: Next) -> Response {
    // 404 everywhere below, never 401 or 403: those confirm the endpoint is
    // there, which is free reconnaissance for anyone probing.
    let deny = || StatusCode::NOT_FOUND.into_response();

    let provided = request
        .headers()
        .get("x-luxctl-signature")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_default();

    // The body has to be buffered to sign over it, then handed onward intact.
    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_SIGNED_BODY).await else {
        return deny();
    };

    if !signature_matches(&config.luxctl_secret, &bytes, &provided) {
        return deny();
    }

    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
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
        (StatusCode::SERVICE_UNAVAILABLE, "database unreachable").into_response()
    }
}

/// Stands in for the content the frontend reads over loopback.
///
/// Shared, because a book listing is the same for everyone — this is the only
/// kind of response a CDN may hold. Anything that consults a session or an
/// entitlement must not use this policy.
async fn books(headers: HeaderMap) -> Response {
    json_response(&headers, CachePolicy::public_content(), ["placeholder"])
}

/// Stands in for the reader's own state — progress, bookmarks.
///
/// `Private`, not `NoStore`: it is scoped to one reader, so nothing shared may
/// hold it, but their own browser revalidating with an `ETag` is both safe and
/// the difference between a snappy dashboard and one that refetches everything.
async fn me(headers: HeaderMap) -> Response {
    json_response(&headers, CachePolicy::Private, "placeholder")
}

/// Stands in for the luxctl surface: eleven endpoints, all behind the layer.
///
/// Signed, so session-dependent: `NoStore`. A luxctl response is scoped to one
/// reader's progress and must not be held anywhere.
async fn ping(headers: HeaderMap) -> Response {
    json_response(&headers, CachePolicy::NoStore, "pong")
}

/// Serialises, sets the cache headers, and answers 304 when the caller's copy
/// is already current.
///
/// Every JSON response goes through here so the policy is always stated. A
/// handler cannot accidentally return a bare body with no `Cache-Control`.
fn json_response<T: serde::Serialize>(
    request_headers: &HeaderMap,
    policy: CachePolicy,
    value: T,
) -> Response {
    let Ok(body) = serde_json::to_vec(&value) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let etag = cache::etag_for(&body);

    // 304 regardless of policy: revalidation is about not resending bytes the
    // caller already has, which is orthogonal to whether anyone may store them.
    if let Some(etag) = etag.as_ref()
        && cache::matches_if_none_match(request_headers, etag)
    {
        let mut response = cache::NOT_MODIFIED.into_response();
        cache::apply(response.headers_mut(), policy, Some(etag.clone()));
        return response;
    }

    let mut response = Json(value).into_response();
    cache::apply(response.headers_mut(), policy, etag);
    response
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;

    #[test]
    fn rejects_a_wrong_signature() {
        assert!(!signature_matches("secret", b"", "00"));
        assert!(!signature_matches("secret", b"", "not-hex"));
        assert!(!signature_matches("secret", b"", ""));
    }

    #[test]
    fn accepts_a_correct_signature() {
        let mut mac = HmacSha256::new_from_slice(b"secret").unwrap();
        mac.update(b"payload");
        let sig = mac.finalize().into_bytes();
        let hex = sig.iter().fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        });

        assert!(signature_matches("secret", b"payload", &hex));
        // same signature, different body
        assert!(!signature_matches("secret", b"tampered", &hex));
        // right body, wrong secret
        assert!(!signature_matches("other", b"payload", &hex));
    }
}
