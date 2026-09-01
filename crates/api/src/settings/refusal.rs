//! Why a write was refused, and what the reader is told.
//!
//! The shape `projects::refusal` and `notes::refusal` have, for the reason
//! they give: an enum rather than a table of `&'static str`, so no call site
//! can invent its own wording for a case that already has one.

use axum::{http::StatusCode, response::Response};

use crate::{cache::CachePolicy, response};

/// Why a token was not created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// A token with no name is one nobody can tell apart on the page.
    NameRequired,
    /// Longer than the settings page can render.
    NameTooLong,
    /// The system random source could not be read.
    NoRandomness,
    /// Longer than `users.tagline` can hold.
    TaglineTooLong,
    BioTooLong,
    /// A `varchar(255)` profile field over its cap.
    FieldTooLong,
    /// Longer than `users.location`, which is narrower than the rest.
    LocationTooLong,
    /// Not an `http(s)` link with a host — see `view::is_web_url`.
    NotALink,
}

impl Refusal {
    /// Stable, never translated, and safe to branch on.
    const fn code(self) -> &'static str {
        match self {
            Self::NameRequired => "name_required",
            Self::NameTooLong => "name_too_long",
            Self::NoRandomness => "no_randomness",
            Self::TaglineTooLong => "tagline_too_long",
            Self::BioTooLong => "bio_too_long",
            Self::FieldTooLong => "field_too_long",
            Self::LocationTooLong => "location_too_long",
            Self::NotALink => "not_a_link",
        }
    }

    const fn status(self) -> StatusCode {
        match self {
            Self::NameRequired
            | Self::NameTooLong
            | Self::TaglineTooLong
            | Self::BioTooLong
            | Self::FieldTooLong
            | Self::LocationTooLong
            | Self::NotALink => StatusCode::UNPROCESSABLE_ENTITY,
            // Not the caller's fault, and retrying may well work.
            Self::NoRandomness => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// A fallback, not the final wording — a client with its own text for
    /// `code` shows that instead.
    const fn message(self) -> &'static str {
        match self {
            Self::NameRequired => "Give the token a name.",
            Self::NameTooLong => "That name is too long.",
            Self::NoRandomness => {
                "Could not generate a token just now. Try again."
            }
            Self::TaglineTooLong => "Tagline must be 160 characters or less.",
            Self::BioTooLong => "Bio must be 1000 characters or less.",
            Self::FieldTooLong => "That is longer than 255 characters.",
            Self::LocationTooLong => "Location must be 120 characters or less.",
            Self::NotALink => "Enter a full link, starting with https://",
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
        CachePolicy::NoStore,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFUSALS: [Refusal; 8] = [
        Refusal::NameRequired,
        Refusal::NameTooLong,
        Refusal::NoRandomness,
        Refusal::TaglineTooLong,
        Refusal::BioTooLong,
        Refusal::FieldTooLong,
        Refusal::LocationTooLong,
        Refusal::NotALink,
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
            // Prose, and it ends like prose — except the one that ends in a
            // url, where a full stop would look like part of the link.
            assert!(
                refusal.message().ends_with('.')
                    || refusal.message().ends_with('/'),
                "{code}"
            );
        }
    }

    #[test]
    fn nothing_here_answers_with_a_success() {
        for refusal in REFUSALS {
            assert!(!refusal.status().is_success(), "{refusal:?}");
        }
    }
}
