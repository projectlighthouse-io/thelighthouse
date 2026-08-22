//! The CSRF state: minting it, and comparing it on the way back.
//!
//! The state is the only thing tying a callback to a login that this site
//! started. Without it, anyone can send a browser to the callback URL with
//! their own `code` and sign the victim into the attacker's account.

use std::fmt::Write as _;

use subtle::ConstantTimeEq;

use crate::error::Error;

/// 32 bytes of CSPRNG output, hex encoded.
///
/// Socialite uses a 40-character alphanumeric string. This is longer and the
/// bytes come from the OS rather than a userspace generator, which for a value
/// whose whole job is being unguessable is the part that matters.
///
/// # Errors
///
/// [`Error::Entropy`] if the system random source cannot be read. There is no
/// fallback on purpose — a predictable state is worse than no login.
pub fn random_state() -> Result<String, Error> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)?;

    Ok(bytes.iter().fold(String::new(), |mut out, b| {
        // Infallible: writing to a String cannot fail, and the lints rule out
        // saying so with an unwrap.
        let _ = write!(out, "{b:02x}");
        out
    }))
}

/// Constant time, and an empty expected state never matches.
///
/// Empty means no state was issued, or the session is gone. Comparing two empty
/// strings would say "match" and wave the callback through, which is the one
/// answer this function must never give.
pub(crate) fn matches(expected: &str, provided: &str) -> bool {
    if expected.is_empty() {
        return false;
    }

    let (expected, provided) = (expected.as_bytes(), provided.as_bytes());

    // ct_eq is only constant time over equal-length inputs; the lengths
    // themselves are not secret.
    expected.len() == provided.len() && bool::from(expected.ct_eq(provided))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_must_match_exactly() {
        assert!(matches("abc", "abc"));
        assert!(!matches("abc", "abd"));
        assert!(!matches("abc", "ab"));
        assert!(!matches("abc", "abcd"));
    }

    #[test]
    fn a_missing_state_never_matches() {
        // The session had none, so nothing may pass — including another empty.
        assert!(!matches("", ""));
        assert!(!matches("", "anything"));
    }

    #[test]
    fn random_state_is_not_a_constant() {
        let (a, b) = (random_state().unwrap(), random_state().unwrap());

        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
