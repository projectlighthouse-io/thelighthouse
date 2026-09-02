//! A membership, as a row and as json.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

use crate::response;

/// Paid up. The only status that grants anything.
pub(crate) const ACTIVE: i16 = 0;
/// A payment failed and the provider is retrying.
pub(crate) const GRACE: i16 = 1;
/// Over, however it ended.
pub(crate) const ENDED: i16 = 2;

/// One reader's subscription, as `memberships` stores it.
#[derive(Debug, FromRow, Serialize)]
pub(crate) struct Membership {
    /// Our name for what was bought, never the provider's price handle.
    pub(crate) plan: String,
    #[serde(serialize_with = "as_name")]
    pub(crate) status: i16,
    /// Which provider took the money. `manual` for a membership nobody paid
    /// for — a scholarship — which is a row here and never a payment.
    pub(crate) provider: String,
    /// The provider's own id for it. Absent for the manual case, where there
    /// was no transaction to name.
    #[serde(skip)]
    pub(crate) provider_ref: Option<String>,
    /// The end of the period already paid for. What a reader keeps after
    /// cancelling, which is why cancelling is not ending.
    #[serde(serialize_with = "response::as_utc")]
    pub(crate) period_ends_at: Option<NaiveDateTime>,
    /// When a requested cancellation takes effect, if one is pending.
    ///
    /// Set means cancelled but not yet over, which is the state the reader is
    /// shown and the only one `resume` can undo.
    #[serde(serialize_with = "response::as_utc")]
    pub(crate) cancel_at: Option<NaiveDateTime>,
}

impl Membership {
    /// Whether this membership grants what it was bought for.
    ///
    /// `GRACE` deliberately does not. Only `active` counted in the laravel app
    /// and only `ACTIVE` counts here: keeping access through a dunning cycle
    /// is a decision to make on purpose, not one to inherit from a match arm
    /// that grouped two statuses together.
    pub(crate) const fn grants_access(&self) -> bool {
        self.status == ACTIVE
    }

    /// Whether a cancellation is pending but has not taken effect.
    ///
    /// The state `resume` exists to undo. A membership with no pending
    /// cancellation has nothing to resume, and saying so is better than asking
    /// the provider and relaying its complaint.
    pub(crate) const fn is_cancelling(&self) -> bool {
        self.cancel_at.is_some()
    }
}

/// The wire spelling of a status, so the frontend branches on a word rather
/// than on `1`.
/// `&i16` rather than `i16` to match serde's `serialize_with`, which is the
/// same reason `response::as_utc` takes `&Option<_>`.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn as_name<S: serde::Serializer>(
    status: &i16,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(match *status {
        ACTIVE => "active",
        GRACE => "grace",
        _ => "ended",
    })
}
