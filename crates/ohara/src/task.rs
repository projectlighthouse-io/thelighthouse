//! A task's yaml, and the folder it sits in.

use serde::Deserialize;
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

/// One language, for the reason [`super::project::ContentPath`] gives.
#[derive(Debug, Deserialize)]
pub struct TaskContentPath {
    pub en: String,
}

/// Whether the task expects an answer typed into luxctl, or only a program
/// that passes its probes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
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

impl Task {
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

        read(&self.task_dir(project, folder).join(&content.en)).map(Some)
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
        let error = fixture::content()
            .task("fixture-project", "03-misnamed")
            .unwrap_err();

        let Error::Malformed { cause, .. } = error else {
            panic!("expected a malformed task");
        };

        assert!(cause.contains("but the folder says"));
    }

    #[test]
    fn a_folder_without_a_number_is_refused() {
        let error = fixture::content()
            .task("fixture-project", "listen-on-port")
            .unwrap_err();

        assert!(matches!(error, Error::Malformed { .. }));
    }
}
