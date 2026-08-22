//! The session cookie: what is in it, and why it can be trusted.
//!
//! There is no session table, because there is no users table yet. The cookie
//! *is* the session — a JSON payload plus an HMAC over it, so the process can
//! verify a cookie it has no memory of issuing. A server-side store with
//! nothing to join against would be a table, a migration and a connection pool
//! bought for zero behaviour.
//!
//! What that costs, stated rather than discovered later: **a session cannot be
//! revoked before it expires.** Signing out clears the reader's own cookie; a
//! copy lifted off that browser stays valid until `exp`. When the users table
//! lands, the payload shrinks to a user id and revocation becomes a row — the
//! two functions below are the only place that has to change.
//!
//! Rotating `SESSION_SECRET` invalidates every outstanding session, which is
//! the blunt revocation available in the meantime.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// 30 days. Long enough that a reader is not signed out between visits, short
/// enough that a stolen cookie is not a permanent key — the compromise a
/// stateless session forces, since nothing else can cut it short.
pub(crate) const MAX_AGE: u64 = 60 * 60 * 24 * 30;

/// Everything the frontend needs to render signed-in chrome, and nothing else.
///
/// The profile is carried here rather than refetched because there is nowhere
/// to refetch it from. It goes back to being a lookup the moment there is a
/// users table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Session {
    /// `provider:id`. Stable across name and email changes, which is what makes
    /// it the thing a future `users` row keys on.
    pub(crate) sub: String,
    pub(crate) provider: String,
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) avatar: Option<String>,
    /// Unix seconds. Checked on the way in, and mirrored by the cookie's
    /// `Max-Age` — the cookie expiring is a courtesy, this is the enforcement.
    pub(crate) exp: u64,
}

/// Unix seconds, or 0 if the clock is somehow before the epoch.
///
/// Zero fails closed: every session then reads as expired, which is a site full
/// of signed-out readers rather than a site that accepts anything.
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default()
}

/// `<base64url payload>.<base64url hmac>`.
///
/// `None` only if the payload cannot be serialised, which for this struct means
/// the process is out of memory. The caller turns it into an apology, not a
/// login.
pub(crate) fn encode(secret: &str, session: &Session) -> Option<String> {
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(session).ok()?);
    let tag = sign(secret, payload.as_bytes())?;

    Some(format!("{payload}.{tag}"))
}

/// The reverse, and the only place a cookie is allowed to become a reader.
///
/// Every failure — no separator, a bad tag, unparseable JSON, past `exp` — is
/// the same `None`. A caller cannot accidentally distinguish "tampered" from
/// "expired" and act on the difference.
pub(crate) fn decode(secret: &str, cookie: &str) -> Option<Session> {
    let (payload, tag) = cookie.split_once('.')?;

    let expected = sign(secret, payload.as_bytes())?;

    // Constant time, and length-checked first: ct_eq is only constant time over
    // equal-length inputs, and the length of a signature is not a secret.
    if expected.len() != tag.len() || !bool::from(expected.as_bytes().ct_eq(tag.as_bytes())) {
        return None;
    }

    let session: Session = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()?;

    // Signed and expired is still expired. The signature proves we issued it,
    // not that it is still good.
    (session.exp > now()).then_some(session)
}

/// Signs the *encoded* payload rather than the raw JSON, so verification never
/// has to decode attacker-supplied base64 before deciding whether to trust it.
fn sign(secret: &str, payload: &[u8]) -> Option<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(payload);

    Some(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "session-secret";

    fn session() -> Session {
        Session {
            sub: "github:1".to_owned(),
            provider: "github".to_owned(),
            name: Some("Octocat".to_owned()),
            email: Some("o@x.test".to_owned()),
            avatar: None,
            exp: now() + MAX_AGE,
        }
    }

    #[test]
    fn a_session_survives_the_round_trip() {
        let issued = session();
        let cookie = encode(SECRET, &issued).unwrap();

        assert_eq!(decode(SECRET, &cookie), Some(issued));
    }

    #[test]
    fn the_cookie_is_readable_but_not_writable() {
        // Not encryption: the payload is plain base64 and a reader can decode
        // their own. Nothing secret goes in it — the signature is what stops it
        // being edited.
        let cookie = encode(SECRET, &session()).unwrap();
        let (payload, _) = cookie.split_once('.').unwrap();
        let decoded = URL_SAFE_NO_PAD.decode(payload).unwrap();

        assert!(String::from_utf8(decoded).unwrap().contains("github:1"));
    }

    #[test]
    fn an_edited_payload_is_rejected() {
        let cookie = encode(SECRET, &session()).unwrap();
        let (payload, tag) = cookie.split_once('.').unwrap();

        // A forged session for somebody else, carrying the original signature.
        let mine = Session {
            sub: "github:2".to_owned(),
            ..session()
        };
        let forged = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&mine).unwrap());

        assert_ne!(forged, payload);
        assert!(decode(SECRET, &format!("{forged}.{tag}")).is_none());
    }

    #[test]
    fn another_secret_cannot_mint_a_session() {
        let cookie = encode("some-other-secret", &session()).unwrap();

        assert!(decode(SECRET, &cookie).is_none());
    }

    #[test]
    fn an_expired_session_is_rejected_even_though_it_is_signed() {
        let stale = Session {
            exp: now() - 1,
            ..session()
        };
        let cookie = encode(SECRET, &stale).unwrap();

        assert!(decode(SECRET, &cookie).is_none());
    }

    #[test]
    fn a_malformed_cookie_is_just_a_signed_out_reader() {
        assert!(decode(SECRET, "").is_none());
        assert!(decode(SECRET, "no-separator").is_none());
        assert!(decode(SECRET, ".").is_none());
        assert!(decode(SECRET, "not-base64!.tag").is_none());
        // right shape, empty signature
        assert!(decode(SECRET, "e30.").is_none());
    }
}
