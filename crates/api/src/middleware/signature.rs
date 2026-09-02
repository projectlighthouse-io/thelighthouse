//! The luxctl boundary: an HMAC over the method and the path, verified here
//! and nowhere else.
//!
//! **The scheme is not ours to choose.** luxctl is installed on readers'
//! machines with the client secret baked in at build time, and it signs
//!
//! ```text
//!   "{unix seconds}.{METHOD}.{path}"
//! ```
//!
//! sending the hex digest as `X-Luxctl-Signature` and the same timestamp as
//! `X-Luxctl-Timestamp`. That is what `VerifyLuxctlClient` in the laravel app
//! checks, byte for byte, and a binary somebody already installed cannot be
//! asked to sign something else. So this matches it rather than improving on
//! it, and the cutover is a DNS change instead of a forced upgrade.
//!
//! **The body is not signed, and that is the scheme's limit rather than an
//! oversight here.** Anyone who can rewrite a request in flight can change its
//! body and the signature still verifies. What the signature does establish is
//! that the caller holds the client secret and that the request is recent,
//! which is what it is for: keeping the api's surface closed to things that
//! are not luxctl. Everything a body can then do is gated a second time, by
//! the bearer token and by the reader's own id being bound into every
//! statement — this is the outer door, never the only one. Signing the body as
//! well would be strictly better and is a luxctl change first.
//!
//! **The timestamp is what stops a replay.** A captured request is valid for
//! five minutes and then is not, which bounds what a recorded signature is
//! worth without this process having to remember every one it has seen.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::config::Config;

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_HEADER: &str = "x-luxctl-signature";
const TIMESTAMP_HEADER: &str = "x-luxctl-timestamp";

/// How far out of step a request's clock may be, in seconds.
///
/// The laravel middleware's window, kept. luxctl's callers are on their own
/// laptops with their own clocks, and five minutes is loose enough that an
/// unsynchronised one still works while a captured request stops being worth
/// anything the same afternoon.
const MAX_SKEW: i64 = 300;

/// Rejects anything without a valid luxctl signature.
///
/// Caddy checks only that the header exists. It cannot verify an HMAC, so this
/// is the real boundary and must never trust the proxy's judgement.
pub(crate) async fn require_signature(
    State(config): State<Config>,
    request: Request,
    next: Next,
) -> Response {
    // 404 everywhere below, never 401 or 403: those confirm the endpoint is
    // there, which is free reconnaissance for anyone probing.
    let deny = || StatusCode::NOT_FOUND.into_response();

    let provided = header(request.headers(), SIGNATURE_HEADER);
    let timestamp = header(request.headers(), TIMESTAMP_HEADER);

    let Some(now) = unix_seconds() else {
        // A clock this process cannot read is a clock it cannot check skew
        // against, and the safe answer to "I do not know what now is" is no.
        tracing::error!("cannot read the clock; refusing signed requests");
        return deny();
    };

    if !recent(&timestamp, now) {
        return deny();
    }

    // The path only, never the query string: that is what luxctl signs, so
    // `?page=2` is not part of it.
    let payload =
        format!("{timestamp}.{}.{}", request.method(), request.uri().path());

    if !matches(&config.luxctl_secret, payload.as_bytes(), &provided) {
        return deny();
    }

    next.run(request).await
}

/// One header as a string, owned.
///
/// Owned rather than borrowed so nothing is still holding the request when it
/// is handed to `next.run`. A missing header and an unreadable one both read as
/// empty, which fails every check below — there is no case where the
/// difference between them changes the answer.
fn header(headers: &axum::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

/// Whether a request's own timestamp is close enough to ours.
///
/// Both directions, as the laravel middleware does: a clock ahead of ours is
/// as much a sign of a forged request as one behind it, and checking only the
/// past would let a caller mint a signature that is good for next year.
///
/// An absent or unparseable timestamp is refused rather than read as zero,
/// which would be an instant in 1970 and rejected as stale — the same answer
/// by accident rather than on purpose.
fn recent(timestamp: &str, now: i64) -> bool {
    let Ok(sent) = timestamp.parse::<i64>() else {
        return false;
    };

    // Saturating, then absolute: a hostile timestamp at either end of the
    // range must not wrap round into "a second ago".
    now.saturating_sub(sent).saturating_abs() <= MAX_SKEW
}

/// Unix seconds, or `None` for a clock set before the epoch.
fn unix_seconds() -> Option<i64> {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs())
        .ok()
}

fn matches(secret: &SecretString, payload: &[u8], provided: &str) -> bool {
    let Some(provided) = hex_decode(provided) else {
        return false;
    };
    let Ok(mut mac) =
        HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
    else {
        return false;
    };
    mac.update(payload);

    // Constant time. A byte-by-byte compare leaks the correct prefix through
    // timing, which is enough to forge a signature given enough attempts.
    mac.finalize().into_bytes().ct_eq(&provided).into()
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }

    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}

/// The signature luxctl would send for one request.
///
/// Here rather than in the test module because the route tests in other
/// modules need it to reach their own handlers at all — every signed route is
/// a 404 without one.
#[cfg(test)]
pub(crate) fn sign(
    secret: &SecretString,
    method: &str,
    path: &str,
    at: i64,
) -> String {
    use std::fmt::Write as _;

    let payload = format!("{at}.{method}.{path}");

    let Ok(mut mac) =
        HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
    else {
        return String::new();
    };
    mac.update(payload.as_bytes());

    mac.finalize()
        .into_bytes()
        .iter()
        .fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    /// The tests deal in string literals; the code deals in wrapped secrets.
    fn secret(key: &str) -> SecretString {
        key.into()
    }

    /// The one assertion that is about the *other* implementations rather than
    /// this one: the payload luxctl builds, spelled out, so a change to the
    /// format above fails here instead of in production against a CLI nobody
    /// can redeploy.
    #[test]
    fn the_payload_is_timestamp_then_method_then_path() {
        let signature =
            sign(&secret("dev-secret-0"), "GET", "/api/v1/ping", NOW);

        assert!(matches(
            &secret("dev-secret-0"),
            b"1800000000.GET./api/v1/ping",
            &signature
        ));
    }

    #[test]
    fn rejects_a_wrong_signature() {
        assert!(!matches(&secret("secret"), b"payload", "00"));
        assert!(!matches(&secret("secret"), b"payload", "not-hex"));
        assert!(!matches(&secret("secret"), b"payload", ""));
        // Odd length, so it does not divide into pairs.
        assert!(!matches(&secret("secret"), b"payload", "abc"));
    }

    #[test]
    fn a_signature_is_bound_to_its_method_and_its_path() {
        let signature = sign(&secret("secret"), "GET", "/api/v1/projects", NOW);
        let for_payload = |payload: String| {
            matches(&secret("secret"), payload.as_bytes(), &signature)
        };

        assert!(for_payload(format!("{NOW}.GET./api/v1/projects")));
        // Same secret, same second, a different request.
        assert!(!for_payload(format!("{NOW}.POST./api/v1/projects")));
        assert!(!for_payload(format!("{NOW}.GET./api/v1/projects/attempts")));
        assert!(!for_payload(format!("{}.GET./api/v1/projects", NOW + 1)));
        // The right request, the wrong secret.
        assert!(!matches(
            &secret("other"),
            format!("{NOW}.GET./api/v1/projects").as_bytes(),
            &signature
        ));
    }

    #[test]
    fn a_request_inside_the_window_is_recent_in_both_directions() {
        assert!(recent(&NOW.to_string(), NOW));
        assert!(recent(&(NOW - MAX_SKEW).to_string(), NOW));
        // A clock a little ahead of ours is somebody's laptop, not an attack.
        assert!(recent(&(NOW + MAX_SKEW).to_string(), NOW));
    }

    #[test]
    fn a_captured_request_stops_being_worth_anything() {
        assert!(!recent(&(NOW - MAX_SKEW - 1).to_string(), NOW));
        assert!(!recent(&(NOW + MAX_SKEW + 1).to_string(), NOW));
    }

    #[test]
    fn a_missing_or_unreadable_timestamp_is_refused_rather_than_read_as_zero() {
        for timestamp in ["", "  ", "nope", "1e9", "9223372036854775808"] {
            assert!(!recent(timestamp, NOW), "{timestamp:?}");
        }
    }

    #[test]
    fn a_timestamp_at_the_end_of_the_range_does_not_wrap_into_recent() {
        assert!(!recent(&i64::MIN.to_string(), NOW));
        assert!(!recent(&i64::MAX.to_string(), NOW));
    }

    #[test]
    fn the_clock_is_readable_and_in_range() {
        let now = unix_seconds().unwrap();

        // Some time after this was written.
        assert!(now > 1_700_000_000);
    }
}
