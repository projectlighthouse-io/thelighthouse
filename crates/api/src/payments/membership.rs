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
    /// Bought outright: no end, and none expected.
    ///
    /// The only thing that makes a null `period_ends_at` mean forever. Without
    /// it null means "we were not told", which a manual scholarship row has
    /// today — and which must not read as lifetime access.
    pub(crate) lifetime: bool,
}

impl Membership {
    /// Whether this membership grants what it was bought for.
    ///
    /// `GRACE` deliberately does not. Only `active` counted in the laravel app
    /// and only `ACTIVE` counts here: keeping access through a dunning cycle
    /// is a decision to make on purpose, not one to inherit from a match arm
    /// that grouped two statuses together.
    ///
    /// **The date is checked, not just the status.** It used to be the status
    /// alone, which is right only while something keeps the two in step — a
    /// provider webhook flipping `status` the moment the period lapses. A
    /// webhook that never arrives, or arrives and fails, left a row that said
    /// `active` with a `period_ends_at` in the past, and the reader kept the
    /// whole track indefinitely. The row knew; nothing asked it.
    ///
    /// **Null is not forever.** A membership with no end date grants nothing
    /// unless it says `lifetime`. Null has meant "we were not told when this
    /// period ends" since the table was written, and treating it as unlimited
    /// would hand a manual scholarship row permanent access by accident.
    ///
    /// **A cancellation can land before the period does.** Whichever comes
    /// first is the end — see [`ends_at`](Self::ends_at).
    pub(crate) fn grants_access(&self) -> bool {
        self.grants_access_at(chrono::Utc::now().naive_utc())
    }

    /// When this membership stops granting, if it ever does.
    ///
    /// The earlier of the period and a pending cancellation. Usually they are
    /// the same instant — cancelling at period end sets `cancel_at` to exactly
    /// that — but a cancellation can be scheduled for sooner, and then it is
    /// the one that decides. Taking `period_ends_at` alone would keep a reader
    /// reading for the whole period they were cancelled out of.
    ///
    /// `None` for a lifetime membership, which is the whole point of the flag.
    pub(crate) fn ends_at(&self) -> Option<NaiveDateTime> {
        if self.lifetime {
            return None;
        }

        match (self.period_ends_at, self.cancel_at) {
            (Some(period), Some(cancel)) => Some(period.min(cancel)),
            // A missing period is not "forever" — `lifetime` above is the only
            // thing that says that — so a cancellation alone still ends it,
            // and neither leaves nothing to grant from.
            (Some(one), None) | (None, Some(one)) => Some(one),
            (None, None) => Some(NaiveDateTime::MIN),
        }
    }

    /// The same question at a stated moment, which is what makes it testable.
    ///
    /// Naive UTC throughout: the columns are `TIMESTAMP(0)` without a zone and
    /// are written as UTC, so comparing against `Utc::now().naive_utc()` keeps
    /// both sides in the same frame. A local `now` here would be an offset's
    /// worth of free or stolen access, depending which way the server leans.
    pub(crate) fn grants_access_at(&self, now: NaiveDateTime) -> bool {
        if self.status != ACTIVE {
            return false;
        }

        self.ends_at().is_none_or(|ends| ends > now)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
            .expect("a test timestamp")
    }

    fn membership(
        status: i16,
        period_ends_at: Option<&str>,
        lifetime: bool,
    ) -> Membership {
        Membership {
            plan: "go_yearly".to_owned(),
            status,
            provider: "stripe".to_owned(),
            provider_ref: Some("sub_1".to_owned()),
            period_ends_at: period_ends_at.map(at),
            cancel_at: None,
            lifetime,
        }
    }

    const NOW: &str = "2026-09-13 12:00:00";

    #[test]
    fn a_paid_up_membership_inside_its_period_grants_access() {
        let live = membership(ACTIVE, Some("2027-09-02 16:16:58"), false);

        assert!(live.grants_access_at(at(NOW)));
    }

    #[test]
    fn a_period_that_has_already_ended_grants_nothing() {
        // The gap this rule was written for. The status alone said `active`,
        // and it is only ever kept in step by a provider webhook arriving — so
        // one that did not left the reader holding the track indefinitely.
        let lapsed = membership(ACTIVE, Some("2026-09-01 00:00:00"), false);

        assert!(!lapsed.grants_access_at(at(NOW)));
    }

    #[test]
    fn no_end_date_is_not_forever() {
        // Null has meant "we were not told when this period ends" since the
        // table was written, and a manual scholarship row has one. Reading it
        // as unlimited would grant permanent access by accident.
        let undated = membership(ACTIVE, None, false);

        assert!(!undated.grants_access_at(at(NOW)));
    }

    #[test]
    fn a_lifetime_membership_never_lapses() {
        let forever = membership(ACTIVE, None, true);

        assert!(forever.grants_access_at(at(NOW)));
        // Far enough out that any date arithmetic would have expired it.
        assert!(forever.grants_access_at(at("2099-01-01 00:00:00")));
    }

    #[test]
    fn a_cancellation_before_the_period_ends_is_what_decides() {
        // What `revoke --date` writes. Taking the period alone would keep the
        // reader reading for the whole period they were cancelled out of.
        let mut cut_short =
            membership(ACTIVE, Some("2027-09-02 16:16:58"), false);
        cut_short.cancel_at = Some(at("2026-09-20 00:00:00"));

        assert!(cut_short.grants_access_at(at(NOW)));
        assert!(!cut_short.grants_access_at(at("2026-10-01 00:00:00")));
    }

    #[test]
    fn a_cancellation_after_the_period_does_not_extend_it() {
        // The earlier of the two, in both directions — a date set beyond the
        // paid period must not buy the reader time they did not pay for.
        let mut odd = membership(ACTIVE, Some("2026-09-20 00:00:00"), false);
        odd.cancel_at = Some(at("2099-01-01 00:00:00"));

        assert!(!odd.grants_access_at(at("2026-10-01 00:00:00")));
    }

    #[test]
    fn a_cancellation_with_no_period_still_ends_it() {
        let mut undated = membership(ACTIVE, None, false);
        undated.cancel_at = Some(at("2026-09-20 00:00:00"));

        assert!(undated.grants_access_at(at(NOW)));
        assert!(!undated.grants_access_at(at("2026-10-01 00:00:00")));
    }

    #[test]
    fn a_lifetime_membership_has_no_end_to_report() {
        assert_eq!(membership(ACTIVE, None, true).ends_at(), None);
    }

    #[test]
    fn only_active_counts_however_long_is_left() {
        // Grace is a payment the provider is still retrying. It deliberately
        // does not grant, and having time left on the clock does not change
        // that — otherwise the dunning decision would be made by the date.
        for status in [GRACE, ENDED] {
            let held = membership(status, Some("2027-09-02 16:16:58"), false);
            assert!(!held.grants_access_at(at(NOW)), "status {status} granted");

            let forever = membership(status, None, true);
            assert!(
                !forever.grants_access_at(at(NOW)),
                "status {status} granted for life"
            );
        }
    }

    #[test]
    fn the_moment_the_period_ends_is_over() {
        // `>` not `>=`: the period ends *at* that instant, and a reader whose
        // clock lands exactly on it has had what they paid for.
        let ending = membership(ACTIVE, Some(NOW), false);

        assert!(!ending.grants_access_at(at(NOW)));
    }
}
