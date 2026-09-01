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
