//! What a reader sends, and whether it is acceptable.

use serde::Deserialize;

/// Which plan to buy.
#[derive(Debug, Deserialize)]
pub(crate) struct ChosenPlan {
    /// Our name for it — `voyage_yearly`. Never a provider's price handle: a
    /// caller that could name one could name any price, including a cheaper
    /// one from a different product.
    pub(crate) plan: String,
}
