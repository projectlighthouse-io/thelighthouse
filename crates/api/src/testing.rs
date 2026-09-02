//! Building the requests a route test has to send, so each test says what it
//! is about rather than how to get past the door.
//!
//! Every signed route answers 404 without a valid `X-Luxctl-Signature`, and
//! the signature covers the method and the path — so it cannot be a constant,
//! and a test that builds one by hand is a test that silently stops reaching
//! its handler the day the scheme changes. That happened once already.
//!
//! `cfg(test)`-only, and only within this crate: the api's tests are the only
//! caller, and nothing here should be reachable from a release binary.

use axum::{body::Body, http::Request};

use crate::middleware::signature::sign;

/// `Config::sample`'s luxctl secret. The one place a test may know it.
pub(crate) const SECRET: &str = "luxctl";

/// A request signed the way luxctl signs, timestamped now.
///
/// Now rather than a fixed instant, because the middleware refuses anything
/// more than five minutes out of step — a frozen timestamp would pass today
/// and fail forever after.
pub(crate) fn signed(method: &str, uri: &str) -> Request<Body> {
    signed_with(method, uri, Body::empty())
}

/// A signed request carrying a json body.
///
/// The body is not part of what is signed — see `middleware::signature` for
/// why that is the scheme's limit and not this helper's.
pub(crate) fn signed_json(
    method: &str,
    uri: &str,
    body: &serde_json::Value,
) -> Request<Body> {
    let mut request = signed_with(method, uri, Body::from(body.to_string()));

    request
        .headers_mut()
        .insert("content-type", "application/json".parse().unwrap());

    request
}

/// A signed request with a bearer token on it, as an authenticated luxctl call
/// carries both.
pub(crate) fn signed_as(method: &str, uri: &str, token: &str) -> Request<Body> {
    let mut request = signed(method, uri);

    request
        .headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());

    request
}

fn signed_with(method: &str, uri: &str, body: Body) -> Request<Body> {
    let at = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap();

    // The path alone is signed, so a uri carrying a query string has to be
    // split before signing — exactly as luxctl builds the path separately from
    // the parameters it hangs off it.
    let path = uri.split('?').next().unwrap_or(uri);

    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-luxctl-signature", sign(&SECRET.into(), method, path, at))
        .header("x-luxctl-timestamp", at.to_string())
        .body(body)
        .unwrap()
}
