//! The luxctl boundary: an HMAC over the body, verified here and nowhere else.

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::config::Config;

type HmacSha256 = Hmac<Sha256>;

/// A signed body larger than this is refused outright. The whole body has to be
/// buffered to compute the HMAC over it, so without a ceiling an unauthenticated
/// caller can make the process allocate as much as it likes.
const MAX_SIGNED_BODY: usize = 1024 * 1024;

const SIGNATURE_HEADER: &str = "x-luxctl-signature";

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

    let provided = request
        .headers()
        .get(SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_default();

    // The body has to be buffered to sign over it, then handed onward intact.
    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_SIGNED_BODY).await else {
        return deny();
    };

    if !matches(&config.luxctl_secret, &bytes, &provided) {
        return deny();
    }

    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}

fn matches(secret: &str, body: &[u8], provided: &str) -> bool {
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

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }

    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;

    #[test]
    fn rejects_a_wrong_signature() {
        assert!(!matches("secret", b"", "00"));
        assert!(!matches("secret", b"", "not-hex"));
        assert!(!matches("secret", b"", ""));
    }

    #[test]
    fn accepts_a_correct_signature() {
        let mut mac = HmacSha256::new_from_slice(b"secret").unwrap();
        mac.update(b"payload");
        let signature = mac.finalize().into_bytes();
        let hex = signature.iter().fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        });

        assert!(matches("secret", b"payload", &hex));
        // same signature, different body
        assert!(!matches("secret", b"tampered", &hex));
        // right body, wrong secret
        assert!(!matches("other", b"payload", &hex));
    }
}
