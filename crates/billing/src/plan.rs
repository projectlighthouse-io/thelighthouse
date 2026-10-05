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
    /// What this plan unlocks, by the application's own name for each thing.
    ///
    /// **Carried, never interpreted.** This crate does not know what one of
    /// these is, cannot check one, and never will — see the note on [`Plans`].
    /// It is here because `deny_unknown_fields` means a key this type does not
    /// declare is a startup failure, and the alternative is a second
    /// configuration file saying which plan unlocks what, parsed separately
    /// and able to disagree with this one about which plans exist.
    ///
    /// Named `books` rather than something this crate could honestly claim to
    /// understand, because `pricing.yaml` calls them books and one name for one
    /// thing is worth more here than a boundary kept in the spelling.
    ///
    /// Empty with `everything` false is a plan that unlocks nothing. That is a
    /// real state — a plan declared before its content — and not this crate's
    /// business to refuse.
    #[serde(default)]
    pub books: Vec<String>,
    /// Whether this plan unlocks everything, including things added later.
    ///
    /// Not expressible as a list: the point of it is the things that do not
    /// exist yet. Also carried and not interpreted.
    #[serde(default)]
    pub everything: bool,
    /// The purchasing-power tiers this plan is discounted by.
    ///
    /// Here rather than in a list beside the plans because a coupon is a
    /// billing object restricted to *this* plan's product at stripe, and a
    /// top-level list had to say which plans it touched — two places to write
    /// the same restriction, able to disagree.
    ///
    /// Absent is none, which is what every plan file written before this field
    /// said.
    #[serde(default)]
    pub ppp: Vec<Ppp>,
}

impl Plan {
    /// The tier a country is offered on this plan, if any.
    ///
    /// Compared as given: `lighthouse-prices` refuses a declaration whose codes
    /// are not already uppercase, and the api uppercases what cloudflare sends,
    /// so neither side normalises at request time.
    ///
    /// A tier that names the country wins; otherwise the plan's rest tier, if
    /// it has one — the price for everywhere no other tier names. `None` is a
    /// request with no usable country header, which is somewhere, so it gets
    /// the rest tier too. A country is never claimed by two named tiers of one
    /// plan: the declaration refuses that.
    #[must_use]
    pub fn for_country(&self, country: Option<&str>) -> Option<&Ppp> {
        let named = country.and_then(|country| {
            self.ppp
                .iter()
                .find(|tier| tier.countries.iter().any(|c| c == country))
        });

        named.or_else(|| self.ppp.iter().find(|tier| tier.rest))
    }
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
/// Note what this crate does not *do* with which content a plan unlocks. The
/// lists ride along on [`Plan::includes`] and [`Plan::everything`] because they
/// are declared beside the prices and there is no sense in a second file, but
/// nothing here reads them: a plan is a thing charged for, and what being
/// charged for it entitles somebody to is a question only the application can
/// answer. Every decision made from those fields is made above this crate.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plans {
    plans: Vec<Plan>,
}

/// A discount offered to readers in particular countries — or, as a plan's
/// rest tier, everywhere its other tiers do not name.
///
/// A coupon is a billing object, so it lives here — unlike which books a plan
/// unlocks, which is the application's question. What is *not* here is how the
/// country is decided: this crate is told one, and where it came from is the
/// caller's business.
///
/// Declared inside the plan it discounts. One code belongs to one plan, so
/// there is nothing here saying which plans it covers: the nesting says it,
/// and stripe holds the same restriction as the coupon's `applies_to`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ppp {
    /// What a reader types at checkout, and the coupon's id at stripe.
    code: String,
    /// The promotion code's own id — stripe's answer, not a decision.
    promotion: String,
    /// Percent off. Exactly one of this and `amount_off`.
    #[serde(default)]
    percent: Option<u8>,
    /// Minor units off, in the plan's currency — 2000 is $20. Exactly one of
    /// this and `percent`.
    #[serde(default)]
    amount_off: Option<i64>,
    /// ISO 3166-1 alpha-2, uppercase, as cloudflare reports them. Empty only
    /// on the rest tier.
    #[serde(default)]
    countries: Vec<String>,
    /// The plan's tier for every country its other tiers do not name. At most
    /// one per plan, and it names no countries of its own.
    #[serde(default)]
    rest: bool,
}

/// How much a tier takes off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Off {
    Percent(u8),
    /// Minor units, in the plan's currency.
    Amount(i64),
}

impl Ppp {
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    #[must_use]
    pub fn promotion(&self) -> &str {
        &self.promotion
    }

    /// How much this tier takes off. [`Plans::from_yaml`] refuses a tier that
    /// does not say exactly once, so a loaded tier always has an answer; the
    /// zero percent here is unreachable rather than a default anybody gets.
    #[must_use]
    pub const fn off(&self) -> Off {
        match (self.percent, self.amount_off) {
            (_, Some(amount)) => Off::Amount(amount),
            (Some(percent), None) => Off::Percent(percent),
            (None, None) => Off::Percent(0),
        }
    }

    /// Whether this is the plan's tier for everywhere not named elsewhere.
    #[must_use]
    pub const fn is_rest(&self) -> bool {
        self.rest
    }
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

            for tier in &plan.ppp {
                if tier.percent.is_some() == tier.amount_off.is_some() {
                    return Err(malformed(format!(
                        "{} on {} must give exactly one of percent and amount_off",
                        tier.code, plan.id
                    )));
                }

                if tier.rest != tier.countries.is_empty() {
                    return Err(malformed(format!(
                        "{} on {} must either name countries or be the rest tier",
                        tier.code, plan.id
                    )));
                }
            }

            if plan.ppp.iter().filter(|tier| tier.rest).count() > 1 {
                return Err(malformed(format!(
                    "{} has more than one rest tier",
                    plan.id
                )));
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
    const WITH_TIERS: &str = "
plans:
  - id: yearly
    price: price_x
    interval: year
    money:
      amount: 4900
      currency: usd
    ppp:
      - code: LH-BD
        promotion: promo_x
        percent: 60
        countries: [BD, IN]
";

    #[test]
    fn a_country_finds_the_plans_tier_and_others_find_nothing() {
        let plans = Plans::from_yaml(WITH_TIERS).unwrap();
        let yearly = plans.get(&"yearly".into()).unwrap();

        assert_eq!(
            yearly.for_country(Some("BD")).map(Ppp::off),
            Some(Off::Percent(60))
        );
        assert_eq!(
            yearly.for_country(Some("IN")).map(Ppp::code),
            Some("LH-BD")
        );
        assert!(yearly.for_country(Some("GB")).is_none());
        // Compared as given: both sides are uppercase by the time they meet.
        assert!(yearly.for_country(Some("bd")).is_none());
    }

    const TWO_PLANS: &str = "
plans:
  - id: lifetime
    price: price_x
    interval: once
    ppp:
      - code: LH-50
        promotion: promo_x
        percent: 50
        countries: [BD]
  - id: yearly
    price: price_y
    interval: year
";

    #[test]
    fn a_tier_discounts_only_the_plan_it_is_declared_inside() {
        // What `covers` used to answer. The nesting says it now, so there is
        // nothing to keep in step with the restriction stripe holds.
        let plans = Plans::from_yaml(TWO_PLANS).unwrap();

        assert!(
            plans
                .get(&"lifetime".into())
                .unwrap()
                .for_country(Some("BD"))
                .is_some()
        );
        assert!(
            plans
                .get(&"yearly".into())
                .unwrap()
                .for_country(Some("BD"))
                .is_none()
        );
    }

    #[test]
    fn a_plan_file_with_no_tiers_still_reads() {
        // Every plan file written before tiers existed says this.
        let plans = Plans::from_yaml(SAMPLE).unwrap();

        assert!(
            plans
                .get(&"yearly".into())
                .unwrap()
                .for_country(Some("BD"))
                .is_none()
        );
    }

    const EVERYONE: &str = "
plans:
  - id: yearly
    price: price_x
    interval: year
    money:
      amount: 11900
      currency: usd
    ppp:
      - code: YEARLY-LOWER
        promotion: promo_lower
        amount_off: 7000
        countries: [IN, BD]
      - code: YEARLY-EVERYONE
        promotion: promo_everyone
        amount_off: 2000
        rest: true
";

    #[test]
    fn a_tier_can_take_a_fixed_amount_off() {
        let plans = Plans::from_yaml(EVERYONE).unwrap();
        let yearly = plans.get(&"yearly".into()).unwrap();

        assert_eq!(
            yearly.for_country(Some("IN")).map(Ppp::off),
            Some(Off::Amount(7000))
        );
    }

    #[test]
    fn a_country_no_tier_names_gets_the_rest_tier() {
        let plans = Plans::from_yaml(EVERYONE).unwrap();
        let yearly = plans.get(&"yearly".into()).unwrap();

        assert_eq!(
            yearly.for_country(Some("GB")).map(Ppp::code),
            Some("YEARLY-EVERYONE")
        );
        // A named country is never offered the rest tier instead, whichever
        // order the tiers are listed in.
        assert_eq!(
            yearly.for_country(Some("BD")).map(Ppp::code),
            Some("YEARLY-LOWER")
        );
    }

    #[test]
    fn a_reader_whose_country_is_unknown_gets_the_rest_tier() {
        // No header is "somewhere", and the rest tier is for everywhere no
        // other tier names.
        let plans = Plans::from_yaml(EVERYONE).unwrap();
        let yearly = plans.get(&"yearly".into()).unwrap();

        assert_eq!(
            yearly.for_country(None).map(Ppp::code),
            Some("YEARLY-EVERYONE")
        );
    }

    #[test]
    fn without_a_rest_tier_an_unknown_country_gets_nothing() {
        let plans = Plans::from_yaml(WITH_TIERS).unwrap();

        assert!(
            plans
                .get(&"yearly".into())
                .unwrap()
                .for_country(None)
                .is_none()
        );
    }

    fn tier_file(tier: &str) -> String {
        format!(
            "
plans:
  - id: yearly
    price: price_x
    interval: year
    ppp:
{tier}"
        )
    }

    #[test]
    fn a_tier_must_say_how_much_off_exactly_once() {
        for tier in [
            // Both.
            "      - code: A\n        promotion: p\n        percent: 10\n        amount_off: 100\n        countries: [IN]\n",
            // Neither.
            "      - code: A\n        promotion: p\n        countries: [IN]\n",
        ] {
            assert!(Plans::from_yaml(&tier_file(tier)).is_err(), "{tier}");
        }
    }

    #[test]
    fn a_tier_names_countries_or_is_the_rest_never_both_nor_neither() {
        for tier in [
            "      - code: A\n        promotion: p\n        percent: 10\n        rest: true\n        countries: [IN]\n",
            "      - code: A\n        promotion: p\n        percent: 10\n",
        ] {
            assert!(Plans::from_yaml(&tier_file(tier)).is_err(), "{tier}");
        }
    }

    #[test]
    fn a_plan_has_at_most_one_rest_tier() {
        let two = "      - code: A\n        promotion: p\n        percent: 10\n        rest: true\n      - code: B\n        promotion: q\n        percent: 20\n        rest: true\n";

        assert!(Plans::from_yaml(&tier_file(two)).is_err());
    }
}
