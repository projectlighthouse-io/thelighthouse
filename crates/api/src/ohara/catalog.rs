//! Every published book and lesson, parsed once and held in memory.
//!
//! Reading a lesson's *metadata* should not touch the disk, and listing books
//! should not touch it 20 times. Roughly 1.5 KB per lesson, so a catalogue of
//! 2,000 lessons is about 3 MB — small enough that holding all of it costs less
//! than deciding which parts to hold.
//!
//! **Bodies are not here.** They stay on disk and are read per request. The
//! kernel's page cache already keeps hot files in memory, with eviction nobody
//! has to write and no second copy to invalidate; 2,000 markdown bodies would
//! be another ~30 MB to manage badly.
//!
//! **Drafts are not here either.** A snapshot holds only what is published, so
//! there is no filtering left to forget at the point a lesson is served.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::{
    Content, Error, Status, body::Body, book::Book, lesson::Folder,
    lesson::Lesson,
};

/// Every published book and lesson, as of one moment.
///
/// Immutable. Reloading builds a new one rather than mutating this, so a
/// request holding an `Arc` to it keeps reading a consistent view even while
/// the next one is being built.
#[derive(Debug)]
pub(crate) struct Snapshot {
    books: HashMap<String, BookEntry>,
    /// Book slugs, in the order [`Content::book_slugs`] found them.
    order: Vec<String>,
}

/// A book and its lessons, already in reading order.
#[derive(Debug)]
pub(crate) struct BookEntry {
    pub(crate) book: Book,
    lessons: HashMap<String, LessonEntry>,
    /// Lesson slugs, chapter order then position within the chapter.
    reading_order: Vec<String>,
}

/// A lesson, and the two things about it that are not in its yaml.
#[derive(Debug)]
pub(crate) struct LessonEntry {
    pub(crate) lesson: Lesson,
    /// The folder on disk — `07-borrowing`. A url carries the slug, the
    /// filesystem needs the number, and this is the only place the two meet.
    pub(crate) folder: String,
    pub(crate) sort_order: i32,
    pub(crate) chapter_id: i32,
}

impl Snapshot {
    /// Walks the whole content repo and parses it.
    ///
    /// # Errors
    ///
    /// Anything [`Content::book`] or [`Content::lesson`] refuses, and a lesson
    /// a chapter lists that is not on disk. A folder on disk that no chapter
    /// lists is *not* an error — that is an unpublished draft.
    pub(crate) fn load(content: &Content) -> Result<Self, Error> {
        let mut books = HashMap::new();
        let mut order = Vec::new();

        for slug in content.book_slugs()? {
            let book = content.book(&slug)?;

            if book.status != Status::Published {
                continue;
            }

            books.insert(slug.clone(), BookEntry::load(content, book, &slug)?);
            order.push(slug);
        }

        Ok(Self { books, order })
    }

    /// Published books, in directory order.
    pub(crate) fn books(&self) -> impl Iterator<Item = &BookEntry> {
        self.order.iter().filter_map(|slug| self.books.get(slug))
    }

    pub(crate) fn book(&self, slug: &str) -> Option<&BookEntry> {
        self.books.get(slug)
    }

    pub(crate) fn lesson(
        &self,
        book: &str,
        lesson: &str,
    ) -> Option<&LessonEntry> {
        self.books.get(book)?.lesson(lesson)
    }
}

impl BookEntry {
    fn load(content: &Content, book: Book, slug: &str) -> Result<Self, Error> {
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

                if lesson.status != Status::Published {
                    continue;
                }

                let named = Folder::parse(folder).map_err(|cause| {
                    Error::Malformed {
                        path: content.lesson_dir(slug, folder),
                        cause,
                    }
                })?;

                reading_order.push(named.slug.clone());
                lessons.insert(
                    named.slug,
                    LessonEntry {
                        lesson,
                        folder: folder.clone(),
                        sort_order: named.sort_order,
                        chapter_id: chapter.id,
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
    pub(crate) fn lessons(&self) -> impl Iterator<Item = &LessonEntry> {
        self.reading_order
            .iter()
            .filter_map(|slug| self.lessons.get(slug))
    }

    pub(crate) fn lesson(&self, slug: &str) -> Option<&LessonEntry> {
        self.lessons.get(slug)
    }

    /// Where "start reading" goes — the first published lesson.
    pub(crate) fn first_lesson(&self) -> Option<&LessonEntry> {
        self.lessons().next()
    }
}

/// The catalogue in front of the content repo, and the ability to rebuild it.
///
/// One of these lives for the life of the process. Handlers take a cheap `Arc`
/// clone of the current [`Snapshot`] and let go of the lock immediately, so a
/// reload never blocks a read for longer than a pointer swap.
#[derive(Debug)]
pub(crate) struct Catalog {
    content: Content,
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
    pub(crate) fn load(content: Content) -> Result<Self, Error> {
        let snapshot = Snapshot::load(&content)?;

        Ok(Self {
            content,
            current: RwLock::new(Arc::new(snapshot)),
        })
    }

    /// The snapshot as it is right now.
    pub(crate) fn current(&self) -> Arc<Snapshot> {
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
    pub(crate) fn reload(&self) -> Result<(), Error> {
        let rebuilt = Arc::new(Snapshot::load(&self.content)?);

        match self.current.write() {
            Ok(mut current) => *current = rebuilt,
            Err(poisoned) => *poisoned.into_inner() = rebuilt,
        }

        Ok(())
    }

    /// A published lesson's prose, split at the paywall.
    ///
    /// `None` when no such published lesson exists — which is the same answer
    /// for a draft, a typo and a deleted lesson, because a reader is owed the
    /// same 404 for all three.
    ///
    /// # Errors
    ///
    /// The lesson is in the catalogue but its markdown cannot be read.
    pub(crate) fn body(
        &self,
        book: &str,
        lesson: &str,
    ) -> Result<Option<Body>, Error> {
        let Some(folder) = self
            .current()
            .lesson(book, lesson)
            .map(|entry| entry.folder.clone())
        else {
            return Ok(None);
        };

        self.content.body(book, &folder).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::super::fixture;
    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot::load(&fixture::content()).unwrap()
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
        let error = Snapshot::load(&fixture::broken()).unwrap_err();

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
    fn a_body_is_read_from_disk_rather_than_held() {
        let catalog = Catalog::load(fixture::content()).unwrap();
        let body = catalog.body("fixture-book", "split-lesson").unwrap();

        assert!(body.unwrap().has_paid_part());
    }

    #[test]
    fn an_unknown_lesson_is_none_rather_than_an_error() {
        let catalog = Catalog::load(fixture::content()).unwrap();

        assert!(catalog.body("fixture-book", "nope").unwrap().is_none());
        assert!(catalog.body("nope", "free-lesson").unwrap().is_none());
    }

    #[test]
    fn a_reload_picks_up_a_new_lesson() {
        let root = scratch_copy("reload-adds");
        let catalog = Catalog::load(Content::at(&root)).unwrap();

        assert!(catalog.current().lesson("fixture-book", "late").is_none());

        let lessons = root.join("books/fixture-book/lessons");
        fs::create_dir_all(lessons.join("03-late")).unwrap();
        fs::write(
            lessons.join("03-late/lesson.yaml"),
            "slug: late\ntitle: A Late Lesson\nstatus: published\n",
        )
        .unwrap();
        fs::write(lessons.join("03-late/lesson.md"), "Written later.\n")
            .unwrap();

        let book = root.join("books/fixture-book/book.yaml");
        let listed = fs::read_to_string(&book).unwrap().replace(
            "      - \"02-split-lesson\"",
            "      - \"02-split-lesson\"\n      - \"03-late\"",
        );
        fs::write(&book, listed).unwrap();

        catalog.reload().unwrap();

        assert!(catalog.current().lesson("fixture-book", "late").is_some());
    }

    #[test]
    fn a_failed_reload_keeps_the_previous_snapshot() {
        let root = scratch_copy("reload-keeps");
        let catalog = Catalog::load(Content::at(&root)).unwrap();

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
