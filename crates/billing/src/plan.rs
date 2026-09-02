//! What is being sold, and how often it is charged for.
//!
//! A plan is named here and priced at the provider. The amount is deliberately
//! not in this crate: plans and their provider handles are configuration, read
//! at boot, so changing what something costs is a config change and never a
//! release of this code.

use serde::Deserialize;

use crate::error::Error;

/// A plan's name, in our vocabulary rather than the provider's.
///
/// `voyage_yearly`, not `price_1QxAbC…`. It is what the application stores
/// against an account, what a request names, and what survives moving to
/// another provider — a provider's own price handle survives none of that.
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
///
/// `deny_unknown_fields` because this is configuration a person edits by hand:
/// a mistyped key that parses to a default is a plan silently on sale at the
/// wrong terms, and a startup failure naming the key is cheaper than finding
/// that out from an invoice.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// Our name for it.
    pub id: PlanId,
    /// The provider's handle for what to charge — a Stripe price id today.
    ///
    /// Opaque here. This crate never parses it or derives an amount from it;
    /// it copies it into a request. It is generated rather than hand-written:
    /// see `lighthouse-prices`, which reconciles the declared amounts against
    /// the provider and writes the handles it gets back.
    pub price: String,
    /// How often it is charged for.
    pub interval: Interval,
    /// What it costs, for anything that has to show a price.
    ///
    /// Optional because a plan is sellable without it — checkout sends the
    /// handle and the provider charges what it holds. A page that cannot find
    /// an amount shows no amount, which is better than showing a wrong one.
    #[serde(default)]
    pub money: Option<Money>,
}

/// What a plan costs.
///
/// Declared, not read back. The declaration is the truth and the provider is
/// made to agree with it — which means a page can say what something costs
/// without a round trip, and the number it says is the number that was
/// deliberately chosen rather than whatever a dashboard currently holds.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Money {
    /// Minor units — 4900 is $49.00. The unit every provider stores.
    pub amount: i64,
    /// ISO 4217, lowercase, as providers write it.
    pub currency: String,
}

/// Every plan this deployment sells.
///
/// Read from configuration at boot rather than compiled in. What is on sale
/// changes far more often than the code that sells it, and a price handle in a
/// source file makes adding a plan a release.
///
/// Note what is *not* here: which content a plan unlocks. That mapping is the
/// application's, not this crate's — a plan is a thing charged for, and what
/// being charged for it entitles somebody to is a question only the
/// application can answer.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plans {
    plans: Vec<Plan>,
}

impl Plans {
    /// Read plans from a yaml document.
    ///
    /// # Errors
    ///
    /// [`Error::Malformed`] if the document does not parse, lists no plans,
    /// uses one name twice, or gives a plan no provider handle. All four are
    /// worth failing at boot: the alternative is a checkout that breaks for
    /// one reader on one plan, weeks later.
    pub fn from_yaml(document: &str) -> Result<Self, Error> {
        let plans: Self =
            serde_norway::from_str(document).map_err(|cause| {
                Error::Malformed {
                    what: "the plan configuration",
                    cause: cause.to_string(),
                }
            })?;

        plans.validate()?;

        Ok(plans)
    }

    /// The plan under this name, if it is on sale.
    ///
    /// `None` for a name that is not configured, which is a reader asking for
    /// something that does not exist rather than a fault.
    #[must_use]
    pub fn get(&self, id: &PlanId) -> Option<&Plan> {
        self.plans.iter().find(|plan| &plan.id == id)
    }

    /// Every plan, in the order configured.
    pub fn all(&self) -> impl Iterator<Item = &Plan> {
        self.plans.iter()
    }

    /// Check the configuration is sellable.
    ///
    /// At boot, so a typo is a named startup failure rather than a checkout
    /// that fails for one reader on one plan, weeks later.
    ///
    /// # Errors
    ///
    /// [`Error::Malformed`] if there are no plans, if a name is used twice —
    /// which would make [`Plans::get`] silently pick the first — or if a plan
    /// carries no provider handle, which cannot be charged for.
    fn validate(&self) -> Result<(), Error> {
        let malformed = |cause: String| Error::Malformed {
            what: "the plan configuration",
            cause,
        };

        if self.plans.is_empty() {
            return Err(malformed("it lists no plans".to_owned()));
        }

        for (position, plan) in self.plans.iter().enumerate() {
            if plan.price.trim().is_empty() {
                return Err(malformed(format!(
                    "{} has no price handle",
                    plan.id
                )));
            }

            if self
                .plans
                .iter()
                .take(position)
                .any(|earlier| earlier.id == plan.id)
            {
                return Err(malformed(format!("{} is listed twice", plan.id)));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "
plans:
  - id: monthly
    price: price_monthly
    interval: month
  - id: yearly
    price: price_yearly
    interval: year
  - id: lifetime
    price: price_lifetime
    interval: once
";

    #[test]
    fn a_plan_is_found_by_our_name_and_carries_the_providers_handle() {
        let plans = Plans::from_yaml(SAMPLE).unwrap();
        let yearly = plans.get(&"yearly".into()).unwrap();

        assert_eq!(yearly.price, "price_yearly");
        assert_eq!(yearly.interval, Interval::Year);
        assert!(plans.get(&"weekly".into()).is_none());
    }

    #[test]
    fn only_the_repeating_intervals_recur() {
        let plans = Plans::from_yaml(SAMPLE).unwrap();

        assert!(plans.get(&"monthly".into()).unwrap().interval.recurs());
        assert!(plans.get(&"yearly".into()).unwrap().interval.recurs());
        assert!(!plans.get(&"lifetime".into()).unwrap().interval.recurs());
    }

    #[test]
    fn a_name_listed_twice_is_a_startup_failure() {
        // Without this, `get` returns whichever came first and the second
        // entry is a price nobody is ever charged.
        let document = "
plans:
  - id: yearly
    price: price_one
    interval: year
  - id: yearly
    price: price_two
    interval: year
";

        assert!(Plans::from_yaml(document).is_err());
    }

    #[test]
    fn a_plan_with_nothing_to_charge_against_is_refused() {
        let document = "
plans:
  - id: yearly
    price: '  '
    interval: year
";

        assert!(Plans::from_yaml(document).is_err());
    }

    #[test]
    fn an_unknown_interval_is_a_parse_error_rather_than_a_default() {
        let document = "
plans:
  - id: fortnightly
    price: price_x
    interval: fortnight
";

        assert!(Plans::from_yaml(document).is_err());
    }

    #[test]
    fn selling_nothing_is_a_misconfiguration() {
        assert!(Plans::from_yaml("plans: []").is_err());
    }
}
