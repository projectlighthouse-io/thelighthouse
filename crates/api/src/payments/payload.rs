//! What a reader sends, and whether it is acceptable.

use serde::Deserialize;

/// Which plan to buy or move to.
#[derive(Debug, Deserialize)]
pub(crate) struct ChosenPlan {
    /// Our name for it — `voyage_yearly`. Never a provider's price handle: a
    /// caller that could name one could name any price, including a cheaper
    /// one from a different product.
    pub(crate) plan: String,
}

/// How a cancellation should take effect.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Cancellation {
    /// Forfeit the rest of the paid period.
    ///
    /// Defaults to false, which is what a reader clicking cancel means: they
    /// paid for the month and they keep the month. Ending it immediately is
    /// for a refund, and is deliberately the option that has to be asked for.
    #[serde(default)]
    pub(crate) immediately: bool,
}
