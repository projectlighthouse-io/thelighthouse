//! The one error type that crosses this crate's boundary.

/// What can go wrong between here and a payment provider.
///
/// Deliberately coarse. A caller's only real decisions are *tell the reader it
/// did not work*, *retry later*, and *this is our bug* — and a variant per
/// Stripe error code would push the provider's vocabulary through the trait
/// that exists to keep it out.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Two drivers registered under one name.
    ///
    /// A startup failure rather than a warning: the second registration would
    /// otherwise be dropped silently and read, much later, as the first one
    /// holding the wrong credentials.
    #[error("{provider} is registered twice")]
    Duplicate {
        /// The name that was registered twice.
        provider: String,
    },

    /// A webhook body did not match its signature, or arrived too long after
    /// it was signed.
    ///
    /// Carries no detail on purpose. Anything more specific — wrong digest
    /// versus stale timestamp — tells whoever is guessing which half they got
    /// right.
    #[error("the webhook signature does not verify")]
    Signature,

    /// Something arrived in a shape this crate cannot read: a webhook body, a
    /// response, a config file.
    #[error("{what} could not be read: {cause}")]
    Malformed {
        /// What was being read, for the log line.
        what: &'static str,
        /// Why it could not be.
        cause: String,
    },

    /// The provider could not be reached, or did not answer in time.
    ///
    /// Distinct from [`Error::Refused`] because the two want opposite
    /// handling: a refusal is an answer and repeating it changes nothing,
    /// while this is the case where the request may or may not have been
    /// performed — which is exactly why a retry of it needs an idempotency
    /// key.
    ///
    /// The cause is a string rather than the underlying transport error, so
    /// that the http client this crate happens to use does not become part of
    /// its public api.
    #[error("the provider could not be reached: {cause}")]
    Unreachable {
        /// What the transport said.
        cause: String,
    },

    /// The system random source could not be read.
    ///
    /// Only reached where randomness is load-bearing — minting an idempotency
    /// key. Failing the request is correct: the retries the key would have
    /// made safe are not safe without it.
    #[error("the system random source is unavailable")]
    Entropy,

    /// The provider answered, and the answer was no.
    ///
    /// A card decline lives here, and so does a bad price id. The caller
    /// separates them by `status`: a 4xx is an answer and must not be retried,
    /// which is exactly the distinction the request strategy is built on.
    #[error("the provider refused the request: {code}")]
    Refused {
        /// The provider's HTTP status.
        status: u16,
        /// The provider's machine-readable code, kept verbatim for the log.
        code: String,
        /// The provider's own message. Never shown to a reader — it is written
        /// for whoever integrated, not whoever is paying.
        message: String,
    },
}

impl Error {
    /// Whether sending the same request again could plausibly do better.
    ///
    /// Transport failures and the provider's own overload signals, and nothing
    /// else. A 402 is a declined card and a 404 is a subscription that does
    /// not exist: both are answers, and asking again produces the same one
    /// while looking, from the provider's side, like a client that cannot take
    /// no for an answer.
    #[must_use]
    pub fn retryable(&self) -> bool {
        match self {
            Self::Unreachable { .. } => true,
            Self::Refused { status, .. } => *status == 429 || *status >= 500,
            Self::Duplicate { .. }
            | Self::Signature
            | Self::Malformed { .. }
            | Self::Entropy => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(status: u16) -> Error {
        Error::Refused {
            status,
            code: "whatever".to_owned(),
            message: String::new(),
        }
    }

    #[test]
    fn only_a_hiccup_is_worth_repeating() {
        assert!(
            Error::Unreachable {
                cause: String::new()
            }
            .retryable()
        );
        assert!(refused(429).retryable());
        assert!(refused(500).retryable());
        assert!(refused(503).retryable());

        // The ones that must never be sent twice: a decline, a bad price, a
        // subscription that is not there.
        assert!(!refused(402).retryable());
        assert!(!refused(400).retryable());
        assert!(!refused(404).retryable());
        assert!(!Error::Signature.retryable());
    }
}
