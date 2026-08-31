//! What luxctl sees, as json.
//!
//! Separate from `ohara`, which describes what is *on disk*. A yaml field is
//! not automatically a wire field: `status` never ships (nothing unpublished is
//! in the catalogue) and the folder name never ships (a url carries the slug).
//!
//! **A hint's text is not here.** The project and task views say that a hint
//! exists, what it will cost and whether it is open yet; the words are served
//! only by `GET /tasks/{task}/hints`, and only for a hint this reader has paid
//! to unlock. The laravel project endpoint shipped every hint's text to
//! everybody, which made the unlock endpoint beside it decorative.
//!
//! **`id` is a uuid.** The integer primary keys are gone — content mints its
//! own ids now, in the yaml, before any row exists. See
//! `20260830050000_content_owns_project_ids.sql`.

use chrono::NaiveDateTime;
use ohara::{
    catalog::{ProjectEntry, TaskEntry},
    project::UnlockMode,
    task::{Hint, InputType},
};
use serde::Serialize;
use uuid::Uuid;

use super::progress::{Progress, Status};

/// A project in a listing.
///
/// No tasks — a listing of sixteen projects should not carry a hundred and
/// sixty-two task descriptions nobody scrolled to.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectSummary<'p> {
    id: Option<Uuid>,
    slug: &'p str,
    name: &'p str,
    headline: Option<&'p str>,
    short_description: Option<&'p str>,
    difficulty: Option<&'p str>,
    /// What luxctl runs the tasks in — `local|go|rust|c`. Carried verbatim;
    /// the CLI is what reads it.
    runner_image: Option<&'p str>,
    is_challenge: bool,
    is_featured: bool,
    featured_order: i16,
    show_tasks: bool,
    unlock_mode: UnlockMode,
    related_book_slug: Option<&'p str>,
    task_count: usize,
}

impl<'p> ProjectSummary<'p> {
    pub(crate) fn of(entry: &'p ProjectEntry) -> Self {
        let project = &entry.project;

        Self {
            id: project.id,
            slug: &project.slug,
            name: &project.name,
            headline: project.headline.as_deref(),
            short_description: project.short_description.as_deref(),
            difficulty: project.difficulty.as_deref(),
            runner_image: project.runner_image.as_deref(),
            is_challenge: project.is_challenge,
            is_featured: project.is_featured,
            featured_order: project.featured_order,
            show_tasks: project.show_tasks,
            unlock_mode: project.unlock_mode,
            related_book_slug: project.related_book_slug.as_deref(),
            task_count: entry.tasks().count(),
        }
    }
}

/// A project's own page: the summary, the pitch, the blueprint and the tasks.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectDetail<'p> {
    #[serde(flatten)]
    summary: ProjectSummary<'p>,
    long_description: Option<&'p str>,
    features: Vec<FeatureView<'p>>,
    /// The project's long-form markdown, unrendered.
    ///
    /// Markdown rather than html, unlike a lesson: the reader of this is a
    /// terminal, and luxctl decides how to print it. Absent for a project that
    /// is only its tasks.
    overview: Option<String>,
    /// The `.bp`, whole and unparsed.
    ///
    /// **Once per project, not once per task.** `tasks.blueprint` used to hold
    /// a copy of this on every row — eighteen copies of the same 46 KB in one
    /// project — and it was the api's answer when luxctl asked. One file, read
    /// once, served here. Nothing in this workspace parses it: luxctl owns that
    /// grammar and the api's whole job is to hand it back.
    blueprint: Option<String>,
    tasks: Vec<TaskView<'p>>,
}

impl<'p> ProjectDetail<'p> {
    pub(crate) fn of(
        entry: &'p ProjectEntry,
        overview: Option<String>,
        blueprint: Option<String>,
        tasks: Vec<TaskView<'p>>,
    ) -> Self {
        Self {
            summary: ProjectSummary::of(entry),
            long_description: entry.project.long_description.as_deref(),
            features: FeatureView::all_of(entry),
            overview,
            blueprint,
            tasks,
        }
    }
}

/// A bullet point beside the pitch.
#[derive(Debug, Serialize)]
pub(crate) struct FeatureView<'p> {
    title: &'p str,
    description: Option<&'p str>,
    icon: Option<&'p str>,
}

impl<'p> FeatureView<'p> {
    fn all_of(entry: &'p ProjectEntry) -> Vec<Self> {
        entry
            .project
            .features
            .iter()
            .map(|feature| Self {
                title: &feature.title,
                description: feature.description.as_deref(),
                icon: feature.icon.as_deref(),
            })
            .collect()
    }
}

/// One task, and where this reader stands on it.
///
/// The progress half is `None` for an anonymous caller rather than zeroed: a
/// signed-out reader has no attempts, which is a different statement from
/// "nought attempts", and luxctl prints them differently.
#[derive(Debug, Serialize)]
pub(crate) struct TaskView<'p> {
    id: Option<Uuid>,
    slug: &'p str,
    title: &'p str,
    sort_order: i32,
    input_type: InputType,
    /// The scoring ladder, verbatim. luxctl shows a reader what the next tier
    /// is worth; the api is what decides what they actually earned.
    scores: Option<&'p str>,
    points: i32,
    abandoned_deduction: i32,
    is_free: bool,
    /// The phase description, as markdown. `None` for a phase that carried
    /// none.
    description: Option<String>,
    hints: Vec<HintSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<TaskProgressView>,
}

impl<'p> TaskView<'p> {
    pub(crate) fn of(
        held: &'p TaskEntry,
        description: Option<String>,
        hints: Vec<HintSummary>,
        progress: Option<TaskProgressView>,
    ) -> Self {
        let task = &held.task;

        Self {
            id: task.id,
            slug: &task.slug,
            title: &task.title,
            sort_order: held.sort_order,
            input_type: task.input_type,
            scores: task.scores.as_deref(),
            points: task.points,
            abandoned_deduction: task.abandoned_deduction,
            is_free: task.is_free,
            description,
            hints,
            progress,
        }
    }
}

/// A hint, without its words: that one exists, what it costs, and whether it
/// is open yet.
///
/// **No `is_unlocked` and no `text`.** This is what a project or task listing
/// carries, and neither answers "has this reader paid for it" — a listing that
/// claimed `is_unlocked: false` for a hint somebody had bought would be a lie,
/// and answering it honestly would be a second query per task. The hints
/// endpoint is where unlock state is asked and answered.
#[derive(Debug, Serialize)]
pub(crate) struct HintSummary {
    id: Option<Uuid>,
    sort_order: usize,
    points_deduction: i32,
    /// Whether the unlock criteria are met yet — see `ohara::task::Hint`.
    is_available: bool,
}

impl HintSummary {
    pub(crate) fn of(hint: &Hint, at: usize, progress: Progress) -> Self {
        Self {
            id: hint.id,
            sort_order: at,
            points_deduction: hint.points_deduction,
            is_available: hint
                .offerable(progress.minutes, progress.attempts_so_far()),
        }
    }
}

/// A hint on the hints endpoint: the summary, plus what this reader holds.
///
/// `text` is dropped rather than sent as null when it is not theirs, so a
/// client cannot print an empty hint by forgetting to check a flag.
#[derive(Debug, Serialize)]
pub(crate) struct HintDetail {
    #[serde(flatten)]
    pub(crate) summary: HintSummary,
    pub(crate) is_unlocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<String>,
}

/// Where one reader stands on one task.
#[derive(Debug, Serialize)]
pub(crate) struct TaskProgressView {
    status: Status,
    attempts: i64,
    points_earned: i32,
    /// The sequential lock: the task before this one is not done. Always false
    /// in an `open` project — see [`UnlockMode`].
    is_locked: bool,
    /// The paywall: the task is not free and the reader does not hold the
    /// project's related book.
    is_paid: bool,
    #[serde(serialize_with = "crate::response::as_utc")]
    started_at: Option<NaiveDateTime>,
    #[serde(serialize_with = "crate::response::as_utc")]
    completed_at: Option<NaiveDateTime>,
}

impl TaskProgressView {
    pub(crate) fn of(
        progress: Progress,
        is_locked: bool,
        is_paid: bool,
    ) -> Self {
        Self {
            status: progress.status(),
            attempts: progress.attempts,
            points_earned: progress.points(),
            is_locked,
            is_paid,
            started_at: progress.started_at,
            completed_at: progress.completed_at,
        }
    }
}

/// What `GET /api/v1/user` answers.
#[derive(Debug, Serialize)]
pub(crate) struct ReaderView {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) stats: StatsView,
}

/// A reader's career totals — see `store::totals` for what each counts.
#[derive(Debug, Serialize)]
pub(crate) struct StatsView {
    pub(crate) projects_attempted: i64,
    pub(crate) tasks_completed: i64,
    pub(crate) total_xp: i64,
}

/// What comes back from recording an attempt.
#[derive(Debug, Serialize)]
pub(crate) struct AttemptView {
    pub(crate) id: i64,
    pub(crate) task_id: Uuid,
    pub(crate) run: i16,
    pub(crate) outcome: super::progress::Outcome,
    /// What this attempt earned. Zero for anything but a first pass, which is
    /// the answer rather than an omission: re-passing a task banks nothing new.
    pub(crate) points_achieved: i32,
    /// Whether the reader had already passed this task before today.
    pub(crate) is_reattempt: bool,
    #[serde(serialize_with = "crate::response::as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
}

/// What comes back from restarting a project.
#[derive(Debug, Serialize)]
pub(crate) struct RestartView {
    /// The run the reader is now on. Replaces laravel's `attempt_group_id`,
    /// which was a row id the client had no use for.
    pub(crate) run: i16,
    #[serde(serialize_with = "crate::response::as_utc")]
    pub(crate) restarted_at: Option<NaiveDateTime>,
}

/// A project as the website renders it.
///
/// Not [`ProjectDetail`]: no blueprint, and no hints. A browser cannot run a
/// `.bp` and has no use for 46 KB of one, and a hint that is bought from a
/// terminal has nothing to say on a marketing page. What is here is what a
/// reader deciding whether to start reads.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectPage<'p> {
    #[serde(flatten)]
    summary: ProjectSummary<'p>,
    long_description: Option<&'p str>,
    features: Vec<FeatureView<'p>>,
    overview: Option<String>,
    tasks: Vec<TaskListing<'p>>,
}

impl<'p> ProjectPage<'p> {
    pub(crate) fn of(
        entry: &'p ProjectEntry,
        overview: Option<String>,
    ) -> Self {
        Self {
            summary: ProjectSummary::of(entry),
            long_description: entry.project.long_description.as_deref(),
            features: FeatureView::all_of(entry),
            overview,
            tasks: entry.tasks().map(TaskListing::of).collect(),
        }
    }
}

/// A task in a contents list: enough to render the row and nothing more.
///
/// No prose and no hints. The page lists what the project asks of a reader;
/// the work itself happens in their terminal.
#[derive(Debug, Serialize)]
pub(crate) struct TaskListing<'p> {
    slug: &'p str,
    title: &'p str,
    sort_order: i32,
    points: i32,
    is_free: bool,
}

impl<'p> TaskListing<'p> {
    fn of(held: &'p TaskEntry) -> Self {
        Self {
            slug: &held.task.slug,
            title: &held.task.title,
            sort_order: held.sort_order,
            points: held.task.points,
            is_free: held.task.is_free,
        }
    }
}

/// What the project page polls for while a reader works in their terminal.
///
/// Deliberately small: this is fetched every five seconds by every open tab,
/// so it carries the numbers that change and nothing that does not. Titles,
/// descriptions and ordering came with the page and have not moved.
#[derive(Debug, Serialize)]
pub(crate) struct ProgressView {
    /// The run the reader is on. A tab that sees this go up knows the reader
    /// restarted somewhere else, and can reload rather than animate every task
    /// backwards.
    pub(crate) run: i16,
    pub(crate) completed: usize,
    pub(crate) total: usize,
    pub(crate) points_earned: i32,
    pub(crate) tasks: Vec<TaskProgressRow>,
}

/// One task's row in a poll.
#[derive(Debug, Serialize)]
pub(crate) struct TaskProgressRow {
    pub(crate) slug: String,
    #[serde(flatten)]
    pub(crate) progress: TaskProgressView,
}

/// One task as the website renders it: the brief, and how to get around.
///
/// The prose is html here and markdown on the luxctl route. The two readers
/// are a browser and a terminal, and each wants what it can draw — rendering
/// once on the way out beats every client carrying a markdown parser.
///
/// **Not gated.** A brief describes what to build; what is sold is the
/// validation and the hints. The laravel handler served descriptions to
/// everybody too, and a paywall on the statement of the problem would only
/// stop people deciding whether they want it.
#[derive(Debug, Serialize)]
pub(crate) struct TaskPage<'p> {
    slug: &'p str,
    title: &'p str,
    sort_order: i32,
    points: i32,
    is_free: bool,
    /// The brief, rendered. Empty for a phase that carried no description.
    html: String,
    /// Which task of the project this is, counting from one.
    position: usize,
    total: usize,
    project: ProjectRef<'p>,
    previous: Option<TaskRef<'p>>,
    next: Option<TaskRef<'p>>,
}

impl<'p> TaskPage<'p> {
    pub(crate) fn of(
        entry: &'p ProjectEntry,
        held: &'p TaskEntry,
        markdown: Option<&str>,
    ) -> Self {
        let ordered: Vec<&TaskEntry> = entry.tasks().collect();
        let at = entry.position_of(&held.task.slug).unwrap_or(1);

        Self {
            slug: &held.task.slug,
            title: &held.task.title,
            sort_order: held.sort_order,
            points: held.task.points,
            is_free: held.task.is_free,
            html: markdown.map(ohara::body::render).unwrap_or_default(),
            position: at,
            total: ordered.len(),
            project: ProjectRef {
                slug: &entry.project.slug,
                name: &entry.project.name,
            },
            previous: at
                .checked_sub(2)
                .and_then(|before| ordered.get(before))
                .map(|held| TaskRef::of(held)),
            next: ordered.get(at).map(|held| TaskRef::of(held)),
        }
    }
}

/// Enough of the project to render a breadcrumb without a second request.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectRef<'p> {
    slug: &'p str,
    name: &'p str,
}

/// Enough of a neighbouring task to render the link to it.
#[derive(Debug, Serialize)]
pub(crate) struct TaskRef<'p> {
    slug: &'p str,
    title: &'p str,
}

impl<'p> TaskRef<'p> {
    fn of(held: &'p TaskEntry) -> Self {
        Self {
            slug: &held.task.slug,
            title: &held.task.title,
        }
    }
}
