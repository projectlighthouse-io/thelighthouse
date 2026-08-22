//! Every way a login can fail.
//!
//! Split by what the caller should *do*, not by where the failure happened:
//! [`Error::InvalidState`] is a redirect back to the login page, and everything
//! else is a log line and a generic apology. A caller that cannot tell those
//! apart ends up showing the same message for a lost session as for a provider
//! outage.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The callback's state did not match the session's. Either the login was
    /// started somewhere else — which is the CSRF this check exists for — or
    /// the session was lost in between.
    #[error("the callback state does not match the state issued at redirect")]
    InvalidState,

    /// The same provider was registered twice at boot. A config mistake, and
    /// the only one here that is not a runtime failure — silently keeping the
    /// first registration would surface much later as wrong credentials.
    #[error("{provider} was registered more than once")]
    Duplicate { provider: &'static str },

    /// The provider answered, and said no.
    #[error("{provider} rejected the authorization code: {message}")]
    Rejected {
        provider: &'static str,
        message: String,
    },

    /// The request never got an answer, or the answer did not parse.
    ///
    /// The source is kept rather than flattened into a string: reqwest's own
    /// message distinguishes a DNS failure from a timeout from a 500, and that
    /// distinction is the whole value of the log line.
    #[error("{provider} request failed")]
    Http {
        provider: &'static str,
        #[source]
        source: reqwest::Error,
    },

    #[error("could not read from the system random source")]
    Entropy(#[from] getrandom::Error),
}
