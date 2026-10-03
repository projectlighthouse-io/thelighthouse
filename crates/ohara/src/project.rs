//! A project's yaml.
//!
//! A project is what luxctl drives: a list of tasks a reader completes from
//! their own terminal, checked locally. It is not a book, and the two share
//! nothing but the repo they live in — no chapters, no prose split at a
//! paywall, no locales.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Content, Error, Status, read, within_the_folder};

/// `projects/<slug>/project.yaml`.
#[derive(Debug, Deserialize)]
pub struct Project {
    /// The project's permanent identity, for the reason
    /// [`super::book::Book::id`] gives at length: `user_project_attempts` and
    /// `user_task_progress` point at a project and its tasks, and a reader's
    /// progress must not move when a slug is retuned.
    #[serde(default)]
    pub id: Option<Uuid>,
    /// Checked against the directory name.
    pub slug: String,
    pub name: String,
    /// The line above the name on the project's page.
    pub headline: Option<String>,
    /// One sentence, for a card in a listing.
    pub short_description: Option<String>,
    /// The pitch, in markdown, kept in the yaml because it is a paragraph
    /// rather than a document — [`Self::content_path`] is where the document
    /// goes.
    pub long_description: Option<String>,
    #[serde(default)]
    pub status: Status,
    pub difficulty: Option<String>,
    /// What luxctl runs the tasks in.
    ///
    /// `local|go|rust|c` means the reader's own machine, with the languages
    /// after the bar being the ones the project accepts. The rebuild has no
    /// VMs, so `local` is the only prefix that still means anything — but the
    /// field is carried verbatim rather than reinterpreted here, because it is
    /// luxctl that reads it.
    pub runner_image: Option<String>,
    #[serde(default)]
    pub unlock_mode: UnlockMode,
    /// A challenge has no companion book: the tasks are the whole thing.
    #[serde(default)]
    pub is_challenge: bool,
    #[serde(default)]
    pub is_featured: bool,
    #[serde(default)]
    pub featured_order: i16,
    /// Whether the project's page lists its tasks, or only pitches the project.
    #[serde(default = "yes")]
    pub show_tasks: bool,
    #[serde(default)]
    pub features: Vec<Feature>,
    /// The blueprint file, relative to this project's folder.
    ///
    /// Named rather than assumed, as a lesson names its prose. luxctl parses
    /// this file — the api only reads it and serves it whole, which is what
    /// the laravel sync did by storing it on every task row.
    pub blueprint_path: String,
    /// The project's long-form markdown, relative to its folder.
    ///
    /// A single file, not a per-locale map: projects have never been
    /// translated, and the translation tables were dropped. `None` for a
    /// project that is only its tasks.
    pub content_path: Option<ContentPath>,
}

/// `show_tasks` defaults to true, and serde needs a function to say so.
const fn yes() -> bool {
    true
}

/// What the reader's page shows as bullet points beside the pitch.
#[derive(Debug, Deserialize)]
pub struct Feature {
    pub title: String,
    pub description: Option<String>,
    /// A name from the frontend's icon set. Not validated here — this crate
    /// has no idea which icons exist.
    pub icon: Option<String>,
}

/// One language, spelled as a map so it reads like a lesson's `content_path`
/// and can grow into one without the file format changing shape.
#[derive(Debug, Deserialize)]
pub struct ContentPath {
    pub en: String,
}

/// How a project's tasks unlock.
///
/// The database stores this as a smallint; the yaml spells it, because
/// `unlock_mode: 1` in a file a human edits is a number nobody can read.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum UnlockMode {
    /// Every task is available from the start.
    #[default]
    Open,
    /// A task opens when the one before it is complete.
    Sequential,
}

impl Project {
    /// Rejects a project that could not be read, or could be read from
    /// somewhere it has no business reaching.
    fn validate(&self) -> Result<(), String> {
        within_the_folder(&self.blueprint_path, "project")
            .map_err(|cause| format!("blueprint_path: {cause}"))?;

        if let Some(content) = &self.content_path {
            within_the_folder(&content.en, "project")
                .map_err(|cause| format!("content_path.en: {cause}"))?;
        }

        Ok(())
    }
}

impl Content {
    /// Reads and parses `projects/<slug>/project.yaml`.
    ///
    /// A task folder on disk is not listed anywhere in this file — unlike a
    /// book, which names every lesson folder it publishes. A project's tasks
    /// are whatever is in its `tasks/` directory, in folder order, because the
    /// blueprint they came from has one order and there is no second list that
    /// could disagree with it.
    ///
    /// # Errors
    ///
    /// The file being absent or unparseable, its `slug` disagreeing with the
    /// directory it sits in, or a path in it that climbs out of the project's
    /// own folder.
    pub fn project(&self, slug: &str) -> Result<Project, Error> {
        let path = self.project_dir(slug).join("project.yaml");
        let raw = read(&path)?;

        let project: Project =
            serde_norway::from_str(&raw).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause: cause.to_string(),
            })?;

        if project.slug != slug {
            return Err(Error::Malformed {
                path,
                cause: format!(
                    "slug is {:?} but the directory is {slug:?}",
                    project.slug
                ),
            });
        }

        project
            .validate()
            .map_err(|cause| Error::Malformed { path, cause })?;

        Ok(project)
    }

    /// The blueprint, verbatim.
    ///
    /// Not parsed, here or anywhere in this workspace: luxctl owns that
    /// grammar. What the api does with it is hand it back.
    ///
    /// # Errors
    ///
    /// The file named by `blueprint_path` being absent or unreadable.
    pub fn blueprint(&self, project: &Project) -> Result<String, Error> {
        self.blueprint_at(&project.slug, &project.blueprint_path)
    }

    /// The blueprint, given a slug and a path already taken off a [`Project`].
    ///
    /// The form the catalogue calls. It copies both out while it holds the
    /// snapshot and opens the file after letting go, so a reload in between
    /// cannot pair one project's slug with another's path.
    ///
    /// # Errors
    ///
    /// As [`Self::blueprint`].
    pub fn blueprint_at(
        &self,
        project: &str,
        path: &str,
    ) -> Result<String, Error> {
        read(&self.project_dir(project).join(path))
    }

    /// A project's long-form markdown, given the path off its `content_path`.
    ///
    /// # Errors
    ///
    /// The file being absent or unreadable.
    pub fn overview_at(
        &self,
        project: &str,
        path: &str,
    ) -> Result<String, Error> {
        read(&self.project_dir(project).join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn a_project_parses() {
        let project = fixture::content().project("fixture-project").unwrap();

        assert_eq!(project.name, "Fixture Project");
        assert_eq!(project.unlock_mode, UnlockMode::Sequential);
        assert_eq!(project.status, Status::Published);
        assert!(project.show_tasks);
        assert_eq!(project.features.len(), 1);
    }

    #[test]
    fn the_blueprint_is_read_whole() {
        let content = fixture::content();
        let project = content.project("fixture-project").unwrap();
        let blueprint = content.blueprint(&project).unwrap();

        assert!(blueprint.contains("phase \"listen\""));
    }

    #[test]
    fn a_slug_that_disagrees_with_its_directory_is_refused() {
        let error = fixture::content().project("nothing-here").unwrap_err();

        assert!(matches!(error, Error::Unreadable { .. }));
    }

    #[test]
    fn a_path_climbing_out_of_the_folder_is_refused() {
        let project: Project = serde_norway::from_str(
            "slug: p\nname: P\nblueprint_path: ../../etc/passwd\n",
        )
        .unwrap();

        assert!(project.validate().is_err());
    }
}
