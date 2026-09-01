//! What is being sold, and how often it is charged for.
//!
//! A plan is named here and priced at the provider. The amount is deliberately
//! not in this crate: plans and their provider handles are configuration, read
//! at boot, so changing what something costs is a config change and never a
//! release of this code.

use serde::Deserialize;

/// A plan's name, in our vocabulary rather than the provider's.
///
/// `voyage_yearly`, not `price_1QxAbC…`. It is what a `memberships` row
/// stores, what a reader's request names, and what survives moving to another
/// provider — a Stripe price id survives none of that.
///
/// A newtype rather than a bare `String` because it gets passed beside the
/// provider's own handle constantly, and two `String` parameters in a row is a
/// call nobody can read and the compiler cannot check.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct PlanId(String);

impl PlanId {
    /// Borrow the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PlanId {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for PlanId {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// How often a plan is charged for.
///
/// [`Interval::Once`] is here because lifetime access is sold as a plan
/// alongside the recurring two, and leaving it out would push "is this one
/// actually a subscription" into a string comparison at every call site.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Interval {
    /// Charged every month.
    Month,
    /// Charged every year.
    Year,
    /// Charged once, and never again.
    Once,
}

impl Interval {
    /// Whether this plan renews on its own.
    ///
    /// A plan that does not is one nothing can cancel, resume or swap, and
    /// those three refusals are better decided here than discovered from the
    /// provider's error message.
    #[must_use]
    pub const fn recurs(self) -> bool {
        matches!(self, Self::Month | Self::Year)
    }
}

/// One thing a reader can subscribe to.
#[derive(Clone, Debug, Deserialize)]
pub struct Plan {
    /// Our name for it.
    pub id: PlanId,
    /// The provider's handle for what to charge — a Stripe price id today.
    ///
    /// Opaque here. This crate never parses it, compares it to anything, or
    /// derives an amount from it; it copies it into a request. That is what
    /// lets the amount live at the provider, where changing it is a dashboard
    /// edit rather than a deploy.
    pub price: String,
    /// How often it is charged for.
    pub interval: Interval,
}
