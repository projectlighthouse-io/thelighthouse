//! What starting a payment takes, and what it gives back.

use crate::plan::PlanId;

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

/// Where the provider sends the browser when checkout finishes.
///
/// Given per checkout rather than configured once on the driver, because the
/// caller usually wants to say something on the way back — which plan was
/// bought, where the reader was before they were sent off to pay. These are
/// the caller's urls and the caller's query string; nothing here reads them.
///
/// **Whatever is put in them comes back through the reader's browser**, so it
/// is visible and editable by them. Fine for deciding what a page says, never
/// for deciding what it may show: entitlement is a question for the api, and
/// the provider's own webhook is what settles that.
#[derive(Clone, Debug)]
pub struct Returns {
    /// Where a reader who paid ends up.
    pub success: String,
    /// Where a reader who backed out ends up.
    pub cancel: String,
}

/// What a finished checkout was for, read back from the provider.
///
/// The provider is asked rather than the browser trusted. A reader returning
/// from checkout carries only a session id, and everything shown to them is
/// looked up from that — so a reader editing the url gets somebody else's id
/// refused, not somebody else's purchase displayed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bought {
    /// The plan bought, from the metadata set when the session was created.
    pub plan: Option<PlanId>,
    /// The application's own reference for whose checkout this was.
    ///
    /// The caller compares it against the reader asking. Without that check
    /// this endpoint would read out any session whose id somebody guessed.
    pub reference: Option<String>,
    /// Whether the money actually moved.
    ///
    /// A session can be complete and unpaid — an async payment method still
    /// clearing. Saying "you now have access" on the strength of a redirect
    /// alone would be saying it before it is true.
    pub paid: bool,
    /// The subscription it created, for a recurring plan.
    pub subscription: Option<String>,
}
