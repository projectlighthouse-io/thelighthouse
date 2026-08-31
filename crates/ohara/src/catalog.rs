//! Every published book, lesson, project and task, parsed once and held in
//! memory.
//!
//! Reading a lesson's *metadata* should not touch the disk, and listing books
//! should not touch it 20 times. Roughly 1.5 KB per lesson, so a catalogue of
//! 2,000 lessons is about 3 MB — small enough that holding all of it costs less
//! than deciding which parts to hold.
//!
//! **Bodies are not here.** They stay on disk and are read per request. The
//! kernel's page cache already keeps hot files in memory, with eviction nobody
//! has to write and no second copy to invalidate; 2,000 markdown bodies would
//! be another ~30 MB to manage badly. A blueprint is the same and more so: one
//! `.bp` runs to 46 KB, and it is read when luxctl asks for it, not before.
//!
//! **Drafts are not here either.** A snapshot holds only what is published, so
//! there is no filtering left to forget at the point a lesson is served.
//!
//! **Books and projects are walked side by side, under those same two rules.**
//! They share nothing else. A project has no chapters, no locales and no
//! paywall, and a task carries no status of its own — the project being a
//! draft is the whole of what keeps its tasks out of the snapshot, which is
//! why there is no per-task check to forget.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use uuid::Uuid;

use super::{
    Content, Drafts, Error, Locale, body::Body, book::Book, lesson::Folder,
    lesson::Lesson, project::Project, task::Task,
};

/// Every published book, lesson, project and task, as of one moment.
///
/// Immutable. Reloading builds a new one rather than mutating this, so a
/// request holding an `Arc` to it keeps reading a consistent view even while
/// the next one is being built.
#[derive(Debug)]
pub struct Snapshot {
    books: HashMap<String, BookEntry>,
    /// Book slugs, in the order [`Content::book_slugs`] found them.
    order: Vec<String>,
    projects: HashMap<String, ProjectEntry>,
    /// Project slugs, in the order [`Content::project_slugs`] found them.
    project_order: Vec<String>,
}

/// A book and its lessons, already in reading order.
#[derive(Debug)]
pub struct BookEntry {
    pub book: Book,
    lessons: HashMap<String, LessonEntry>,
    /// Lesson slugs, chapter order then position within the chapter.
    reading_order: Vec<String>,
}

/// A lesson, and the two things about it that are not in its yaml.
#[derive(Debug)]
pub struct LessonEntry {
    pub lesson: Lesson,
    /// The folder on disk — `07-borrowing`. A url carries the slug, the
    /// filesystem needs the number, and this is the only place the two meet.
    pub folder: String,
    pub sort_order: i32,
    pub chapter_id: i32,
    /// Whether an unentitled reader is missing anything here.
    ///
    /// Computed once, while the snapshot is built, so a table of contents can
    /// mark its paid lessons without opening a single file. The body itself is
    /// still not held — this is one bool per lesson, not the prose.
    ///
    /// False when the markdown cannot be read. A lesson whose file is missing
    /// 404s when someone asks for it, which is the honest answer; refusing the
    /// whole catalogue over it would turn one broken lesson into a site with
    /// no content.
    pub has_paid_part: bool,
}

/// A project and its tasks, already in the order they are worked through.
#[derive(Debug)]
pub struct ProjectEntry {
    pub project: Project,
    tasks: HashMap<String, TaskEntry>,
    /// Task slugs, in folder order — which is the only order there is. See
    /// [`Content::task_folders`].
    task_order: Vec<String>,
}

/// A task, and the two things about it that are not in its yaml.
///
/// The mirror of [`LessonEntry`], minus everything a task does not have: no
/// chapter, because a project has none, and no `has_paid_part`, because a task
/// is withheld whole or not at all.
#[derive(Debug)]
pub struct TaskEntry {
    pub task: Task,
    /// The folder on disk — `01-listen-on-port`. A url carries the slug, the
    /// filesystem needs the number, and this is the only place the two meet.
    pub folder: String,
    pub sort_order: i32,
}

impl Snapshot {
    /// Walks the whole content repo and parses it.
    ///
    /// # Errors
    ///
    /// Anything [`Content::book`] or [`Content::lesson`] refuses, and a lesson
    /// a chapter lists that is not on disk. A folder on disk that no chapter
    /// lists is *not* an error — that is an unpublished draft. Two books or
    /// two lessons claiming the same id is also an error, for the reason
    /// [`unique_ids`] gives.
    pub fn load(content: &Content, drafts: Drafts) -> Result<Self, Error> {
        let mut books = HashMap::new();
        let mut order = Vec::new();

        for slug in content.book_slugs()? {
            let book = content.book(&slug)?;

            if !drafts.admits(book.status) {
                continue;
            }

            books.insert(
                slug.clone(),
                BookEntry::load(content, book, &slug, drafts)?,
            );
            order.push(slug);
        }

        let mut projects = HashMap::new();
        let mut project_order = Vec::new();

        for slug in content.project_slugs()? {
            let project = content.project(&slug)?;

            // The only draft check there is. A task has no status, so a draft
            // project is the whole of what keeps its tasks unpublished — and
            // skipping here is what stops them leaking, because nothing below
            // this line ever sees the project again.
            if !drafts.admits(project.status) {
                continue;
            }

            projects.insert(
                slug.clone(),
                ProjectEntry::load(content, project, &slug)?,
            );
            project_order.push(slug);
        }

        let snapshot = Self {
            books,
            order,
            projects,
            project_order,
        };
        snapshot.unique_ids()?;

        Ok(snapshot)
    }

    /// Refuses a repo where two things of the same kind claim one id.
    ///
    /// Five kinds now: books, lessons, projects, tasks and hints. Each is a
    /// primary key of its own table, so ids collide *within* a kind and never
    /// across one — a task and a hint sharing a uuid is a coincidence, not a
    /// mistake, and refusing it would be refusing a legal repo.
    ///
    /// The ids are hand written, one per file, and unique across the whole
    /// repo rather than within a book or a project. Copying a folder to start
    /// the next one and forgetting to change the id is the obvious way to
    /// break that, and it is not visible in either file on its own.
    ///
    /// The database would catch it eventually, as a primary key violation
    /// during a sync. This catches it in `make content-check`, which is the
    /// gate that runs before a deploy, and names both files instead of one id.
    fn unique_ids(&self) -> Result<(), Error> {
        self.unique_book_ids()?;
        self.unique_project_ids()
    }

    /// Books and lessons — see [`Self::unique_ids`].
    fn unique_book_ids(&self) -> Result<(), Error> {
        let mut books = HashMap::new();
        let mut lessons = HashMap::new();

        for entry in self.books() {
            if let Some(id) = entry.book.id
                && let Some(seen) = books.insert(id, &entry.book.slug)
            {
                return Err(Error::Malformed {
                    path: PathBuf::from(format!(
                        "books/{}/book.yaml",
                        entry.book.slug
                    )),
                    cause: format!("id {id} is already used by {seen}"),
                });
            }

            for lesson in
                entry.reading_order.iter().filter_map(|s| entry.lesson(s))
            {
                if let Some(id) = lesson.lesson.id
                    && let Some(seen) =
                        lessons.insert(id, lesson.lesson.slug.clone())
                {
                    return Err(Error::Malformed {
                        path: PathBuf::from(format!(
                            "books/{}/lessons/{}/lesson.yaml",
                            entry.book.slug, lesson.folder
                        )),
                        cause: format!("id {id} is already used by {seen}"),
                    });
                }
            }
        }

        Ok(())
    }

    /// Projects, tasks and hints — see [`Self::unique_ids`].
    ///
    /// A hint is the third kind and the easy one to forget: it has no file of
    /// its own, so a duplicate is two lines of one `task.yaml` rather than two
    /// files, and copying a hint block to write the next one leaves the id
    /// behind. `user_unlocked_hints.task_hint_id` points at it, so a collision
    /// is a reader's paid unlock naming the wrong hint.
    fn unique_project_ids(&self) -> Result<(), Error> {
        let mut projects = HashMap::new();
        let mut tasks = HashMap::new();
        let mut hints = HashMap::new();

        for entry in self.projects() {
            let slug = &entry.project.slug;
            let path =
                || PathBuf::from(format!("projects/{slug}/project.yaml"));

            if let Some(id) = entry.project.id
                && let Some(seen) = projects.insert(id, slug.clone())
            {
                return Err(Error::Malformed {
                    path: path(),
                    cause: format!("id {id} is already used by {seen}"),
                });
            }

            for held in entry.tasks() {
                let path = || {
                    PathBuf::from(format!(
                        "projects/{slug}/tasks/{}/task.yaml",
                        held.folder
                    ))
                };

                if let Some(id) = held.task.id
                    && let Some(seen) = tasks.insert(id, held.task.slug.clone())
                {
                    return Err(Error::Malformed {
                        path: path(),
                        cause: format!("id {id} is already used by {seen}"),
                    });
                }

                for hint in &held.task.hints {
                    if let Some(id) = hint.id
                        && let Some(seen) =
                            hints.insert(id, held.task.slug.clone())
                    {
                        return Err(Error::Malformed {
                            path: path(),
                            cause: format!(
                                "hint id {id} is already used by a hint in \
                                 {seen}"
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Published books, in directory order.
    pub fn books(&self) -> impl Iterator<Item = &BookEntry> {
        self.order.iter().filter_map(|slug| self.books.get(slug))
    }

    #[must_use]
    pub fn book(&self, slug: &str) -> Option<&BookEntry> {
        self.books.get(slug)
    }

    #[must_use]
    pub fn lesson(&self, book: &str, lesson: &str) -> Option<&LessonEntry> {
        self.books.get(book)?.lesson(lesson)
    }

    /// Published projects, in directory order.
    pub fn projects(&self) -> impl Iterator<Item = &ProjectEntry> {
        self.project_order
            .iter()
            .filter_map(|slug| self.projects.get(slug))
    }

    #[must_use]
    pub fn project(&self, slug: &str) -> Option<&ProjectEntry> {
        self.projects.get(slug)
    }

    /// The project a uuid or a slug names.
    ///
    /// luxctl sends either, and which one it sent is not worth branching on
    /// twice — the id is looked up first because it is the exact answer, and
    /// the slug only if nothing claimed the id.
    #[must_use]
    pub fn project_by(&self, identifier: &str) -> Option<&ProjectEntry> {
        if let Ok(id) = identifier.parse::<Uuid>() {
            return self.projects().find(|entry| entry.project.id == Some(id));
        }

        self.project(identifier)
    }

    #[must_use]
    pub fn task(&self, project: &str, task: &str) -> Option<&TaskEntry> {
        self.projects.get(project)?.task(task)
    }

    /// The task a uuid or a slug names, and the project it belongs to.
    ///
    /// **A slug can be ambiguous and a uuid cannot.** Task slugs are unique
    /// within a project, not across the repo — `listen-on-port` opens more
    /// than one project — so a slug resolves to the first match in directory
    /// order. That is what the laravel handler did with `->first()`, kept
    /// deliberately rather than fixed here: making it an error would break
    /// every `lux task listen-on-port` that works today, and the uuid is the
    /// unambiguous form luxctl already has from the project listing.
    #[must_use]
    pub fn task_by(
        &self,
        identifier: &str,
    ) -> Option<(&ProjectEntry, &TaskEntry)> {
        if let Ok(id) = identifier.parse::<Uuid>() {
            return self.projects().find_map(|entry| {
                let task =
                    entry.tasks().find(|held| held.task.id == Some(id))?;

                Some((entry, task))
            });
        }

        self.projects()
            .find_map(|entry| Some((entry, entry.task(identifier)?)))
    }
}

impl BookEntry {
    fn load(
        content: &Content,
        book: Book,
        slug: &str,
        drafts: Drafts,
    ) -> Result<Self, Error> {
        let mut lessons = HashMap::new();
        let mut reading_order = Vec::new();

        // Walked by chapter rather than through `Book::lesson_folders`, so the
        // chapter's id comes from the chapter that listed the lesson instead of
        // being looked up again afterwards.
        //
        // The chapters decide the order, not the directory listing. A folder
        // here that is not on disk is a mistake worth stopping for; one on disk
        // that is not here is a draft, and is simply absent.
        for chapter in &book.chapters {
            for folder in &chapter.lessons {
                let lesson = content.lesson(slug, folder)?;

                if !drafts.admits(lesson.status) {
                    continue;
                }

                let named = Folder::parse(folder).map_err(|cause| {
                    Error::Malformed {
                        path: content.lesson_dir(slug, folder),
                        cause,
                    }
                })?;

                // Read once here rather than per request: the alternative is
                // a book page opening every lesson file to render one list.
                let has_paid_part = content
                    .body(
                        slug,
                        folder,
                        lesson.body_file(Locale::En),
                        lesson.access,
                    )
                    .is_ok_and(|body| body.has_paid_part());

                reading_order.push(named.slug.clone());
                lessons.insert(
                    named.slug,
                    LessonEntry {
                        lesson,
                        folder: folder.clone(),
                        sort_order: named.sort_order,
                        chapter_id: chapter.id,
                        has_paid_part,
                    },
                );
            }
        }

        Ok(Self {
            book,
            lessons,
            reading_order,
        })
    }

    /// Published lessons, in reading order.
    pub fn lessons(&self) -> impl Iterator<Item = &LessonEntry> {
        self.reading_order
            .iter()
            .filter_map(|slug| self.lessons.get(slug))
    }

    #[must_use]
    pub fn lesson(&self, slug: &str) -> Option<&LessonEntry> {
        self.lessons.get(slug)
    }

    /// Where "start reading" goes — the first published lesson.
    #[must_use]
    pub fn first_lesson(&self) -> Option<&LessonEntry> {
        self.lessons().next()
    }

    /// The lessons either side of this one, in reading order.
    ///
    /// Both `None` for a lesson that is not in this book, which is the same
    /// answer a one-lesson book gives — the caller has already established the
    /// lesson exists before it wants to know what is next to it.
    pub fn neighbours(&self, slug: &str) -> (Option<&str>, Option<&str>) {
        let Some(at) = self.reading_order.iter().position(|s| s == slug) else {
            return (None, None);
        };

        let previous = at
            .checked_sub(1)
            .and_then(|before| self.reading_order.get(before));

        (
            previous.map(String::as_str),
            self.reading_order.get(at + 1).map(String::as_str),
        )
    }
}

impl ProjectEntry {
    /// Walks `tasks/`, which is the whole list and the order at once.
    ///
    /// Unlike [`BookEntry::load`] there is nothing to walk *by*: a book names
    /// every lesson folder it publishes and a folder it does not name is a
    /// draft, while a project's yaml names no tasks at all. So the directory
    /// is the answer, and a task folder cannot be an unpublished draft — it is
    /// in the project or it is not on disk.
    fn load(
        content: &Content,
        project: Project,
        slug: &str,
    ) -> Result<Self, Error> {
        let mut tasks = HashMap::new();
        let mut task_order = Vec::new();

        for folder in content.task_folders(slug)? {
            let task = content.task(slug, &folder)?;

            // Parsed again rather than carried out of `Content::task`, which
            // checks the slug against it and drops it. One line, and the
            // alternative is a second return value every caller has to thread.
            let named =
                Folder::parse(&folder).map_err(|cause| Error::Malformed {
                    path: content.task_dir(slug, &folder),
                    cause,
                })?;

            task_order.push(named.slug.clone());
            tasks.insert(
                named.slug,
                TaskEntry {
                    task,
                    folder,
                    sort_order: named.sort_order,
                },
            );
        }

        Ok(Self {
            project,
            tasks,
            task_order,
        })
    }

    /// The project's tasks, in the order they are worked through.
    pub fn tasks(&self) -> impl Iterator<Item = &TaskEntry> {
        self.task_order
            .iter()
            .filter_map(|slug| self.tasks.get(slug))
    }

    #[must_use]
    pub fn task(&self, slug: &str) -> Option<&TaskEntry> {
        self.tasks.get(slug)
    }

    /// Which task of the project this is, counting from one.
    ///
    /// `None` for a task that is not in this project, which is not the same
    /// answer as "the first one" — a caller that cannot place a task must not
    /// go on to decide whether the one before it was completed.
    #[must_use]
    pub fn position_of(&self, slug: &str) -> Option<usize> {
        self.task_order
            .iter()
            .position(|other| other == slug)
            .map(|at| at + 1)
    }

    /// The task before this one, or `None` for the first.
    ///
    /// What the sequential unlock is decided against — see
    /// [`super::project::UnlockMode`]. Both `None` for a task that is not in
    /// this project, which the caller has already established exists.
    #[must_use]
    pub fn task_before(&self, slug: &str) -> Option<&TaskEntry> {
        let at = self.task_order.iter().position(|other| other == slug)?;

        self.task(self.task_order.get(at.checked_sub(1)?)?)
    }
}

/// The catalogue in front of the content repo, and the ability to rebuild it.
///
/// One of these lives for the life of the process. Handlers take a cheap `Arc`
/// clone of the current [`Snapshot`] and let go of the lock immediately, so a
/// reload never blocks a read for longer than a pointer swap.
#[derive(Debug)]
pub struct Catalog {
    content: Content,
    /// Held so a reload applies the same policy the first load did. Reading it
    /// again from the environment would let a SIGHUP quietly change what the
    /// site serves.
    drafts: Drafts,
    current: RwLock<Arc<Snapshot>>,
}

impl Catalog {
    /// Reads the content repo and holds the result.
    ///
    /// # Errors
    ///
    /// As [`Snapshot::load`]. Failing here should stop the process: there is no
    /// previous snapshot to fall back on, and a site that boots with no content
    /// looks broken rather than down.
    pub fn load(content: Content, drafts: Drafts) -> Result<Self, Error> {
        let snapshot = Snapshot::load(&content, drafts)?;

        Ok(Self {
            content,
            drafts,
            current: RwLock::new(Arc::new(snapshot)),
        })
    }

    /// The snapshot as it is right now.
    pub fn current(&self) -> Arc<Snapshot> {
        match self.current.read() {
            Ok(current) => Arc::clone(&current),
            // A writer panicked while holding the lock. The data behind it is
            // an immutable `Arc<Snapshot>` that was fully built before it was
            // stored, so it cannot be half-written — recovering beats taking
            // the site down over a lock's bookkeeping.
            Err(poisoned) => Arc::clone(&poisoned.into_inner()),
        }
    }

    /// Rereads the content repo, replacing the snapshot only if it parses.
    ///
    /// This is the runtime sync: no restart, and in-flight requests finish
    /// against the snapshot they already hold.
    ///
    /// # Errors
    ///
    /// As [`Snapshot::load`] — and on error the previous snapshot stays in
    /// place, so a typo in one lesson does not empty the site.
    pub fn reload(&self) -> Result<(), Error> {
        let rebuilt = Arc::new(Snapshot::load(&self.content, self.drafts)?);

        match self.current.write() {
            Ok(mut current) => *current = rebuilt,
            Err(poisoned) => *poisoned.into_inner() = rebuilt,
        }

        Ok(())
    }

    /// A published lesson's prose in `locale`, split at the paywall.
    ///
    /// `None` when no such published lesson exists — which is the same answer
    /// for a draft, a typo and a deleted lesson, because a reader is owed the
    /// same 404 for all three.
    ///
    /// A language this lesson is not written in is *not* one of those cases:
    /// it falls back to English rather than 404ing, so a reader whose language
    /// is only half translated reads the book instead of hitting holes.
    ///
    /// # Errors
    ///
    /// The lesson is in the catalogue but its markdown cannot be read — which,
    /// with a `content_path` naming a file nobody wrote, is the mistake this
    /// reports rather than silently serving English.
    pub fn body(
        &self,
        book: &str,
        lesson: &str,
        locale: Locale,
    ) -> Result<Option<Body>, Error> {
        // The folder and the file are both taken while the snapshot is held,
        // so a reload between the two cannot pair one lesson's folder with
        // another's filename.
        let Some((folder, file, access)) =
            self.current().lesson(book, lesson).map(|entry| {
                (
                    entry.folder.clone(),
                    entry.lesson.body_file(locale).to_owned(),
                    entry.lesson.access,
                )
            })
        else {
            return Ok(None);
        };

        self.content.body(book, &folder, &file, access).map(Some)
    }

    /// A published project's blueprint, whole.
    ///
    /// Read per call rather than held, for the reason at the top of this file:
    /// one `.bp` is 46 KB and the only caller is luxctl fetching a project it
    /// is about to work through. `None` when no such published project exists
    /// — a draft, a typo and a deleted project all get the same 404.
    ///
    /// # Errors
    ///
    /// The project is in the catalogue but the file its `blueprint_path` names
    /// cannot be read. Not silently an empty blueprint: luxctl would run a
    /// project with no phases and report every task passing.
    pub fn blueprint(&self, project: &str) -> Result<Option<String>, Error> {
        let Some(named) = self.current().project(project).map(|entry| {
            // The whole `Project` rather than a borrow, so the snapshot is let
            // go before the file is opened and a reload cannot pair one
            // project's slug with another's path.
            (
                entry.project.slug.clone(),
                entry.project.blueprint_path.clone(),
            )
        }) else {
            return Ok(None);
        };

        self.content.blueprint_at(&named.0, &named.1).map(Some)
    }

    /// A published task's prose. `None` for no such task *or* a task that
    /// carries none — a phase with no description is ordinary.
    ///
    /// # Errors
    ///
    /// The task names a `content_path` and the file is not there.
    pub fn task_body(
        &self,
        project: &str,
        task: &str,
    ) -> Result<Option<String>, Error> {
        let Some((folder, file)) =
            self.current().task(project, task).and_then(|held| {
                Some((
                    held.folder.clone(),
                    held.task.content_path.as_ref()?.en.clone(),
                ))
            })
        else {
            return Ok(None);
        };

        self.content.task_body_at(project, &folder, &file).map(Some)
    }

    /// A published project's long-form markdown, if it has one.
    ///
    /// # Errors
    ///
    /// As [`Self::task_body`].
    pub fn overview(&self, project: &str) -> Result<Option<String>, Error> {
        let Some(file) = self.current().project(project).and_then(|entry| {
            Some(entry.project.content_path.as_ref()?.en.clone())
        }) else {
            return Ok(None);
        };

        self.content.overview_at(project, &file).map(Some)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn two_lessons_claiming_one_id_are_refused_by_name() {
        // The mistake this exists for: a lesson folder copied to start the
        // next one, with the id left as it was. Neither file is wrong on its
        // own, so nothing but a whole-repo pass can see it.
        let error = Snapshot::load(&fixture::duplicate_ids(), Drafts::Hidden)
            .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("lesson.yaml"), "{message}");
        assert!(message.contains("already used by"), "{message}");
    }
    use std::{fs, path::PathBuf};

    use super::super::{Status, fixture};
    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot::load(&fixture::content(), Drafts::Hidden).unwrap()
    }

    /// A throwaway copy of the fixture that a test may break on purpose.
    ///
    /// Named after the test so two running at once cannot collide, and left
    /// behind in `/tmp` if a test fails — which is where you would want it.
    fn scratch_copy(name: &str) -> PathBuf {
        fn copy_dir(from: &PathBuf, to: &PathBuf) {
            fs::create_dir_all(to).unwrap();
            for entry in fs::read_dir(from).unwrap().filter_map(Result::ok) {
                let target = to.join(entry.file_name());
                if entry.path().is_dir() {
                    copy_dir(&entry.path(), &target);
                } else {
                    fs::copy(entry.path(), target).unwrap();
                }
            }
        }

        let root = std::env::temp_dir().join(format!("ohara-{name}"));
        let _ = fs::remove_dir_all(&root);

        let source = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixture"
        ));
        copy_dir(&source, &root);

        root
    }

    #[test]
    fn a_snapshot_holds_every_published_book() {
        let snapshot = snapshot();

        assert_eq!(snapshot.books().count(), 1);
        assert!(snapshot.book("fixture-book").is_some());
    }

    #[test]
    fn one_bad_book_refuses_the_whole_catalogue() {
        // All or nothing, deliberately. Loading the rest would leave a book
        // silently missing from the site, which nobody notices until a reader
        // does; refusing is loud, and the deploy is where it gets caught.
        let error =
            Snapshot::load(&fixture::broken(), Drafts::Hidden).unwrap_err();

        assert!(error.to_string().contains("mislabelled"), "{error}");
    }

    #[test]
    fn lessons_are_keyed_by_slug_but_remember_their_folder() {
        let snapshot = snapshot();
        let entry = snapshot.lesson("fixture-book", "free-lesson").unwrap();

        // The url will carry `free-lesson`; the disk needs `01-free-lesson`.
        assert_eq!(entry.folder, "01-free-lesson");
        assert_eq!(entry.sort_order, 1);
        assert_eq!(entry.chapter_id, 1);
    }

    #[test]
    fn lessons_come_back_in_reading_order() {
        let snapshot = snapshot();
        let book = snapshot.book("fixture-book").unwrap();

        assert_eq!(book.lessons().count(), 2);
        assert_eq!(book.first_lesson().unwrap().lesson.slug, "free-lesson");
    }

    #[test]
    fn a_published_book_still_drops_its_draft_lessons() {
        // The case a book-level check would miss: `fixture-book` is published
        // and one of its three lessons is not. Status is per lesson, so the
        // book being live says nothing about the lesson.
        let snapshot =
            Snapshot::load(&fixture::content(), Drafts::Hidden).unwrap();
        let book = snapshot.book("fixture-book").unwrap();

        assert_eq!(book.book.status, Status::Published);
        assert_eq!(book.lessons().count(), 2);
        assert!(book.lesson("draft-lesson").is_none());
        // And it is absent from reading order, not merely unlisted — so
        // `neighbours` cannot walk into it either.
        assert_eq!(
            book.neighbours("split-lesson"),
            (Some("free-lesson"), None)
        );
    }

    #[test]
    fn a_local_run_keeps_the_draft_lesson_beside_the_published_ones() {
        let snapshot =
            Snapshot::load(&fixture::content(), Drafts::Shown).unwrap();
        let book = snapshot.book("fixture-book").unwrap();

        assert_eq!(book.lessons().count(), 3);

        let draft = book.lesson("draft-lesson").unwrap();
        assert_eq!(draft.lesson.status, Status::Draft);
        // In reading order where it belongs, so it can be read while written.
        assert_eq!(
            book.neighbours("split-lesson"),
            (Some("free-lesson"), Some("draft-lesson"))
        );
    }

    #[test]
    fn a_draft_book_is_absent_in_production_and_present_locally() {
        // The other axis, so the two cannot be confused: whole-book status.
        let hidden =
            Snapshot::load(&fixture::drafts(), Drafts::Hidden).unwrap();
        let shown = Snapshot::load(&fixture::drafts(), Drafts::Shown).unwrap();

        assert_eq!(hidden.books().count(), 0);
        assert_eq!(shown.books().count(), 1);
    }

    #[test]
    fn a_drafts_body_is_only_readable_when_drafts_are_shown() {
        let production =
            Catalog::load(fixture::content(), Drafts::Hidden).unwrap();
        let local = Catalog::load(fixture::content(), Drafts::Shown).unwrap();

        // Not a different response for the same lesson — no lesson at all.
        assert!(
            production
                .body("fixture-book", "draft-lesson", Locale::En)
                .unwrap()
                .is_none()
        );
        assert!(
            local
                .body("fixture-book", "draft-lesson", Locale::En)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn a_body_is_read_from_disk_rather_than_held() {
        let catalog =
            Catalog::load(fixture::content(), Drafts::Hidden).unwrap();
        let body = catalog
            .body("fixture-book", "split-lesson", Locale::En)
            .unwrap();

        assert!(body.unwrap().has_paid_part());
    }

    #[test]
    fn an_unknown_lesson_is_none_rather_than_an_error() {
        let catalog =
            Catalog::load(fixture::content(), Drafts::Hidden).unwrap();

        assert!(
            catalog
                .body("fixture-book", "nope", Locale::En)
                .unwrap()
                .is_none()
        );
        assert!(
            catalog
                .body("nope", "free-lesson", Locale::En)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_snapshot_holds_a_project_and_its_tasks_in_folder_order() {
        let snapshot = snapshot();
        let project = snapshot.project("fixture-project").unwrap();

        assert_eq!(snapshot.projects().count(), 1);
        assert_eq!(
            project
                .tasks()
                .map(|t| t.task.slug.as_str())
                .collect::<Vec<_>>(),
            ["listen-on-port", "say-something"]
        );

        let first = project.task("listen-on-port").unwrap();
        // The url will carry `listen-on-port`; the disk needs `01-...`.
        assert_eq!(first.folder, "01-listen-on-port");
        assert_eq!(first.sort_order, 1);
        assert_eq!(project.position_of("say-something"), Some(2));
        assert!(project.task_before("listen-on-port").is_none());
        assert_eq!(
            project.task_before("say-something").unwrap().task.slug,
            "listen-on-port"
        );
    }

    #[test]
    fn a_draft_project_takes_its_tasks_out_of_the_snapshot_with_it() {
        // The leak this exists to catch: a task has no status of its own, so
        // if the project's were checked anywhere but before the walk, every
        // task of an unpublished project would be served.
        let hidden =
            Snapshot::load(&fixture::drafts(), Drafts::Hidden).unwrap();
        let shown = Snapshot::load(&fixture::drafts(), Drafts::Shown).unwrap();

        // `pointed-at-nothing` sits beside it and is published, so this is
        // the draft one going missing rather than the walk finding nothing.
        assert_eq!(hidden.projects().count(), 1);
        assert!(hidden.project("unfinished-project").is_none());
        assert!(hidden.task("unfinished-project", "a-task").is_none());
        assert!(hidden.task_by("a-task").is_none());

        assert_eq!(shown.projects().count(), 2);
        assert!(shown.task("unfinished-project", "a-task").is_some());
    }

    #[test]
    fn a_project_and_a_task_answer_to_a_uuid_or_a_slug() {
        let snapshot = snapshot();
        let project = snapshot.project("fixture-project").unwrap();
        let id = project.project.id.unwrap();
        let task_id = project.task("say-something").unwrap().task.id.unwrap();

        assert_eq!(
            snapshot.project_by(&id.to_string()).unwrap().project.slug,
            "fixture-project"
        );
        assert_eq!(
            snapshot.project_by("fixture-project").unwrap().project.id,
            Some(id)
        );
        assert!(snapshot.project_by("no-such-project").is_none());
        // A uuid nobody claims is not then retried as a slug.
        assert!(
            snapshot
                .project_by(&uuid::Uuid::nil().to_string())
                .is_none()
        );

        let (owner, task) = snapshot.task_by(&task_id.to_string()).unwrap();
        assert_eq!(owner.project.slug, "fixture-project");
        assert_eq!(task.task.slug, "say-something");
        assert_eq!(snapshot.task_by("listen-on-port").unwrap().1.sort_order, 1);
        assert!(snapshot.task_by("no-such-task").is_none());
    }

    #[test]
    fn two_hints_claiming_one_id_are_refused_by_name() {
        // The third kind of id, and the one with no file of its own: a
        // duplicate is two lines of one task.yaml rather than two files, so
        // nothing but a whole-repo pass can see it. Built by breaking a copy
        // rather than shipping a fourth fixture root for one assertion.
        let root = scratch_copy("duplicate-hints");
        let task = root
            .join("projects/fixture-project/tasks/02-say-something/task.yaml");

        fs::write(
            &task,
            format!(
                "{}hints:\n  - id: 9c3f5b71-8e2a-4d60-b7c9-1f4a8d5e2b03\n    \
                 text: A copy of the first task's hint.\n    \
                 unlock_criteria: '5:3:A'\n",
                fs::read_to_string(&task).unwrap()
            ),
        )
        .unwrap();

        let error =
            Snapshot::load(&Content::at(&root), Drafts::Hidden).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("02-say-something"), "{message}");
        assert!(message.contains("hint id"), "{message}");
        assert!(message.contains("listen-on-port"), "{message}");
    }

    #[test]
    fn a_blueprint_and_a_task_body_are_read_from_disk_rather_than_held() {
        let catalog =
            Catalog::load(fixture::content(), Drafts::Hidden).unwrap();

        assert!(
            catalog
                .blueprint("fixture-project")
                .unwrap()
                .unwrap()
                .contains("phase \"listen\"")
        );
        assert!(
            catalog
                .task_body("fixture-project", "listen-on-port")
                .unwrap()
                .unwrap()
                .contains("bind")
        );
        assert!(catalog.overview("fixture-project").unwrap().is_some());

        // A task that names no prose, and a project that is not there: both
        // `None`, and neither is an error.
        assert!(
            catalog
                .task_body("fixture-project", "say-something")
                .unwrap()
                .is_none()
        );
        assert!(catalog.blueprint("nope").unwrap().is_none());
        assert!(
            catalog
                .task_body("nope", "listen-on-port")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_reload_picks_up_a_new_lesson() {
        let root = scratch_copy("reload-adds");
        let catalog =
            Catalog::load(Content::at(&root), Drafts::Hidden).unwrap();

        assert!(catalog.current().lesson("fixture-book", "late").is_none());

        let lessons = root.join("books/fixture-book/lessons");
        fs::create_dir_all(lessons.join("04-late")).unwrap();
        fs::write(
            lessons.join("04-late/lesson.yaml"),
            "slug: late\ntitle: A Late Lesson\nstatus: published\n\
             content_path:\n  en: lesson.md\n",
        )
        .unwrap();
        fs::write(lessons.join("04-late/lesson.md"), "Written later.\n")
            .unwrap();

        let book = root.join("books/fixture-book/book.yaml");
        let listed = fs::read_to_string(&book).unwrap().replace(
            "      - \"03-draft-lesson\"",
            "      - \"03-draft-lesson\"\n      - \"04-late\"",
        );
        fs::write(&book, listed).unwrap();

        catalog.reload().unwrap();

        assert!(catalog.current().lesson("fixture-book", "late").is_some());
    }

    #[test]
    fn a_failed_reload_keeps_the_previous_snapshot() {
        let root = scratch_copy("reload-keeps");
        let catalog =
            Catalog::load(Content::at(&root), Drafts::Hidden).unwrap();

        fs::write(root.join("books/fixture-book/book.yaml"), "slug: [\n")
            .unwrap();

        // The typo is reported...
        assert!(catalog.reload().is_err());

        // ...and the site keeps serving what it had.
        assert!(
            catalog
                .current()
                .lesson("fixture-book", "free-lesson")
                .is_some()
        );
    }
}
