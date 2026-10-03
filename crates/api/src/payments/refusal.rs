//! Why a billing request was refused, and what the reader is told.

use axum::{http::StatusCode, response::Response};

use crate::{cache::CachePolicy, response};

/// Why a billing request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// A plan name that is not on sale.
    UnknownPlan,
    /// Already paying for something. Changing plan happens on the provider's
    /// page, not in a second checkout — two live subscriptions is two charges
    /// a month.
    AlreadySubscribed,
    /// The provider has no customer for this reader — they have never paid
    /// for anything, so there is no billing to manage.
    NoCustomer,
    /// The provider could not be reached, or broke.
    Unavailable,
}

impl Refusal {
    /// Stable, never translated, and safe to branch on.
    const fn code(self) -> &'static str {
        match self {
            Self::UnknownPlan => "unknown_plan",
            Self::AlreadySubscribed => "already_subscribed",
            Self::NoCustomer => "no_customer",
            Self::Unavailable => "provider_unavailable",
        }
    }

    const fn status(self) -> StatusCode {
        match self {
            // The name parsed fine and simply names nothing on sale, which is
            // the distinction 422 draws against 400.
            Self::UnknownPlan => StatusCode::UNPROCESSABLE_ENTITY,
            Self::AlreadySubscribed => StatusCode::CONFLICT,
            Self::NoCustomer => StatusCode::NOT_FOUND,
            Self::Unavailable => StatusCode::BAD_GATEWAY,
        }
    }

    /// A fallback, not the final wording — the frontend keys off the code.
    const fn message(self) -> &'static str {
        match self {
            Self::UnknownPlan => "That plan is not for sale.",
            Self::AlreadySubscribed => {
                "You are already subscribed. Change your plan under manage \
                 billing instead of starting a second subscription."
            }
            Self::NoCustomer => {
                "You have not bought anything yet, so there is no billing to \
                 manage."
            }
            Self::Unavailable => {
                "The payment provider could not be reached. Please try again."
            }
        }
    }
}

/// The refusal a caller sees.
pub(crate) fn refuse(cause_of: Refusal) -> Response {
    response::json(
        cause_of.status(),
        serde_json::json!({
            "code": cause_of.code(),
            "error": cause_of.message(),
        }),
        // Money-dependent, and wrong the moment it changes.
        CachePolicy::NoStore,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant, so adding one fails here first — the prompt to give the
    /// frontend an entry for its code.
    const REFUSALS: [Refusal; 4] = [
        Refusal::UnknownPlan,
        Refusal::AlreadySubscribed,
        Refusal::NoCustomer,
        Refusal::Unavailable,
    ];

    #[test]
    fn every_refusal_has_a_distinct_code() {
        let mut codes: Vec<&str> = REFUSALS.iter().map(|r| r.code()).collect();
        codes.sort_unstable();
        let total = codes.len();
        codes.dedup();

        assert_eq!(codes.len(), total, "two refusals share a code");
    }

    #[test]
    fn a_code_is_stable_wire_text_and_a_message_is_prose() {
        for refusal in REFUSALS {
            let code = refusal.code();

            assert!(!code.is_empty());
            assert!(
                code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{code} is not a stable identifier"
            );
            assert!(refusal.message().ends_with('.'), "{code}");
        }
    }

    #[test]
    fn a_refusal_is_the_callers_fault_or_the_providers_and_says_which() {
        for refusal in REFUSALS {
            let status = refusal.status();

            assert!(
                status.is_client_error() || status == StatusCode::BAD_GATEWAY,
                "{} answers {status}",
                refusal.code()
            );
        }
    }
}
