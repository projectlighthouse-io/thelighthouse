//! What starting a payment takes, and what it gives back.

/// Who is paying, as much of them as a provider needs to know.
///
/// Borrowed rather than owned: the caller already has these on hand from its
/// own session, and a checkout that allocates three strings to describe
/// somebody it is holding a record of is copying for no reason.
#[derive(Clone, Copy, Debug)]
pub struct Customer<'a> {
    /// Our identifier for them — a user id, as a string.
    ///
    /// Sent to the provider as metadata and handed back on every webhook,
    /// which is what lets a delivery about `sub_123` find the account it
    /// belongs to without a lookup table.
    pub reference: &'a str,
    /// Where a receipt goes.
    pub email: &'a str,
    /// The provider's own identifier for them, if one has been minted.
    ///
    /// `None` the first time somebody pays. Passing the existing one back is
    /// what stops a reader accumulating a customer record per purchase, and
    /// with it a payment history split across several of them.
    pub existing: Option<&'a str>,
}

/// Where to send the browser to finish paying.
///
/// A struct rather than a bare `String` so that adding what a provider returns
/// alongside the url — a session id worth logging, an expiry — does not change
/// the signature of every method that hands one back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handoff {
    /// The provider's hosted page. Redirect to it; do not render it.
    pub url: String,
}
