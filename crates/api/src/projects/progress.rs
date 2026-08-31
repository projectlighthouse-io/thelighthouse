//! What a reader has done on a task, derived from the attempt log.
//!
//! `task_attempts` is append-only: a row per submission, never updated. Status,
//! attempt count, when they started, when they finished and what they earned
//! are all queries over it rather than columns beside it — see the header of
//! `20260830060000_add_task_attempts.sql`, which is where those derivations are
//! written down and where they must stay in step with this file.
//!
//! The laravel model kept the summary in `user_task_progress` and the log in
//! `user_project_attempts`, and the two could disagree; `GetTask` carried a
//! branch for exactly that. Deriving costs one grouped query per project page
//! — behind an authenticated endpoint that is not cached anyway — and buys the
//! guarantee that there is nothing to disagree with.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// What one submission said. The `outcome` smallint, and its name.
///
/// Kept in step with the CHECK constraint in
/// `20260830060000_add_task_attempts.sql`, exactly as `ohara::Status` is with
/// the constraints that carry it: adding a variant means widening the CHECK in
/// a new migration, and widening the CHECK without adding the variant means
/// rows this code refuses to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    /// Tried, and the run said nothing either way. What luxctl records when a
    /// reader opens a task.
    Attempted,
    Passed,
    Failed,
    /// Given up on. Nothing submits this yet — `abandoned_deduction` in ohara
    /// implies it, and a ledger that could not record it would need a column
    /// added later rather than a variant.
    Abandoned,
}

impl Outcome {
    #[must_use]
    pub(crate) const fn as_db(self) -> i16 {
        match self {
            Self::Attempted => 0,
            Self::Passed => 1,
            Self::Failed => 2,
            Self::Abandoned => 3,
        }
    }

    /// `None` for anything the CHECK constraint should have refused — see
    /// `ohara::Status::from_db` for why this is not defaulted.
    #[must_use]
    pub(crate) const fn from_db(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Attempted),
            1 => Some(Self::Passed),
            2 => Some(Self::Failed),
            3 => Some(Self::Abandoned),
            _ => None,
        }
    }
}

/// Where a reader stands on a task, in the words luxctl already prints.
///
/// The names are laravel's `TaskStatus` enum, kept: the CLI matches on them,
/// and renaming `challenge_awaits` to something tidier would be a wire change
/// for no gain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Status {
    /// Nothing recorded in this run.
    #[default]
    ChallengeAwaits,
    Challenged,
    ChallengeCompleted,
    ChallengeFailed,
    ChallengeAbandoned,
}

/// One task's row in the derivation, as the store returns it.
///
/// **`attempts`, `started_at` and `completed_at` are this run's**; `points` is
/// every run's. A restart is meant to reset how a task *looks* while never
/// taking back points a reader has already banked — which is the rule laravel
/// had as `alreadyPassed`, checked globally while its progress row was
/// per-group. Keeping the two scopes explicit here is the only way that stays
/// true when somebody adds a field.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Progress {
    pub(crate) attempts: i64,
    pub(crate) started_at: Option<NaiveDateTime>,
    pub(crate) completed_at: Option<NaiveDateTime>,
    /// Whether any attempt in this run passed. Not the same as
    /// `completed_at.is_some()` only by accident, and read separately so the
    /// status rule below does not depend on that coincidence.
    pub(crate) passed_this_run: bool,
    /// Minutes since the first attempt of this run.
    ///
    /// **Measured by postgres, not by this process.** `started_at` is written
    /// by the server's clock, so the server's clock is what should be saying
    /// how old it is — comparing against this process's would make the scoring
    /// ladder and hint availability depend on how far the two have drifted.
    /// Zero for a run with nothing recorded in it.
    pub(crate) minutes: i64,
    /// The newest attempt in this run, or `None` when the run is untouched.
    pub(crate) latest: Option<Outcome>,
    /// The best a passed attempt was ever worth, across every run.
    pub(crate) best_points: i32,
    /// What hints on this task have cost, across every run. Unlocks are not
    /// undone by a restart: the reader read the hint.
    pub(crate) hints_deducted: i32,
}

impl Progress {
    /// The status a reader sees.
    ///
    /// Passing wins over whatever came after it. A reader who passes and then
    /// runs the task again to check something has not un-passed it, and the
    /// laravel writer had the same rule spelled as "don't downgrade status if
    /// already passed in this group".
    #[must_use]
    pub(crate) fn status(self) -> Status {
        if self.passed_this_run {
            return Status::ChallengeCompleted;
        }

        match self.latest {
            None => Status::ChallengeAwaits,
            Some(Outcome::Attempted) => Status::Challenged,
            Some(Outcome::Passed) => Status::ChallengeCompleted,
            Some(Outcome::Failed) => Status::ChallengeFailed,
            Some(Outcome::Abandoned) => Status::ChallengeAbandoned,
        }
    }

    /// What the reader has actually banked on this task.
    ///
    /// Never below zero: hints can cost more than a task is worth — a reader
    /// who unlocks every hint on a five-point task has spent more than they
    /// can earn — and a negative here would be a task that takes points away
    /// from the rest of the project.
    #[must_use]
    pub(crate) fn points(self) -> i32 {
        self.best_points.saturating_sub(self.hints_deducted).max(0)
    }

    /// How many attempts this run has already seen, as the ladder and the hint
    /// criteria count them.
    #[must_use]
    pub(crate) fn attempts_so_far(self) -> i32 {
        i32::try_from(self.attempts).unwrap_or(i32::MAX)
    }

    /// The attempt number this submission will be, within this run.
    ///
    /// The count so far plus the one being recorded, which is what the ladder
    /// is graded against: passing first time is `1`.
    #[must_use]
    pub(crate) fn next_attempt(self) -> i32 {
        i32::try_from(self.attempts.saturating_add(1)).unwrap_or(i32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTCOMES: [Outcome; 4] = [
        Outcome::Attempted,
        Outcome::Passed,
        Outcome::Failed,
        Outcome::Abandoned,
    ];

    #[test]
    fn every_outcome_survives_the_round_trip() {
        for outcome in OUTCOMES {
            assert_eq!(Outcome::from_db(outcome.as_db()), Some(outcome));
        }
    }

    /// The numbers are the storage format and are part of the schema — every
    /// row already written means what these say it means.
    #[test]
    fn the_stored_numbers_are_fixed() {
        assert_eq!(Outcome::Attempted.as_db(), 0);
        assert_eq!(Outcome::Passed.as_db(), 1);
        assert_eq!(Outcome::Failed.as_db(), 2);
        assert_eq!(Outcome::Abandoned.as_db(), 3);
    }

    #[test]
    fn a_value_the_check_constraint_would_refuse_is_not_guessed_at() {
        for out_of_range in [-1, 4, 99] {
            assert_eq!(Outcome::from_db(out_of_range), None);
        }
    }

    #[test]
    fn the_wire_names_are_the_ones_luxctl_already_matches_on() {
        assert_eq!(
            serde_json::to_string(&Outcome::Passed).unwrap(),
            "\"passed\""
        );
        assert_eq!(
            serde_json::to_string(&Status::ChallengeAwaits).unwrap(),
            "\"challenge_awaits\""
        );
        assert_eq!(
            serde_json::to_string(&Status::ChallengeCompleted).unwrap(),
            "\"challenge_completed\""
        );
    }

    #[test]
    fn an_untouched_run_is_awaiting_whatever_happened_in_an_earlier_one() {
        let progress = Progress {
            // A pass banked before the restart, and no attempt since.
            best_points: 25,
            ..Progress::default()
        };

        assert_eq!(progress.status(), Status::ChallengeAwaits);
        // ...and the points survive it, which is the whole point of the split.
        assert_eq!(progress.points(), 25);
        assert_eq!(progress.next_attempt(), 1);
    }

    #[test]
    fn a_pass_is_not_undone_by_running_the_task_again() {
        let progress = Progress {
            passed_this_run: true,
            latest: Some(Outcome::Failed),
            ..Progress::default()
        };

        assert_eq!(progress.status(), Status::ChallengeCompleted);
    }

    #[test]
    fn without_a_pass_the_newest_attempt_is_the_status() {
        for (outcome, expected) in [
            (Outcome::Attempted, Status::Challenged),
            (Outcome::Failed, Status::ChallengeFailed),
            (Outcome::Abandoned, Status::ChallengeAbandoned),
        ] {
            let progress = Progress {
                latest: Some(outcome),
                ..Progress::default()
            };

            assert_eq!(progress.status(), expected, "{outcome:?}");
        }
    }

    #[test]
    fn hints_come_off_the_points_but_never_below_zero() {
        let earned = Progress {
            best_points: 25,
            hints_deducted: 10,
            ..Progress::default()
        };
        assert_eq!(earned.points(), 15);

        // Every hint on a cheap task costs more than the task is worth. That
        // is the reader's choice; it must not drain the rest of the project.
        let overspent = Progress {
            best_points: 5,
            hints_deducted: 20,
            ..Progress::default()
        };
        assert_eq!(overspent.points(), 0);
    }

    #[test]
    fn the_attempt_being_recorded_is_the_one_the_ladder_grades() {
        let twice = Progress {
            attempts: 2,
            ..Progress::default()
        };

        assert_eq!(twice.attempts_so_far(), 2);
        assert_eq!(twice.next_attempt(), 3);
        // Passing first time is attempt 1, not 0 — the top tier of a `5:10:25`
        // ladder is reachable rather than one try short of it.
        assert_eq!(Progress::default().next_attempt(), 1);
    }
}
