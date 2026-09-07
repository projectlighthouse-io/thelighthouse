//! Why a write was refused, and what the author is told.

use axum::response::Response;

use crate::response::bad_request;

/// Why a write was refused. An enum for the reason `notes::refusal::Refusal`
/// is one: a `Result<_, &str>` makes every string literal a valid error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    EmptyTitle,
    TitleTooLong,
    EmptySubtitle,
    SubtitleTooLong,
    EmptyBody,
    BodyTooLong,
    /// A topic that is not in `payload::CATEGORIES`, whether it was sent on a
    /// write or asked for as a filter.
    UnknownCategory,
    /// A write with an empty topic list. An article is about something.
    NoTopics,
    TooManyTopics,
    /// Two articles minted the same slug. Vanishingly rare and worth saying
    /// out loud rather than logging as a 500 — see `store::insert`.
    SlugTaken,
}

impl Refusal {
    /// Stable, never translated, and safe to branch on. As much a contract as
    /// the field names — changing one breaks any client that reads it.
    const fn code(self) -> &'static str {
        match self {
            Self::EmptyTitle => "article_title_empty",
            Self::TitleTooLong => "article_title_too_long",
            Self::EmptySubtitle => "article_subtitle_empty",
            Self::SubtitleTooLong => "article_subtitle_too_long",
            Self::EmptyBody => "article_body_empty",
            Self::BodyTooLong => "article_body_too_long",
            Self::UnknownCategory => "article_category_unknown",
            Self::NoTopics => "article_topics_empty",
            Self::TooManyTopics => "article_topics_too_many",
            Self::SlugTaken => "article_slug_taken",
        }
    }

    /// A fallback, not the final wording — `docs/rebuild.md` puts UI strings in
    /// the frontend's own typed table, keyed by the code above.
    ///
    /// The lengths and the topic list are repeated in words because a
    /// `const fn` cannot interpolate; change these and `payload`'s constants
    /// together — `every_category_is_named_in_the_refusal` is what fails if
    /// the two drift.
    const fn message(self) -> &'static str {
        match self {
            Self::EmptyTitle => "Please give your article a title.",
            Self::TitleTooLong => "A title must not exceed 200 characters.",
            Self::EmptySubtitle => {
                "Please add a subtitle — it is the line readers decide from."
            }
            Self::SubtitleTooLong => {
                "A subtitle must not exceed 200 characters."
            }
            Self::EmptyBody => "Please write something.",
            Self::BodyTooLong => {
                "An article must not exceed 50,000 characters."
            }
            Self::UnknownCategory => {
                "Please pick one of: go, rust, containers, systems, docker, \
                 kubernetes, networking, dsa."
            }
            Self::NoTopics => "Please pick at least one topic.",
            Self::TooManyTopics => "An article can carry at most three topics.",
            Self::SlugTaken => "That address was just taken. Please try again.",
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
    const REFUSALS: [Refusal; 10] = [
        Refusal::EmptyTitle,
        Refusal::TitleTooLong,
        Refusal::EmptySubtitle,
        Refusal::SubtitleTooLong,
        Refusal::EmptyBody,
        Refusal::BodyTooLong,
        Refusal::UnknownCategory,
        Refusal::NoTopics,
        Refusal::TooManyTopics,
        Refusal::SlugTaken,
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

    /// The codes are namespaced, so a frontend table shared with `notes` has
    /// no two entries meaning different things.
    #[test]
    fn no_code_collides_with_a_note_refusal() {
        for refusal in REFUSALS {
            let code = refusal.code();

            assert!(code.starts_with("article_"), "{code} is not namespaced");
        }
    }

    /// The message spells the list out because a `const fn` cannot
    /// interpolate. This is what fails when a topic is added to
    /// `payload::CATEGORIES` and not to the sentence an author reads.
    #[test]
    fn every_category_is_named_in_the_refusal() {
        let message = Refusal::UnknownCategory.message();

        for category in crate::articles::payload::CATEGORIES {
            assert!(message.contains(category), "{category} is not offered");
        }
    }
}
