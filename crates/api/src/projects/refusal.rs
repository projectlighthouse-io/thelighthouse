//! Why a write was refused, and what the reader is told.
//!
//! The same shape `notes::refusal` has, and for the same reason: an enum
//! rather than a table of `&'static str`, so no call site can invent its own
//! wording for a case that already has one.
//!
//! The wording is for somebody reading a terminal, which is the one difference
//! from the notes messages — "run `lux restart`" is useful advice in a CLI and
//! meaningless in a browser.

use axum::{http::StatusCode, response::Response};

use crate::{cache::CachePolicy, response};

/// Why a write was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// The body named a task that is not in the named project.
    TaskNotInProject,
    /// A sequential project, and the task before this one is not done.
    TaskLocked,
    /// The task is paid and the reader does not hold the project's book.
    TaskNotEntitled,
    /// The hint's unlock criteria are not met yet.
    HintNotAvailable,
}

impl Refusal {
    /// Stable, never translated, and safe to branch on. As much a contract as
    /// the field names — changing one breaks any client that reads it.
    const fn code(self) -> &'static str {
        match self {
            Self::TaskNotInProject => "task_not_in_project",
            Self::TaskLocked => "task_locked",
            Self::TaskNotEntitled => "task_not_entitled",
            Self::HintNotAvailable => "hint_not_available",
        }
    }

    /// The status that goes with it.
    ///
    /// Three of these are 403 and one is 422, which is the split laravel had:
    /// a task that is not in the project is a malformed request, while the
    /// other three are well-formed requests this reader may not make.
    const fn status(self) -> StatusCode {
        match self {
            Self::TaskNotInProject => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TaskLocked
            | Self::TaskNotEntitled
            | Self::HintNotAvailable => StatusCode::FORBIDDEN,
        }
    }

    /// A fallback, not the final wording — a client with its own text for
    /// `code` shows that instead. Written for a terminal.
    const fn message(self) -> &'static str {
        match self {
            Self::TaskNotInProject => {
                "That task does not belong to that project."
            }
            Self::TaskLocked => "Finish the task before this one first.",
            Self::TaskNotEntitled => {
                "This task is part of the paid course. Read more at \
                 projectlighthouse.io."
            }
            Self::HintNotAvailable => {
                "That hint is not available yet. Keep going, or try again in \
                 a few minutes."
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
        CachePolicy::NoStore,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant, so adding one fails here first — the prompt to give
    /// luxctl an entry for its code.
    const REFUSALS: [Refusal; 4] = [
        Refusal::TaskNotInProject,
        Refusal::TaskLocked,
        Refusal::TaskNotEntitled,
        Refusal::HintNotAvailable,
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
    fn nothing_here_answers_with_a_success() {
        for refusal in REFUSALS {
            assert!(refusal.status().is_client_error(), "{refusal:?}");
        }
    }
}
