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

/// What a request with no address at all counts against.
///
/// One shared bucket rather than one bucket each: an unattributable request is
/// exactly the shape of the traffic worth limiting, and giving each its own
/// counter would be no limit at all.
///
/// Reached only when the peer address is missing too, which in practice means
/// a test router built without connect info — a real listener always has one.
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
///
/// Shared with `newsletter`, which limits an anonymous form against its own
/// budget and needs the same answer to "who is this" — including the same
/// rules about which address header can be believed. A second implementation
/// would be a second set of those rules to keep right.
pub(crate) fn caller(request: &Request) -> String {
    if let Some(session) = request.extensions().get::<Session>() {
        return format!("user:{}", session.user_id);
    }

    address(request.headers()).unwrap_or_else(|| peer(request))
}

/// The address the connection actually came from.
///
/// The fallback when Cloudflare's header is absent, which is every request
/// that did not come through the edge: nitro's server-side renders over
/// loopback, a health probe, anything reaching the origin directly.
///
/// **Behind a proxy this is the proxy**, so it does not tell one reader from
/// another — that is what `CF-Connecting-IP` is for, and why it is preferred.
/// What it does do is stop all of that traffic sharing one bucket with every
/// other unattributable request, which is what made a prerender exhaust the
/// budget in a few seconds.
fn peer(request: &Request) -> String {
    request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map_or_else(
            || UNATTRIBUTED.to_owned(),
            |peer| format!("peer:{}", peer.0.ip()),
        )
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
fn address(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CF_CONNECTING_IP)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|ip| format!("ip:{ip}"))
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
            Some("ip:203.0.113.7".to_owned())
        );
    }

    #[test]
    fn a_forwarded_for_header_alone_buys_nothing() {
        // Forgeable at the first entry and useless at the last, so it is not a
        // key. A request carrying only this falls back to its peer address.
        assert_eq!(
            address(&headers(&[("x-forwarded-for", "203.0.113.7")])),
            None
        );
    }

    #[test]
    fn an_empty_or_absent_header_is_not_an_address() {
        assert_eq!(address(&HeaderMap::new()), None);
        assert_eq!(address(&headers(&[("cf-connecting-ip", "  ")])), None);
    }

    #[test]
    fn the_peer_is_the_key_when_the_edge_did_not_see_the_request() {
        // Server-side rendering calls this api over loopback and Cloudflare
        // never sees it. Before this fallback every such request shared one
        // bucket, and a prerender spent the whole budget in seconds.
        let mut request = Request::new(axum::body::Body::empty());
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            std::net::SocketAddr::from(([127, 0, 0, 1], 54321)),
        ));

        assert_eq!(caller(&request), "peer:127.0.0.1");
    }

    #[test]
    fn a_request_from_nowhere_at_all_still_has_a_bucket() {
        assert_eq!(
            caller(&Request::new(axum::body::Body::empty())),
            UNATTRIBUTED
        );
    }

    #[test]
    fn a_reader_an_address_and_a_peer_can_never_collide() {
        // Different namespaces, so a user id can never be spent by somebody
        // arriving from an address that happens to look like one.
        assert!(
            address(&headers(&[("cf-connecting-ip", "203.0.113.7")]))
                .unwrap()
                .starts_with("ip:")
        );
        assert!(!format!("user:{}", 7).starts_with("ip:"));
        assert!(!"peer:127.0.0.1".starts_with("ip:"));
    }
}
