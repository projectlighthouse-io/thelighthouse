//! What luxctl sends when it records an attempt, and whether it is acceptable.
//!
//! **Three of the five fields the laravel request accepted are gone.**
//! `points_achieved` was accepted and then overwritten with a server-side
//! calculation on every path, so it was a field the client could set and never
//! affect — it is not accepted here at all, because a parameter that is
//! ignored is one somebody will eventually assume works. `run` is not a field
//! either: the api reads it from `project_restarts`, and a client that picks
//! its own run number can rewrite its history.

use serde::Deserialize;

use super::progress::Outcome;

/// What `context` may hold, matching the column's `varchar(5000)`.
///
/// The cap is the column's, so the check happens before the write rather than
/// as a database error with a constraint name in it.
const MAX_CONTEXT: usize = 5_000;

/// The body of `POST /api/v1/projects/attempts`.
#[derive(Debug, Deserialize)]
pub(crate) struct Attempt {
    pub(crate) project_slug: String,
    /// The task's uuid. luxctl reads it off the project listing, which is the
    /// only place it comes from — there is no integer id any more.
    pub(crate) task_id: uuid::Uuid,
    /// `attempted`, `passed`, `failed` or `abandoned`. An unknown word fails
    /// deserialisation, so a typo is a 400 rather than a row recorded as
    /// something nobody meant.
    pub(crate) task_outcome: Outcome,
    /// Whatever the CLI said about the run. Free text, shown back to the
    /// reader and never parsed.
    #[serde(default)]
    pub(crate) task_outcome_context: Option<String>,
}

impl Attempt {
    /// The context, trimmed to what the column can hold.
    ///
    /// Truncated rather than refused: a long stack trace is a reader having a
    /// bad time, and rejecting their submission over the size of the
    /// explanation would lose the attempt as well as the explanation. Cut on a
    /// character boundary, because slicing a utf-8 string anywhere else
    /// panics.
    ///
    /// Empty is `None` — a stored empty string would be a context that renders
    /// as a blank box rather than as nothing.
    #[must_use]
    pub(crate) fn context(&self) -> Option<&str> {
        let context = self.task_outcome_context.as_deref()?.trim();

        if context.is_empty() {
            return None;
        }

        Some(match context.char_indices().nth(MAX_CONTEXT) {
            Some((at, _)) => context.get(..at).unwrap_or_default(),
            None => context,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_context(context: Option<&str>) -> Attempt {
        Attempt {
            project_slug: "p".to_owned(),
            task_id: uuid::Uuid::nil(),
            task_outcome: Outcome::Passed,
            task_outcome_context: context.map(str::to_owned),
        }
    }

    #[test]
    fn an_outcome_is_one_of_the_four_words_or_the_body_is_refused() {
        let body = |outcome: &str| {
            format!(
                r#"{{"project_slug":"p",
                     "task_id":"00000000-0000-0000-0000-000000000000",
                     "task_outcome":"{outcome}"}}"#
            )
        };

        for outcome in ["attempted", "passed", "failed", "abandoned"] {
            assert!(
                serde_json::from_str::<Attempt>(&body(outcome)).is_ok(),
                "{outcome}"
            );
        }

        for outcome in ["Passed", "pass", "", "completed"] {
            assert!(
                serde_json::from_str::<Attempt>(&body(outcome)).is_err(),
                "{outcome}"
            );
        }
    }

    #[test]
    fn a_task_id_that_is_not_a_uuid_is_refused_by_the_parser() {
        // The integer ids are gone. A client still sending one gets a 400
        // naming the field rather than a lookup that finds nothing.
        let body =
            r#"{"project_slug":"p","task_id":7,"task_outcome":"passed"}"#;

        assert!(serde_json::from_str::<Attempt>(body).is_err());
    }

    #[test]
    fn a_client_cannot_choose_its_points_or_its_run() {
        // Both present, both ignored: serde drops unknown fields, and neither
        // is a field on this struct. The row that gets written takes its
        // points from the ladder and its run from `project_restarts`.
        let body = r#"{"project_slug":"p",
                       "task_id":"00000000-0000-0000-0000-000000000000",
                       "task_outcome":"passed",
                       "points_achieved":9999,
                       "run":1}"#;

        assert!(serde_json::from_str::<Attempt>(body).is_ok());
    }

    #[test]
    fn nothing_worth_storing_is_none_rather_than_an_empty_string() {
        assert!(with_context(None).context().is_none());
        assert!(with_context(Some("")).context().is_none());
        assert!(with_context(Some("   \n ")).context().is_none());
    }

    #[test]
    fn a_context_is_trimmed_and_kept() {
        assert_eq!(
            with_context(Some("  exit status 1  ")).context(),
            Some("exit status 1")
        );
    }

    #[test]
    fn an_enormous_context_is_cut_rather_than_costing_the_attempt() {
        let long = "x".repeat(MAX_CONTEXT + 500);
        let attempt = with_context(Some(&long));

        assert_eq!(attempt.context().unwrap().len(), MAX_CONTEXT);
    }

    #[test]
    fn cutting_a_context_never_splits_a_character() {
        // Three bytes each, so a byte-wise cut at 5000 would land inside one
        // and panic. `panic = "abort"` in release makes that the whole site.
        let long = "✂".repeat(MAX_CONTEXT + 10);
        let cut = with_context(Some(&long));

        assert_eq!(cut.context().unwrap().chars().count(), MAX_CONTEXT);
    }
}
