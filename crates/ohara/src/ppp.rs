//! Purchasing power parity: who pays less, and how much less.

use std::collections::HashSet;

use serde::Deserialize;

use super::{Content, Error, price::Price, read};

/// `pricing/ppp.yaml`.
///
/// One file for every book. The discount is a property of where the buyer is,
/// never of what they are buying, so no book will ever want a rate of its own —
/// and a rate kept per book would be the same number copied once per title,
/// with one of the copies eventually stale.
#[derive(Debug, Deserialize)]
pub struct Ppp {
    /// Minor units. No sale lands below this: under roughly $2 the card fee is
    /// most of the charge, and Stripe refuses under $0.50 outright.
    pub floor: i32,
    pub tiers: Vec<Tier>,
}

/// A discount, and everyone who gets it.
///
/// One tier is one Stripe coupon — which is why the rate is grouped with its
/// countries rather than listed against each. `percent_off` is Stripe's field
/// name in Stripe's units, so creating the coupon copies rather than converts.
#[derive(Debug, Deserialize)]
pub struct Tier {
    pub percent_off: u8,
    /// ISO 3166-1 alpha-2. Compared case-insensitively, because Cloudflare
    /// sends `BD` and the file reads better as `bd`.
    pub countries: Vec<String>,
}

/// What to actually charge, and whether Stripe can show it as a discount.
///
/// Two fields and not one because the floor and the coupon cannot both be in
/// charge: Stripe computes a coupon's discount itself and has no notion of a
/// minimum, so once the floor binds, the amount has to be sent as the price
/// instead. The buyer then pays the floor without seeing a discount line —
/// correct, and rare enough to be worth the plainness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Charge {
    /// Minor units to send as the checkout amount.
    pub amount: i32,
    /// The coupon to apply on top, or `None` when `amount` is already final.
    pub percent_off: Option<u8>,
}

impl Ppp {
    /// The tier a country falls in, if any.
    ///
    /// Cloudflare's `XX` (unknown) and `T1` (Tor) match nothing and so pay full
    /// price, without either needing to be named here.
    #[must_use]
    pub fn tier_for(&self, country: &str) -> Option<&Tier> {
        self.tiers.iter().find(|tier| {
            tier.countries
                .iter()
                .any(|listed| listed.eq_ignore_ascii_case(country))
        })
    }

    /// What a book costs someone in `country`.
    #[must_use]
    pub fn charge(&self, price: Price, country: &str) -> Charge {
        let full = Charge {
            amount: price.amount,
            percent_off: None,
        };

        // A book that is already free, or already at or under the floor, has
        // nothing to discount. Without this the floor would *raise* the price
        // of a cheap book, which is the opposite of the point.
        if price.is_free() || price.amount <= self.floor {
            return full;
        }

        let Some(tier) = self.tier_for(country) else {
            return full;
        };

        let discounted = tier.applied_to(price.amount);

        if discounted < self.floor {
            return Charge {
                amount: self.floor,
                percent_off: None,
            };
        }

        Charge {
            amount: discounted,
            percent_off: Some(tier.percent_off),
        }
    }

    /// Rejects a file that would price something wrongly.
    fn validate(&self) -> Result<(), String> {
        if self.floor < 0 {
            return Err(format!("floor is {}, which is negative", self.floor));
        }

        let mut seen = HashSet::new();

        for tier in &self.tiers {
            if tier.percent_off == 0 || tier.percent_off > 100 {
                return Err(format!(
                    "percent_off is {}, and stripe takes 1 to 100",
                    tier.percent_off
                ));
            }

            for country in &tier.countries {
                // A country in two tiers gets whichever is listed first, which
                // is a silent answer to a question the file is asking twice.
                if !seen.insert(country.to_ascii_lowercase()) {
                    return Err(format!("{country} is in more than one tier"));
                }
            }
        }

        Ok(())
    }
}

impl Tier {
    /// The discounted amount, truncated to the minor unit — so a fraction of a
    /// cent goes to the buyer.
    ///
    /// Only decides whether the floor binds, and what to show. When a coupon is
    /// applied Stripe computes the charge itself, and the two may differ by a
    /// minor unit.
    // Integer division is the point: money is minor units, and clippy's advice
    // to reach for floats would introduce the rounding this avoids.
    #[allow(clippy::integer_division)]
    fn applied_to(&self, amount: i32) -> i32 {
        let amount = i64::from(amount);
        let kept = 100 - i64::from(self.percent_off);

        // i64 throughout: an i32 amount times 100 overflows above $21m, and a
        // wrapped multiply here would be a free book.
        i32::try_from(amount * kept / 100).unwrap_or(i32::MAX)
    }
}

impl Content {
    /// Reads and parses `pricing/ppp.yaml`.
    ///
    /// # Errors
    ///
    /// The file being absent or unparseable, or describing a discount that
    /// Stripe would refuse or that two tiers both claim.
    pub fn ppp(&self) -> Result<Ppp, Error> {
        let path = self.root.join("pricing").join("ppp.yaml");
        let raw = read(&path)?;

        let ppp: Ppp =
            serde_norway::from_str(&raw).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause: cause.to_string(),
            })?;

        ppp.validate()
            .map_err(|cause| Error::Malformed { path, cause })?;

        Ok(ppp)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{fixture, price::Currency};
    use super::*;

    fn ppp() -> Ppp {
        fixture::content().ppp().unwrap()
    }

    fn priced(amount: i32) -> Price {
        Price {
            amount,
            currency: Currency::Usd,
        }
    }

    #[test]
    fn a_listed_country_is_discounted_and_names_its_coupon() {
        let charge = ppp().charge(priced(2900), "bd");

        assert_eq!(charge.amount, 870);
        assert_eq!(charge.percent_off, Some(70));
    }

    #[test]
    fn cloudflare_sends_uppercase_and_it_still_matches() {
        assert_eq!(ppp().charge(priced(2900), "BD").percent_off, Some(70));
    }

    #[test]
    fn an_unlisted_country_pays_full_price() {
        let charge = ppp().charge(priced(2900), "us");

        assert_eq!(charge.amount, 2900);
        assert_eq!(charge.percent_off, None);
    }

    #[test]
    fn unknown_and_tor_pay_full_price_without_being_named() {
        let ppp = ppp();

        assert_eq!(ppp.charge(priced(2900), "XX").amount, 2900);
        assert_eq!(ppp.charge(priced(2900), "T1").amount, 2900);
    }

    #[test]
    fn the_floor_binds_and_drops_the_coupon_with_it() {
        // 70% off $9 is $2.70, under the fixture's $5 floor. Stripe computes a
        // coupon's discount itself and cannot be told a minimum, so the amount
        // has to be sent as the price instead.
        let charge = ppp().charge(priced(900), "bd");

        assert_eq!(charge.amount, 500);
        assert_eq!(charge.percent_off, None);
    }

    #[test]
    fn a_book_already_under_the_floor_is_not_raised_to_it() {
        let charge = ppp().charge(priced(300), "bd");

        assert_eq!(charge.amount, 300);
        assert_eq!(charge.percent_off, None);
    }

    #[test]
    fn a_free_book_stays_free_everywhere() {
        assert_eq!(ppp().charge(priced(0), "bd").amount, 0);
    }

    #[test]
    fn a_country_in_two_tiers_is_refused_rather_than_answered_twice() {
        let ppp: Ppp = serde_norway::from_str(
            "floor: 500\ntiers:\n\
             - percent_off: 70\n  countries: [bd]\n\
             - percent_off: 40\n  countries: [bd]\n",
        )
        .unwrap();

        let cause = ppp.validate().unwrap_err();

        assert!(cause.contains("more than one tier"), "{cause}");
    }

    #[test]
    fn a_percentage_stripe_would_refuse_is_refused_here_first() {
        let ppp: Ppp = serde_norway::from_str(
            "floor: 500\ntiers:\n- percent_off: 0\n  countries: [bd]\n",
        )
        .unwrap();

        assert!(ppp.validate().is_err());
    }
}
