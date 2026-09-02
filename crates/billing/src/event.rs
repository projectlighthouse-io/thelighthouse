//! What a provider told us happened.

use crate::subscription::Subscription;

/// A webhook delivery, once it has been verified and understood.
///
/// Providers each have their own event vocabulary and each send far more of it
/// than any application acts on. A driver maps its own onto this, and every
/// event it does not map becomes [`Event::Ignored`] rather than an error —
/// answering an unhandled delivery with a failure earns retries of it forever.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Money moved and a subscription now exists.
    Started(Subscription),
    /// An existing subscription changed: renewed, cancelled, swapped, or
    /// failed a payment.
    Changed(Subscription),
    /// It is over and will not renew.
    Ended(Subscription),
    /// A reader finished the hosted checkout.
    ///
    /// Separate from [`Event::Started`] because it is the only delivery that
    /// carries our own reference back, and so the only one that can attach a
    /// provider's customer to an account.
    CheckoutCompleted {
        /// The provider's customer identifier.
        customer: String,
        /// Our reference, as it was sent into checkout.
        reference: String,
        /// What was bought, as it was sent into checkout.
        ///
        /// The only place a one-time purchase says what it was for: there is
        /// no subscription afterwards to read it back off, so if this is lost
        /// the payment cannot be turned into anything.
        plan: Option<crate::plan::PlanId>,
        /// The subscription it created, when it created one.
        ///
        /// `None` for a one-time purchase, which is what tells the two apart.
        subscription: Option<String>,
    },
    /// A charge failed. The provider is likely retrying on its own schedule.
    PaymentFailed {
        /// The provider's customer identifier.
        customer: String,
    },
    /// A delivery this driver has nothing to say about.
    ///
    /// Verified and well-formed — it just is not one of ours. The caller
    /// answers 200 and does no work.
    Ignored,
}
