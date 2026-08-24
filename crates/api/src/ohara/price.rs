//! What a book costs.

use serde::Deserialize;

/// A book's list price, before any discount.
///
/// Minor units, as both `books.price` and Stripe store it — 2900 is $29.00.
/// Keeping the currency beside the amount means a number is never read against
/// the wrong one, which is the failure that turns $29 into ¥29.
///
/// This is not a Stripe Price object and no id is kept for one. Stripe prices
/// are immutable in amount, so an id here would make every price change a
/// Stripe login *and* a commit; the amount is sent inline at checkout instead,
/// and changing it is a commit alone.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Price {
    /// Minor units. Zero is free, and free is a real answer — not every book
    /// is sold.
    pub(crate) amount: i32,
    pub(crate) currency: Currency,
}

impl Price {
    pub(crate) const fn is_free(self) -> bool {
        self.amount == 0
    }
}

/// Only what is actually charged in.
///
/// An enum and not a `String` so a typo is a parse error naming the file rather
/// than a checkout session Stripe rejects. Charging happens in USD everywhere —
/// purchasing power is handled by discounting the amount, not by switching
/// currency, which would need a price per country per book.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Currency {
    #[default]
    Usd,
}

impl Default for Price {
    /// Free, in dollars. A book yaml with no `price` block is a free book.
    fn default() -> Self {
        Self {
            amount: 0,
            currency: Currency::Usd,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_price_parses_from_minor_units() {
        let price: Price =
            serde_norway::from_str("amount: 2900\ncurrency: usd\n").unwrap();

        assert_eq!(price.amount, 2900);
        assert_eq!(price.currency, Currency::Usd);
        assert!(!price.is_free());
    }

    #[test]
    fn zero_is_free() {
        assert!(Price::default().is_free());
    }

    #[test]
    fn an_unsupported_currency_is_refused_rather_than_silently_taken() {
        let parsed: Result<Price, _> =
            serde_norway::from_str("amount: 2900\ncurrency: gbp\n");

        assert!(parsed.is_err());
    }
}
