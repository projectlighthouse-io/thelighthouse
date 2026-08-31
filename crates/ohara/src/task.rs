//! A task's yaml, and the folder it sits in.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Content, Error, lesson::Folder, read, within_the_folder};

/// `projects/<project>/tasks/<order>-<slug>/task.yaml`.
///
/// No `sort_order` field: the folder's number is the order, as it is for a
/// lesson, so a task cannot claim a position the directory disagrees with.
#[derive(Debug, Deserialize)]
pub struct Task {
    /// The task's permanent identity. `user_task_progress` points at it, so a
    /// task that is renumbered or renamed keeps every completion it earned.
    #[serde(default)]
    pub id: Option<Uuid>,
    /// Checked against the folder name with its number stripped.
    pub slug: String,
    pub title: String,
    /// What completing it is worth.
    #[serde(default)]
    pub points: i32,
    /// 1 free for all, 2 free once signed in, 3 paid. A smallint the laravel
    /// schema defined and the rebuild still stores; [`Self::is_free`] is what
    /// anything downstream should read.
    #[serde(default = "one")]
    pub visibility_level: i32,
    #[serde(default)]
    pub is_free: bool,
    /// What abandoning the task costs, in points.
    #[serde(default = "five")]
    pub abandoned_deduction: i32,
    #[serde(default)]
    pub input_type: InputType,
    /// The scoring ladder, in luxctl's own notation — `5:10:25|10:20:15` is
    /// read by the CLI and passed through untouched here.
    pub scores: Option<String>,
    #[serde(default)]
    pub hints: Vec<Hint>,
    /// The task's prose, relative to its folder. Absent for a task whose
    /// blueprint phase carried no description.
    pub content_path: Option<TaskContentPath>,
}

const fn one() -> i32 {
    1
}

const fn five() -> i32 {
    5
}

/// One tier of a `scores` ladder: `5:10:25` as (attempts, minutes, points).
///
/// `None` for anything that is not three numbers, so one bad tier is skipped
/// and the rest of the ladder still works.
fn tier(text: &str) -> Option<(i32, i64, i32)> {
    let mut parts = text.split(':');
    let (Some(attempts), Some(minutes), Some(points)) =
        (parts.next(), parts.next(), parts.next())
    else {
        return None;
    };

    if parts.next().is_some() {
        return None;
    }

    Some((
        attempts.parse().ok()?,
        minutes.parse().ok()?,
        points.parse().ok()?,
    ))
}

/// One language, for the reason [`super::project::ContentPath`] gives.
#[derive(Debug, Deserialize)]
pub struct TaskContentPath {
    pub en: String,
}

/// Whether the task expects an answer typed into luxctl, or only a program
/// that passes its probes.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    #[default]
    None,
    Text,
}

/// A hint a reader can spend points to unlock.
#[derive(Debug, Deserialize)]
pub struct Hint {
    /// `user_unlocked_hints` points at this, and luxctl unlocks a hint by it.
    /// An unlock a reader has paid for must keep naming the same hint when the
    /// blueprint around it is edited.
    #[serde(default)]
    pub id: Option<Uuid>,
    pub text: String,
    /// luxctl's notation for when the hint becomes offerable — `5:3:A`. Read
    /// by the CLI, carried verbatim here.
    pub unlock_criteria: Option<String>,
    #[serde(default = "five")]
    pub points_deduction: i32,
}

impl Hint {
    /// Whether this hint may be offered yet.
    ///
    /// `10:3:A` is "after 10 minutes **or** 3 attempts, priority A". Either
    /// condition alone opens the hint, and a `0` switches its condition off
    /// rather than satisfying it immediately — `0:3:A` is attempts only, and
    /// `0:0:A` is a hint that never opens, which is how a hint is written but
    /// held back.
    ///
    /// The priority letter orders hints for a reader and is not read here.
    ///
    /// A criteria string that is absent or not three parts is `false`: a hint
    /// nobody can reach is a worse failure than a hint that opens late, and
    /// this way the typo is visible rather than paid for.
    #[must_use]
    pub fn offerable(&self, minutes: i64, attempts: i32) -> bool {
        let Some(criteria) = self.unlock_criteria.as_deref() else {
            return false;
        };

        let mut parts = criteria.split(':');
        let (Some(after_minutes), Some(after_attempts), Some(_priority)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return false;
        };

        if parts.next().is_some() {
            return false;
        }

        let (Ok(after_minutes), Ok(after_attempts)) =
            (after_minutes.parse::<i64>(), after_attempts.parse::<i32>())
        else {
            return false;
        };

        (after_minutes > 0 && minutes >= after_minutes)
            || (after_attempts > 0 && attempts >= after_attempts)
    }
}

impl Task {
    /// What passing this task is worth, having taken `attempts` tries over
    /// `minutes`.
    ///
    /// **The ladder is content, so decoding it lives beside the field.** The
    /// api needs the number — points are worked out on the server, never sent
    /// by the client — and it is the same notation luxctl reads, so a second
    /// reading of it somewhere else is a second thing to keep in step.
    ///
    /// `5:10:25|10:20:15` is "25 points within 5 attempts and 10 minutes, else
    /// 15 within 10 and 20". Tiers are tried in order and the first that fits
    /// both bounds wins, so they are written generous-first. A run that fits no
    /// tier earns the last tier's points, which is what makes the ladder a
    /// floor rather than a cliff — the laravel `calculatePoints` did the same,
    /// and readers have points banked under it.
    ///
    /// An empty or absent `scores` falls back to [`Self::points`], and a
    /// malformed tier is skipped rather than failing the submission: a typo in
    /// one line of yaml should cost that tier, not a reader's completed task.
    #[must_use]
    pub fn points_for(&self, attempts: i32, minutes: i64) -> i32 {
        let Some(ladder) = self.scores.as_deref().filter(|s| !s.is_empty())
        else {
            return self.points;
        };

        let tiers = || ladder.split('|').filter_map(tier);

        for (max_attempts, max_minutes, points) in tiers() {
            if attempts <= max_attempts && minutes <= max_minutes {
                return points;
            }
        }

        // Nothing fitted: the last readable tier's points, or the task's flat
        // value if every tier was malformed.
        tiers()
            .next_back()
            .map_or(self.points, |(_, _, points)| points)
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(content) = &self.content_path {
            within_the_folder(&content.en, "task")
                .map_err(|cause| format!("content_path.en: {cause}"))?;
        }

        Ok(())
    }
}

impl Content {
    /// Reads and parses a task's yaml, given its folder — `01-listen-on-port`.
    ///
    /// # Errors
    ///
    /// As [`Content::project`]: absent, unparseable, a slug that disagrees
    /// with its folder, or a path that climbs out of the task's own folder.
    /// Also a folder name that is not `<number>-<slug>`.
    pub fn task(&self, project: &str, folder: &str) -> Result<Task, Error> {
        let path = self.task_dir(project, folder).join("task.yaml");

        let named =
            Folder::parse(folder).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause,
            })?;

        let raw = read(&path)?;

        let task: Task =
            serde_norway::from_str(&raw).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause: cause.to_string(),
            })?;

        if task.slug != named.slug {
            return Err(Error::Malformed {
                path,
                cause: format!(
                    "slug is {:?} but the folder says {:?}",
                    task.slug, named.slug
                ),
            });
        }

        task.validate()
            .map_err(|cause| Error::Malformed { path, cause })?;

        Ok(task)
    }

    /// A task's prose.
    ///
    /// # Errors
    ///
    /// The file named by `content_path` being absent or unreadable.
    pub fn task_body(
        &self,
        project: &str,
        folder: &str,
        task: &Task,
    ) -> Result<Option<String>, Error> {
        let Some(content) = &task.content_path else {
            return Ok(None);
        };

        self.task_body_at(project, folder, &content.en).map(Some)
    }

    /// A task's prose, given a path already taken off a [`Task`].
    ///
    /// The form the catalogue calls — see [`Content::blueprint_at`] for why
    /// the path arrives separately rather than being read out of the task
    /// while the snapshot is still held.
    ///
    /// # Errors
    ///
    /// The file being absent or unreadable.
    pub fn task_body_at(
        &self,
        project: &str,
        folder: &str,
        path: &str,
    ) -> Result<String, Error> {
        read(&self.task_dir(project, folder).join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn a_task_parses() {
        let task = fixture::content()
            .task("fixture-project", "01-listen-on-port")
            .unwrap();

        assert_eq!(task.title, "Listen on a Port");
        assert_eq!(task.points, 25);
        assert!(task.is_free);
        assert_eq!(task.input_type, InputType::None);
        assert_eq!(task.hints.len(), 1);
    }

    #[test]
    fn the_body_is_read_when_one_is_named() {
        let content = fixture::content();
        let task = content
            .task("fixture-project", "01-listen-on-port")
            .unwrap();

        let body = content
            .task_body("fixture-project", "01-listen-on-port", &task)
            .unwrap();

        assert!(body.unwrap().contains("bind"));
    }

    #[test]
    fn a_task_with_no_prose_reads_none() {
        let content = fixture::content();
        let task = content.task("fixture-project", "02-say-something").unwrap();

        assert!(
            content
                .task_body("fixture-project", "02-say-something", &task)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_slug_that_disagrees_with_its_folder_is_refused() {
        // In `broken` rather than beside the tasks that parse: a catalogue
        // walks every task folder a project has, so one planted mistake in the
        // shipped fixture would fail every test about something else.
        let error = fixture::broken()
            .task("fixture-project", "03-misnamed")
            .unwrap_err();

        let Error::Malformed { cause, .. } = error else {
            panic!("expected a malformed task");
        };

        assert!(cause.contains("but the folder says"));
    }

    /// A task carrying a ladder, built here rather than read off the fixture:
    /// these assertions are about the notation, and pinning them to a file
    /// somebody may retune would make them fail for the wrong reason.
    fn scored(scores: Option<&str>, points: i32) -> Task {
        Task {
            id: None,
            slug: "t".to_owned(),
            title: "T".to_owned(),
            points,
            visibility_level: 1,
            is_free: true,
            abandoned_deduction: 5,
            input_type: InputType::None,
            scores: scores.map(str::to_owned),
            hints: Vec::new(),
            content_path: None,
        }
    }

    #[test]
    fn the_first_tier_that_fits_both_bounds_is_what_it_is_worth() {
        let task = scored(Some("5:10:25|10:20:15"), 0);

        // Inside the first tier on both counts.
        assert_eq!(task.points_for(1, 0), 25);
        assert_eq!(task.points_for(5, 10), 25);
        // Over on attempts, or over on minutes: the next tier down.
        assert_eq!(task.points_for(6, 0), 15);
        assert_eq!(task.points_for(1, 11), 15);
    }

    #[test]
    fn a_run_that_fits_no_tier_earns_the_last_one() {
        // A floor rather than a cliff, which is what the laravel ladder did
        // and what readers already have points banked under.
        let task = scored(Some("5:10:25|10:20:15"), 0);

        assert_eq!(task.points_for(99, 999), 15);
    }

    #[test]
    fn no_ladder_at_all_is_the_tasks_flat_points() {
        assert_eq!(scored(None, 7).points_for(99, 999), 7);
        assert_eq!(scored(Some(""), 7).points_for(1, 0), 7);
    }

    #[test]
    fn one_malformed_tier_costs_that_tier_and_not_the_submission() {
        let task = scored(Some("nope|10:20:15"), 3);

        assert_eq!(task.points_for(1, 0), 15);
        // ...and a ladder that is entirely unreadable falls back to `points`.
        assert_eq!(scored(Some("nope|also-nope"), 3).points_for(1, 0), 3);
    }

    fn hinted(criteria: Option<&str>) -> Hint {
        Hint {
            id: None,
            text: "h".to_owned(),
            unlock_criteria: criteria.map(str::to_owned),
            points_deduction: 5,
        }
    }

    #[test]
    fn either_half_of_the_unlock_criteria_opens_a_hint() {
        let hint = hinted(Some("10:3:A"));

        assert!(!hint.offerable(0, 0));
        assert!(!hint.offerable(9, 2));
        // Long enough...
        assert!(hint.offerable(10, 0));
        // ...or often enough.
        assert!(hint.offerable(0, 3));
    }

    #[test]
    fn a_zero_switches_its_condition_off_rather_than_satisfying_it() {
        // `0:3:A` is attempts only. Read the other way round, every hint in
        // the repo would be open the moment a reader started.
        let attempts_only = hinted(Some("0:3:A"));
        assert!(!attempts_only.offerable(9_999, 0));
        assert!(attempts_only.offerable(0, 3));

        let minutes_only = hinted(Some("10:0:A"));
        assert!(!minutes_only.offerable(0, 9_999));
        assert!(minutes_only.offerable(10, 0));

        // Written, and deliberately never offered.
        assert!(!hinted(Some("0:0:A")).offerable(9_999, 9_999));
    }

    #[test]
    fn an_unreadable_criteria_holds_the_hint_back() {
        for criteria in [
            None,
            Some(""),
            Some("10:3"),
            Some("10:3:A:B"),
            Some("ten:3:A"),
        ] {
            assert!(!hinted(criteria).offerable(9_999, 9_999), "{criteria:?}");
        }
    }

    #[test]
    fn a_folder_without_a_number_is_refused() {
        let error = fixture::content()
            .task("fixture-project", "listen-on-port")
            .unwrap_err();

        assert!(matches!(error, Error::Malformed { .. }));
    }
}
