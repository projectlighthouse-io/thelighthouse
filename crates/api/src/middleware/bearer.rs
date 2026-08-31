//! Resolves a luxctl bearer token to a reader, or refuses.
//!
//! The counterpart of `reader`, which does the same job for a browser's
//! session cookie. Two layers rather than one that accepts either: a route
//! knows which of the two callers it is for, and a layer that took both would
//! quietly let a browser reach the CLI surface with no CSRF token in sight.
//!
//! Both live behind `signature`, which has already established that the caller
//! is luxctl. This establishes *which* luxctl.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

use crate::{
    api::AppState,
    cache::CachePolicy,
    response,
    tokens::{Holder, Presented, store},
};

/// Rejects anything without a live token, and puts the holder in extensions.
///
/// **401, not 404.** The signature has already been checked by the time this
/// runs, so the caller holds the client secret and knows perfectly well the
/// endpoint is there. Hiding it buys nothing, and would make an expired token
/// look like a broken CLI.
pub(crate) async fn require_token(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // The header is taken *before* the await, so the borrow ends here.
    // `axum::body::Body` is not `Sync`, which makes a `&Request` held across an
    // await enough to stop this future being `Send` — and the compiler reports
    // that as the layer failing a `Service` bound rather than as a borrow.
    let presented = presented_in(request.headers());

    match resolve(&state, presented).await {
        Resolved::Reader(holder) => {
            request.extensions_mut().insert(holder);
            next.run(request).await
        }
        Resolved::Anonymous => unauthenticated(),
        Resolved::Unknown => response::server_error(),
    }
}

/// Puts the holder in extensions when there is one, and passes through when
/// there is not.
///
/// What the five public endpoints carry. They answer from the catalogue either
/// way; a token only folds this reader's own progress into what comes back,
/// which is what the laravel handlers did with `auth('sanctum')` on a public
/// route.
///
/// **A token that is present and wrong is still refused.** Passing through
/// would answer 200 with no progress in it, and a reader whose token had
/// expired would see every task reset to untouched and conclude their work was
/// gone. Anonymous means no `Authorization` header at all.
pub(crate) async fn optional_token(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let offered = request.headers().contains_key(AUTHORIZATION);
    let presented = presented_in(request.headers());

    match resolve(&state, presented).await {
        Resolved::Reader(holder) => {
            request.extensions_mut().insert(holder);
            next.run(request).await
        }
        Resolved::Anonymous if offered => unauthenticated(),
        Resolved::Anonymous => next.run(request).await,
        Resolved::Unknown => response::server_error(),
    }
}

/// What the `Authorization` header turned out to be.
///
/// Three answers and not two, because the third is not the reader's fault: a
/// database that cannot be reached must not be reported as a bad token, or
/// every luxctl user spends the outage being told to log in again.
#[derive(Debug)]
enum Resolved {
    Reader(Holder),
    /// No header, a malformed one, or one naming nothing live.
    Anonymous,
    /// The lookup itself failed.
    Unknown,
}

/// The token in a request's headers, if it is even shaped like one.
fn presented_in(headers: &HeaderMap) -> Option<Presented> {
    Presented::parse(headers.get(AUTHORIZATION)?.to_str().ok()?)
}

async fn resolve(state: &AppState, presented: Option<Presented>) -> Resolved {
    let Some(presented) = presented else {
        return Resolved::Anonymous;
    };

    match store::holder(&state.db, &presented).await {
        Ok(Some(holder)) => Resolved::Reader(holder),
        Ok(None) => Resolved::Anonymous,
        Err(cause) => {
            tracing::error!(%cause, "failed to resolve a token");
            Resolved::Unknown
        }
    }
}

/// The one refusal, worded for a terminal rather than a browser.
fn unauthenticated() -> Response {
    response::json(
        StatusCode::UNAUTHORIZED,
        serde_json::json!({
            "code": "unauthenticated",
            "error": "Sign in with `lux login` and try again.",
        }),
        CachePolicy::NoStore,
    )
}
