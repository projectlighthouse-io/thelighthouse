//! How hard to try a request, and how to try it twice without paying twice.
//!
//! Retrying a payment request is not like retrying a read. A `POST` that
//! creates a charge and then times out may well have succeeded, and sending it
//! again is how one subscription becomes two. Every payment provider worth
//! integrating solves this the same way — a key the caller chooses, under
//! which the provider will perform an operation exactly once — so the rule
//! lives here rather than inside any one driver.

use std::time::Duration;

use crate::error::Error;

/// The longest key a provider will honour.
///
/// Stripe's limit, and the one worth assuming: it does not reject a longer
/// key, it ignores it. A silently ignored idempotency key is a duplicate
/// charge rather than an error, so the check happens here, at construction,
/// before anything can be sent under it.
const MAX_KEY: usize = 255;

/// How long to wait before the second attempt. Doubles from there.
const BASE_BACKOFF: Duration = Duration::from_millis(250);

/// A key under which a provider performs an operation at most once.
///
/// Send the same key with the same request twice and the second call returns
/// the first call's result rather than doing the work again. That is what
/// makes a retry safe, and it is the only thing that does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Use a key the caller chose.
    ///
    /// Worth doing whenever the operation has a natural one — "subscribe user
    /// 41 to `voyage_yearly`" is a key, and using it means that a reader who
    /// double-clicks, or a request replayed by a proxy, still buys one
    /// subscription. A random key protects a retry; a chosen key protects
    /// everything.
    ///
    /// # Errors
    ///
    /// [`Error::Malformed`] if the key is empty or longer than 255 characters.
    pub fn new(key: impl Into<String>) -> Result<Self, Error> {
        let key = key.into();

        if key.is_empty() || key.len() > MAX_KEY {
            return Err(Error::Malformed {
                what: "an idempotency key",
                cause: format!(
                    "must be between 1 and {MAX_KEY} bytes, and this one is {}",
                    key.len()
                ),
            });
        }

        Ok(Self(key))
    }

    /// Mint a random key, for an operation with no natural one.
    ///
    /// # Errors
    ///
    /// [`Error::Entropy`] if the system random source cannot be read.
    pub fn random() -> Result<Self, Error> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| Error::Entropy)?;

        let mut key = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write as _;
            // Writing to a String cannot fail; the result is discarded rather
            // than unwrapped so this stays a formatting detail.
            let _ = write!(key, "{byte:02x}");
        }

        Ok(Self(key))
    }

    /// Borrow the key, to put in a header.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Possible strategies for sending a request, including retry behaviour and
/// use of idempotency keys.
///
/// Set per driver, at registration. A driver that is handed one applies it to
/// every request it sends.
#[derive(Clone, Debug, Default)]
pub enum RequestStrategy {
    /// Run the request once.
    ///
    /// The default, and the right answer for a read: there is nothing to make
    /// idempotent, and a failed read is the caller's to repeat or not.
    #[default]
    Once,
    /// Run it once under a key the caller chose.
    Idempotent(IdempotencyKey),
    /// Try up to this many times, all under one random key, with no pause
    /// between attempts.
    Retry(u32),
    /// Try up to this many times, all under one random key, waiting longer
    /// after each failure.
    ExponentialBackoff(u32),
}

impl RequestStrategy {
    /// Begin one operation: mint whatever key it will run under, and say how
    /// many times it may be attempted.
    ///
    /// **The key is minted here, once, and every attempt reuses it.** A key
    /// regenerated per attempt is not idempotency at all — it is a loop that
    /// charges once per timeout, which is the exact failure this module
    /// exists to prevent.
    ///
    /// # Errors
    ///
    /// [`Error::Entropy`] if a random key is needed and cannot be minted.
    /// Failing the operation is correct: without a key, the retries this
    /// strategy asks for are not safe to make.
    pub fn begin(&self) -> Result<Attempts, Error> {
        let (count, key, backoff) = match self {
            Self::Once => (1, None, false),
            Self::Idempotent(key) => (1, Some(key.clone()), false),
            Self::Retry(count) => {
                (*count, Some(IdempotencyKey::random()?), false)
            }
            Self::ExponentialBackoff(count) => {
                (*count, Some(IdempotencyKey::random()?), true)
            }
        };

        Ok(Attempts {
            // A strategy asking for zero attempts means somebody wrote a zero,
            // not that the request should never be sent. One is the floor.
            count: count.max(1),
            key,
            backoff,
        })
    }
}

/// One operation's plan: the key it runs under, and how many goes it gets.
///
/// Produced by [`RequestStrategy::begin`] and consumed by a driver's send
/// loop, which owns the two decisions this cannot make for it: what counts as
/// a failure worth repeating, and how to sleep.
#[derive(Clone, Debug)]
pub struct Attempts {
    count: u32,
    key: Option<IdempotencyKey>,
    backoff: bool,
}

impl Attempts {
    /// How many times this may be sent, in total.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// The key every attempt must carry, if there is one.
    #[must_use]
    pub const fn key(&self) -> Option<&IdempotencyKey> {
        self.key.as_ref()
    }

    /// How long to wait before the attempt after this one.
    ///
    /// `attempt` is zero-based. Doubles each time, from 250ms.
    ///
    /// No jitter, deliberately. Jitter is what stops a fleet of retrying
    /// clients synchronising into a thundering herd, and the shape of this
    /// traffic — a handful of people checking out — cannot make one. Add it
    /// alongside the first bulk operation, not before.
    #[must_use]
    pub fn backoff(&self, attempt: u32) -> Duration {
        if !self.backoff {
            return Duration::ZERO;
        }

        BASE_BACKOFF
            .saturating_mul(1u32.checked_shl(attempt).unwrap_or(u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_longer_than_the_provider_honours_is_refused_before_it_is_sent() {
        // The failure this prevents is not an error from the provider. It is
        // the provider quietly ignoring the key and charging twice.
        let too_long = "k".repeat(MAX_KEY + 1);

        assert!(IdempotencyKey::new(too_long).is_err());
        assert!(IdempotencyKey::new("").is_err());
        assert!(IdempotencyKey::new("k".repeat(MAX_KEY)).is_ok());
    }

    #[test]
    fn a_random_key_is_hex_and_not_the_same_one_twice() {
        let first = IdempotencyKey::random().unwrap();
        let second = IdempotencyKey::random().unwrap();

        assert_eq!(first.as_str().len(), 32);
        assert!(first.as_str().chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }

    #[test]
    fn one_attempt_and_no_key_is_what_a_plain_request_gets() {
        let attempts = RequestStrategy::Once.begin().unwrap();

        assert_eq!(attempts.count(), 1);
        assert!(attempts.key().is_none());
        assert_eq!(attempts.backoff(0), Duration::ZERO);
    }

    #[test]
    fn a_chosen_key_survives_into_the_attempt() {
        let key = IdempotencyKey::new("subscribe:41:voyage_yearly").unwrap();
        let attempts =
            RequestStrategy::Idempotent(key.clone()).begin().unwrap();

        assert_eq!(attempts.count(), 1);
        assert_eq!(attempts.key(), Some(&key));
    }

    #[test]
    fn every_attempt_of_one_operation_shares_one_key() {
        // Stated as a test because the alternative — a key per attempt —
        // compiles, looks reasonable, and charges the reader once per timeout.
        let attempts = RequestStrategy::Retry(3).begin().unwrap();
        let key = attempts.key().unwrap().clone();

        assert_eq!(attempts.count(), 3);
        for _ in 0..attempts.count() {
            assert_eq!(attempts.key(), Some(&key));
        }
    }

    #[test]
    fn two_operations_do_not_share_a_key() {
        let first = RequestStrategy::Retry(2).begin().unwrap();
        let second = RequestStrategy::Retry(2).begin().unwrap();

        assert_ne!(first.key(), second.key());
    }

    #[test]
    fn backing_off_doubles_and_not_backing_off_does_not_wait() {
        let backing_off =
            RequestStrategy::ExponentialBackoff(4).begin().unwrap();

        assert_eq!(backing_off.backoff(0), Duration::from_millis(250));
        assert_eq!(backing_off.backoff(1), Duration::from_millis(500));
        assert_eq!(backing_off.backoff(2), Duration::from_millis(1000));

        let immediate = RequestStrategy::Retry(4).begin().unwrap();
        assert_eq!(immediate.backoff(2), Duration::ZERO);
    }

    #[test]
    fn a_strategy_asking_for_no_attempts_still_sends_once() {
        assert_eq!(RequestStrategy::Retry(0).begin().unwrap().count(), 1);
    }
}
