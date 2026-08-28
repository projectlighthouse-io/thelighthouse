//! Why a bookmark was refused, and what the reader is told.
//!
//! Its own enum rather than `notes::Refusal` with two variants added: the codes
//! are a contract with the frontend, and one list covering two endpoints means
//! a client branching on codes that can never arrive on the route it called.

use axum::response::Response;

use crate::response::bad_request;

/// Why a bookmark was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    EmptySelection,
    SelectionTooLong,
    BadAnchor,
}

impl Refusal {
    /// Stable, never translated, and safe to branch on.
    const fn code(self) -> &'static str {
        match self {
            Self::EmptySelection => "selection_empty",
            Self::SelectionTooLong => "selection_too_long",
            Self::BadAnchor => "anchor_invalid",
        }
    }

    /// A fallback, not the final wording — see `notes::refusal`.
    ///
    /// `SelectionTooLong` repeats `payload::MAX_SELECTION` in words because a
    /// `const fn` cannot interpolate; change the two together.
    const fn message(self) -> &'static str {
        match self {
            Self::EmptySelection => "Please select a passage to bookmark.",
            Self::SelectionTooLong => {
                "That passage is too long to bookmark. Please select less \
                 than 500 characters."
            }
            Self::BadAnchor => {
                "That selection could not be anchored. Please try again."
            }
        }
    }
}

/// The 400 a caller sees.
pub(crate) fn refuse(cause_of: Refusal) -> Response {
    bad_request(cause_of.code(), cause_of.message())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant, so adding one fails here first — the prompt to give the
    /// frontend an entry for its code.
    const REFUSALS: [Refusal; 3] = [
        Refusal::EmptySelection,
        Refusal::SelectionTooLong,
        Refusal::BadAnchor,
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

    /// `anchor_invalid` is deliberately the same code notes use for the same
    /// mistake, so a frontend needs one entry rather than two.
    #[test]
    fn a_bad_anchor_says_what_a_bad_note_anchor_says() {
        assert_eq!(Refusal::BadAnchor.code(), "anchor_invalid");
    }
}
