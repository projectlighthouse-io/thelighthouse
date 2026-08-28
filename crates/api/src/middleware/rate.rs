//! The limit every route inherits, and what it counts against.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::RETRY_AFTER},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{api::AppState, session::Session};

/// Cloudflare's own header, and the only address here worth trusting.
///
/// Cloudflare sets it from the connection it terminated and **overwrites**
/// whatever the client sent, so a caller cannot choose their own bucket by
/// supplying one. That is only true of traffic that actually came through
/// Cloudflare — see [`caller`].
const CF_CONNECTING_IP: &str = "cf-connecting-ip";

/// What everyone without an address counts against, together.
///
/// One shared bucket rather than one bucket each: an unattributable request is
/// exactly the shape of the traffic worth limiting, and giving each its own
/// counter would be no limit at all.
const UNATTRIBUTED: &str = "ip:unknown";

/// Counts one request against the caller's budget, and refuses past it.
///
/// `RateLimiter::for('api')` from the laravel app: sixty a minute, keyed by
/// the reader when there is one and by address otherwise.
pub(crate) async fn limit_requests(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let key = caller(&request);

    let Some(retry_after) = state.requests.check(&key) else {
        return next.run(request).await;
    };

    tracing::info!(%key, "rate limit reached");

    let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();

    // A 429 with no `Retry-After` tells a client to back off but not by how
    // much, so it guesses, and the guess is usually "immediately".
    if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
        response.headers_mut().insert(RETRY_AFTER, value);
    }

    response
}

/// Who this request counts against.
///
/// The reader first, so one office behind one address is not one bucket, and
/// so a signed-in reader keeps their budget across a changing address.
///
/// This runs *outside* `require_reader`, so there is usually no `Session` in
/// the extensions — only the routes that mount that gate insert one. Falling
/// back to the address is therefore the common path, not the exception.
fn caller(request: &Request) -> String {
    if let Some(session) = request.extensions().get::<Session>() {
        return format!("user:{}", session.user_id);
    }

    address(request.headers())
}

/// The client's address, as far as it can be believed.
///
/// **This is only as trustworthy as the firewall.** Cloudflare overwrites
/// `CF-Connecting-IP`, so through the edge it is the real caller and cannot be
/// chosen. Reaching the origin directly, it can be set to anything — and the
/// answer to that is the firewall that lets only Cloudflare's ranges reach
/// `:443`, not anything this function could do.
///
/// `X-Forwarded-For` is deliberately not consulted. Caddy appends to whatever
/// the client sent, so its first entry is the client's own claim and its last
/// is Cloudflare's address rather than the reader's — one is forgeable and the
/// other is useless as a key.
fn address(headers: &HeaderMap) -> String {
    headers
        .get(CF_CONNECTING_IP)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(|| UNATTRIBUTED.to_owned(), |ip| format!("ip:{ip}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(
                axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        headers
    }

    #[test]
    fn cloudflares_header_is_the_address() {
        assert_eq!(
            address(&headers(&[("cf-connecting-ip", "203.0.113.7")])),
            "ip:203.0.113.7"
        );
    }

    #[test]
    fn a_forwarded_for_header_alone_buys_nothing() {
        // Forgeable at the first entry and useless at the last, so it is not a
        // key. Everything carrying only this shares one bucket.
        let key = address(&headers(&[("x-forwarded-for", "203.0.113.7")]));

        assert_eq!(key, UNATTRIBUTED);
    }

    #[test]
    fn an_empty_or_absent_address_is_not_its_own_bucket() {
        assert_eq!(address(&HeaderMap::new()), UNATTRIBUTED);
        assert_eq!(
            address(&headers(&[("cf-connecting-ip", "  ")])),
            UNATTRIBUTED
        );
    }

    #[test]
    fn a_reader_and_an_address_can_never_collide() {
        // Different namespaces, so a user id can never be spent by somebody
        // arriving from an address that happens to look like one.
        assert!(address(&HeaderMap::new()).starts_with("ip:"));
        assert!(!format!("user:{}", 7).starts_with("ip:"));
    }
}
