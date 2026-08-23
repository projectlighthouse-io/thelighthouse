//! Why a write was refused, and what the reader is told.

use axum::response::Response;

use crate::response::bad_request;

/// Why a write was refused.
///
/// An enum, not a table of `&'static str`: a `Result<_, &str>` makes every
/// string literal a valid error, so nothing stops a call site inventing its own
/// wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    EmptyNote,
    NoteTooLong,
    SelectionTooLong,
    BadAnchor,
    NestedReply,
    NoSuchNote,
}

impl Refusal {
    /// Stable, never translated, and safe to branch on. As much a contract as
    /// the field names — changing one breaks any client that reads it.
    const fn code(self) -> &'static str {
        match self {
            Self::EmptyNote => "note_empty",
            Self::NoteTooLong => "note_too_long",
            Self::SelectionTooLong => "selection_too_long",
            Self::BadAnchor => "anchor_invalid",
            Self::NestedReply => "reply_not_top_level",
            Self::NoSuchNote => "note_not_found",
        }
    }

    /// A fallback, not the final wording: `docs/rebuild.md` puts UI strings in
    /// the frontend's own typed table, keyed by the code above.
    ///
    /// `NoteTooLong` repeats `payload::MAX_NOTE` in words because a `const fn`
    /// cannot interpolate; change the two together.
    const fn message(self) -> &'static str {
        match self {
            Self::EmptyNote => "Please enter a note.",
            Self::NoteTooLong => "Your note must not exceed 500 characters.",
            Self::SelectionTooLong => {
                "That selection is too long. Please select a shorter passage."
            }
            Self::BadAnchor => "That selection could not be anchored. Please try again.",
            Self::NestedReply => "You can only reply to top-level notes.",
            Self::NoSuchNote => "That note could not be found.",
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
    const REFUSALS: [Refusal; 6] = [
        Refusal::EmptyNote,
        Refusal::NoteTooLong,
        Refusal::SelectionTooLong,
        Refusal::BadAnchor,
        Refusal::NestedReply,
        Refusal::NoSuchNote,
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
}
