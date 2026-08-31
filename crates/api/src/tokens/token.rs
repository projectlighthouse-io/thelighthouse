//! The string a reader pastes, taken apart and hashed.
//!
//! No database, no HTTP. Everything here is a pure function over the header,
//! which is what lets the awkward cases — an empty half, two bars, a bar in
//! the secret — be tested without a row existing.

use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// A `Bearer` value that at least has the right shape.
///
/// Constructed only by [`Presented::parse`], so there is no way to hold one
/// whose halves were never checked. It is *not* a proof of anything: the id is
/// public and the secret has not been compared to a row yet.
#[derive(Debug)]
pub(crate) struct Presented {
    /// `personal_access_tokens.id`. A lookup key, not a credential.
    pub(crate) id: i64,
    /// The half that is the proof. Never logged, never stored, never returned.
    secret: String,
}

impl Presented {
    /// Takes apart `Authorization: Bearer {id}|{secret}`.
    ///
    /// `None` for anything that is not that, and deliberately with no reason
    /// attached: a caller that could tell "no such id" from "malformed" from
    /// "wrong secret" could use the difference, and there is nothing useful it
    /// could do with it either way.
    ///
    /// Split on the *first* bar, so a secret containing one survives. Sanctum
    /// generates 40 alphanumerics and would never produce one — but a parser
    /// that only works because of what the generator happens to emit is a
    /// parser that breaks when the generator changes.
    pub(crate) fn parse(header: &str) -> Option<Self> {
        let value = header.strip_prefix("Bearer ")?.trim();
        let (id, secret) = value.split_once('|')?;

        if secret.is_empty() {
            return None;
        }

        Some(Self {
            id: id.parse().ok()?,
            secret: secret.to_owned(),
        })
    }

    /// Whether this is the secret behind `stored`.
    ///
    /// `stored` is what the column holds: lowercase hex of the sha256 of the
    /// secret, which is what Sanctum writes. Compared in constant time, for
    /// the reason the HMAC in `middleware::signature` is — a byte-by-byte
    /// compare leaks the correct prefix through timing, and here the thing
    /// being leaked is a credential rather than a signature over a path.
    ///
    /// No salt and no work factor, and that is right for this one case: the
    /// secret is 40 characters from a CSPRNG rather than something a human
    /// chose, so there is no dictionary to run and nothing for a rainbow table
    /// to precompute. A password would need argon2; this is not a password.
    #[must_use]
    pub(crate) fn matches(&self, stored: &str) -> bool {
        use std::fmt::Write as _;

        let hex = Sha256::digest(self.secret.as_bytes()).iter().fold(
            String::with_capacity(64),
            |mut out, byte| {
                let _ = write!(out, "{byte:02x}");
                out
            },
        );

        // `ct_eq` on slices of different lengths returns false in variable
        // time, which leaks only the *length* of the column's value — not a
        // secret, and the digest's length is fixed and public anyway.
        hex.as_bytes().ct_eq(stored.as_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// sha256("secret"), as `personal_access_tokens.token` would hold it.
    const SECRET_SHA256: &str =
        "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b";

    #[test]
    fn a_token_is_an_id_and_a_secret_either_side_of_a_bar() {
        let presented = Presented::parse("Bearer 42|secret").unwrap();

        assert_eq!(presented.id, 42);
        assert!(presented.matches(SECRET_SHA256));
    }

    #[test]
    fn the_secret_keeps_any_bars_of_its_own() {
        // Split on the first bar, not on every one. Sanctum's generator would
        // not produce this; a parser that relies on that is one that breaks
        // when the generator is replaced.
        let presented = Presented::parse("Bearer 7|a|b").unwrap();

        assert_eq!(presented.id, 7);
        assert!(presented.matches(&hex_sha256("a|b")));
    }

    #[test]
    fn anything_that_is_not_that_shape_is_refused() {
        for header in [
            "",
            "Bearer",
            "Bearer ",
            "Bearer nobar",
            // Both halves have to be there.
            "Bearer 42|",
            "Bearer |secret",
            // The id is a row id, not a word.
            "Bearer abc|secret",
            "Bearer 4.2|secret",
            // The scheme is part of the contract; a bare token is not one.
            "42|secret",
            "Basic 42|secret",
            // Case matters: `HeaderMap` normalises the header *name*, never
            // its value, and Sanctum's scheme is spelled `Bearer`.
            "bearer 42|secret",
        ] {
            assert!(Presented::parse(header).is_none(), "{header:?}");
        }
    }

    #[test]
    fn a_wrong_secret_does_not_match() {
        let presented = Presented::parse("Bearer 42|not-the-secret").unwrap();

        assert!(!presented.matches(SECRET_SHA256));
    }

    #[test]
    fn a_column_holding_something_that_is_not_a_digest_never_matches() {
        // What a half-migrated or hand-edited row looks like. It must fail
        // closed rather than compare equal to a short answer.
        let presented = Presented::parse("Bearer 42|secret").unwrap();

        for stored in ["", "not-hex", &SECRET_SHA256[..63], "plaintext"] {
            assert!(!presented.matches(stored), "{stored:?}");
        }
    }

    #[test]
    fn the_digest_is_lowercase_hex_of_the_secret_half_only() {
        // The wire format the laravel rows are already written in. If this
        // ever changes, every token a reader configured stops working — so it
        // is asserted against a value computed elsewhere, not against
        // `matches` calling itself.
        assert_eq!(hex_sha256("secret"), SECRET_SHA256);
        assert!(
            Presented::parse("Bearer 1|secret")
                .unwrap()
                .matches(SECRET_SHA256)
        );
        // The id half is not in the digest.
        assert!(
            Presented::parse("Bearer 99|secret")
                .unwrap()
                .matches(SECRET_SHA256)
        );
    }

    fn hex_sha256(value: &str) -> String {
        use std::fmt::Write as _;

        Sha256::digest(value.as_bytes()).iter().fold(
            String::new(),
            |mut out, byte| {
                let _ = write!(out, "{byte:02x}");
                out
            },
        )
    }
}
