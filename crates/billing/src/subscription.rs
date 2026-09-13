//! A subscription as the provider currently sees it.

use chrono::NaiveDateTime;

use crate::plan::PlanId;

/// What a provider knows about one subscription, in neutral terms.
///
/// Not a database row. The api builds a `memberships` row out of this; the
/// difference matters because the provider is the authority on status and
/// dates, and we are the authority on which reader it belongs to — the
/// provider has never heard of a `user_id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subscription {
    /// The provider's own identifier, kept so we can talk about this
    /// subscription again.
    pub reference: String,
    /// The application's own reference for whose subscription this is, as it
    /// was sent into checkout.
    ///
    /// Here because webhook deliveries have no guaranteed order: the event
    /// announcing a new subscription can arrive before the one announcing the
    /// checkout that created it. Without this, the first delivery names a
    /// subscription with no way to tell which account it belongs to, and the
    /// only recovery is to drop it and hope the other one lands.
    ///
    /// `None` for a subscription created outside this crate — in a dashboard,
    /// or by an older integration.
    pub account: Option<String>,
    /// Which plan it is on, when the provider said.
    ///
    /// `None` when the answer cannot be mapped back to a configured plan —
    /// a price id retired from the config, or a subscription created by hand
    /// in the dashboard. Not an error: the membership is real and the reader
    /// paid, and refusing to model it would lock them out.
    pub plan: Option<PlanId>,
    /// Where it stands.
    pub status: Status,
    /// The end of the period already paid for.
    ///
    /// This is what a reader keeps after cancelling, which is why a
    /// cancellation is not an ending.
    pub period_ends_at: Option<NaiveDateTime>,
    /// When a requested cancellation takes effect, if one is pending.
    pub cancel_at: Option<NaiveDateTime>,
}

/// Where a subscription stands.
///
/// Four states, not the provider's list. Stripe alone has eight, and the
/// differences between `incomplete_expired` and `canceled` are its billing
/// machinery talking to itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Paid up. The reader gets what they bought.
    Active,
    /// A payment failed and the provider is retrying.
    ///
    /// **Access stops here**, matching what the Laravel app did: only `active`
    /// and `trialing` ever counted, so `past_due` locked immediately. Keeping
    /// access through a dunning cycle is a decision somebody can make later,
    /// and it should be made deliberately rather than inherited from a match
    /// arm that grouped this with `Active`.
    PastDue,
    /// Over. Either it ran out or somebody ended it.
    Canceled,
    /// Never started — checkout was abandoned, or the first payment needs an
    /// action nobody took.
    Incomplete,
}

impl Status {
    /// Whether this status is one that grants access.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// When a cancellation should take effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cancel {
    /// At the end of the period already paid for.
    ///
    /// The default a reader means when they click cancel: they paid for the
    /// month and they keep the month.
    AtPeriodEnd,
    /// Immediately, forfeiting the rest of the period.
    ///
    /// For a refund, or an account being closed. Not for a reader who changed
    /// their mind.
    Now,
    /// At a stated moment, whenever that is.
    ///
    /// Neither of the two above: the period already paid for may end later or
    /// sooner than this. What it is for is an end somebody chose — a refund
    /// taking effect at the end of the month, a scholarship running out on a
    /// date agreed with the reader.
    ///
    /// Must be in the future. Stripe refuses a `cancel_at` that has already
    /// passed, and the caller is the one that knows whether "now" was meant —
    /// [`Now`](Self::Now) says that without ambiguity.
    At(chrono::DateTime<chrono::Utc>),
}
