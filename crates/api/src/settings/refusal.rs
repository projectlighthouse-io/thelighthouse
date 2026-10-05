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
    /// `varchar(255)` profile fields over their cap, one each so the form can
    /// say which.
    CompanyTooLong,
    EducationTooLong,
    /// Longer than `users.location`, which is narrower than the rest.
    LocationTooLong,
    /// Not an `http(s)` link with a host — see `view::is_web_url` — or, for
    /// the two named networks, a link to somewhere else.
    LinkedinNotALink,
    XNotALink,
    WebsiteNotALink,
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
            Self::CompanyTooLong => "company_too_long",
            Self::EducationTooLong => "education_too_long",
            Self::LocationTooLong => "location_too_long",
            Self::LinkedinNotALink => "linkedin_not_a_link",
            Self::XNotALink => "x_not_a_link",
            Self::WebsiteNotALink => "website_not_a_link",
        }
    }

    const fn status(self) -> StatusCode {
        match self {
            Self::NameRequired
            | Self::NameTooLong
            | Self::TaglineTooLong
            | Self::BioTooLong
            | Self::CompanyTooLong
            | Self::EducationTooLong
            | Self::LocationTooLong
            | Self::LinkedinNotALink
            | Self::XNotALink
            | Self::WebsiteNotALink => StatusCode::UNPROCESSABLE_ENTITY,
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
            Self::CompanyTooLong => "Company must be 255 characters or less.",
            Self::EducationTooLong => {
                "Education must be 255 characters or less."
            }
            Self::LocationTooLong => "Location must be 120 characters or less.",
            Self::LinkedinNotALink => {
                "Enter your LinkedIn link, starting with https://linkedin.com/"
            }
            Self::XNotALink => {
                "Enter your X link, starting with https://x.com/"
            }
            Self::WebsiteNotALink => {
                "Enter a full link, starting with https://"
            }
        }
    }

    /// The form field this refusal is about, as the page names its input.
    /// `None` for the one that is nobody's typing.
    const fn field(self) -> Option<&'static str> {
        match self {
            Self::NameRequired | Self::NameTooLong => Some("name"),
            Self::NoRandomness => None,
            Self::TaglineTooLong => Some("tagline"),
            Self::BioTooLong => Some("bio"),
            Self::CompanyTooLong => Some("company"),
            Self::EducationTooLong => Some("education"),
            Self::LocationTooLong => Some("location"),
            Self::LinkedinNotALink => Some("linkedin_url"),
            Self::XNotALink => Some("x_url"),
            Self::WebsiteNotALink => Some("website_url"),
        }
    }
}

/// Every field that is wrong, as one 422 the form draws beside its inputs.
///
/// A refusal with no field — the random source — has nothing to draw, so it
/// answers on its own instead.
pub(crate) fn refuse_all(causes: &[Refusal]) -> Response {
    if let Some(&unfielded) = causes.iter().find(|c| c.field().is_none()) {
        return refuse(unfielded);
    }

    response::invalid(
        causes
            .iter()
            .filter_map(|c| c.field().map(|field| (field, c.message()))),
    )
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

    const REFUSALS: [Refusal; 11] = [
        Refusal::NameRequired,
        Refusal::NameTooLong,
        Refusal::NoRandomness,
        Refusal::TaglineTooLong,
        Refusal::BioTooLong,
        Refusal::CompanyTooLong,
        Refusal::EducationTooLong,
        Refusal::LocationTooLong,
        Refusal::LinkedinNotALink,
        Refusal::XNotALink,
        Refusal::WebsiteNotALink,
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
