//! Ohara — the content repo: where it is, what shape it has, how it is read.
//!
//! Named after the repo it reads, `projectlighthouse-io/ohara`, which is
//! private for the reason the next paragraph but one gives.
//!
//! ```text
//!   <CONTENT_PATH>/
//!     books/
//!       build-your-own-container/
//!         book.yaml
//!         lessons/
//!           03-there-is-no-such-thing/
//!             lesson.yaml
//!             lesson.md
//!             lesson.bn.md
//!     pricing/
//!       ppp.yaml
//! ```
//!
//! **The directory names the instance, the file names the type.** Every lesson
//! is a `lesson.yaml` and the prose beside it; which lesson it is comes from
//! the folder. Renumbering a book is then one rename per lesson rather than
//! three, and a folder whose name drifts from its files is not expressible.
//!
//! **A lesson names its own prose, per language.** `content_path` maps a
//! locale to a file relative to the lesson's folder, and it is required —
//! there is no "it is always `lesson.md`" convention to fall back on. A
//! convention is a rule the files can break silently: a `lesson.bn.md` nobody
//! listed is a translation nobody is served, and a missing `lesson.md` is a
//! 404 rather than a load error. Stated, both are caught at `make
//! content-check`.
//!
//! **A lesson folder is `<sort_order>-<slug>`.** Both halves are load-bearing:
//! the number orders the book, the rest is checked against the yaml's `slug`.
//! Neither is repeated in a yaml field, so neither can disagree with itself.
//!
//! **This repo knows the structure. It never contains the content.** The prose
//! is the product, it lives in a private repo, and nothing here should tempt
//! anyone to check a copy in. Tests run against `fixture/`, which is
//! deliberately thin and deliberately fake.
//!
//! ```text
//!   mod.rs     the layout above, and walking it
//!   status.rs  the smallints `status` and `tier` store, and the draft policy
//!   book.rs    a book's yaml
//!   lesson.rs  a lesson's yaml, and its folder name
//!   locale.rs  the languages a lesson can be read in
//!   body.rs    markdown: the paywall split, and rendering both halves
//!   price.rs   what a book costs
//!   ppp.rs     what it costs somewhere poorer
//! ```

// Two crates read this one and neither reads all of it: the api never syncs and
// `lighthouse-content` never serves. A field that only one of them wants is
// still part of the file format, so it is parsed either way.
#![allow(dead_code)]

pub mod body;
pub mod book;
pub mod catalog;
pub mod lesson;
pub mod locale;
pub mod ppp;
pub mod price;
pub mod status;

use std::{
    io,
    path::{Path, PathBuf},
};

pub use locale::Locale;
pub use status::{Drafts, Status, Tier};

/// What went wrong reading the content repo.
///
/// Every variant names the path, because the first question about any of these
/// is "which file" and a bare `io::Error` does not say.
#[derive(Debug)]
pub enum Error {
    /// The path does not exist, or cannot be read.
    Unreadable { path: PathBuf, cause: io::Error },
    /// The file is there and is not what it claims to be.
    Malformed { path: PathBuf, cause: String },
}

/// So `main` can `?` on a boot-time load. Nothing else in the crate treats
/// these as a `dyn Error`.
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unreadable { cause, .. } => Some(cause),
            Self::Malformed { .. } => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable { path, cause } => {
                write!(f, "cannot read {}: {cause}", path.display())
            }
            Self::Malformed { path, cause } => {
                write!(f, "cannot parse {}: {cause}", path.display())
            }
        }
    }
}

/// The content repo on disk.
///
/// Holds the root and nothing else — no cache, no open handles. Reading is
/// per-call, and whatever caches goes in front of this rather than inside it,
/// so the CLI can expire that cache without this type having an opinion.
#[derive(Clone, Debug)]
pub struct Content {
    root: PathBuf,
}

impl Content {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn books_dir(&self) -> PathBuf {
        self.root.join("books")
    }

    /// Every book slug, in directory order.
    ///
    /// The slug is the directory name — not a field read out of the yaml. Those
    /// two can disagree, and [`Self::book`] is where that is caught; here the
    /// directory is what exists.
    ///
    /// # Errors
    ///
    /// `books/` missing or unreadable.
    pub fn book_slugs(&self) -> Result<Vec<String>, Error> {
        dirs_in(&self.books_dir())
    }

    /// Every lesson folder within a book — `07-borrowing`, not `borrowing`.
    ///
    /// Folder names and not slugs, because the number is half the name and the
    /// caller needs it: it is the lesson's `sort_order`. [`lesson::Folder`]
    /// takes them apart.
    ///
    /// Alphabetical order *is* reading order here, since the numbers are zero
    /// padded — but only up to 99 lessons, and only by accident. Anything that
    /// needs the order should parse it rather than trust this.
    ///
    /// # Errors
    ///
    /// The book's `lessons/` missing or unreadable.
    pub fn lesson_folders(&self, book: &str) -> Result<Vec<String>, Error> {
        dirs_in(&self.book_dir(book).join("lessons"))
    }

    #[must_use]
    pub fn book_dir(&self, book: &str) -> PathBuf {
        self.books_dir().join(book)
    }

    #[must_use]
    pub fn lesson_dir(&self, book: &str, folder: &str) -> PathBuf {
        self.book_dir(book).join("lessons").join(folder)
    }
}

/// Subdirectory names, sorted.
///
/// Sorted because directory order is whatever the filesystem says and differs
/// between machines, which would make a sync run unreproducible.
fn dirs_in(dir: &Path) -> Result<Vec<String>, Error> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|cause| Error::Unreadable {
            path: dir.to_owned(),
            cause,
        })?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    names.sort();

    Ok(names)
}

/// Reads a file, naming it if that fails.
///
/// # Errors
///
/// [`Error::Unreadable`], carrying the path. Which file could not be read is
/// the only useful thing about this failure, and a bare `io::Error` drops it.
pub fn read(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|cause| Error::Unreadable {
        path: path.to_owned(),
        cause,
    })
}

/// Behind a feature rather than `cfg(test)`, which only applies while *this*
/// crate's own tests compile. The api's tests build a router against the fake
/// repo, and a dependency's `cfg(test)` items are invisible to a dependent —
/// so the feature is what carries the fixture across the crate boundary.
///
/// Enabled as a dev-dependency only, so nothing ships in a release binary.
#[cfg(any(test, feature = "testing"))]
pub mod fixture {
    use super::Content;

    /// The thin, fake content repo this crate ships for tests.
    ///
    /// Loads cleanly, end to end. Everything deliberately wrong lives in
    /// [`broken`] instead, because a catalogue walks the whole repo and one
    /// planted mistake would make every test about something else fail.
    #[must_use]
    pub fn content() -> Content {
        Content::at(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixture"))
    }

    /// A repo where things are wrong on purpose, for the tests that check they
    /// are refused.
    ///
    /// Its own root, sitting *beside* `fixture/books/` rather than inside it,
    /// which is what keeps it out of every walk that is not looking for it.
    #[must_use]
    pub fn broken() -> Content {
        Content::at(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixture/mislabelled"
        ))
    }

    /// A repo whose only book is a draft, for the whole-book half of the
    /// policy. Its own root so it cannot pad any other walk.
    #[must_use]
    pub fn drafts() -> Content {
        Content::at(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixture/drafts"
        ))
    }

    /// A repo where two lessons claim the same id.
    ///
    /// Its own root for the same reason [`broken`] has one. Separate from it
    /// because the mistakes are caught in different places — a slug that
    /// disagrees with its folder is refused while parsing one file, and a
    /// reused id is only visible once the whole repo has been walked.
    #[must_use]
    pub fn duplicate_ids() -> Content {
        Content::at(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixture/duplicate-ids"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_lists_its_books_and_lessons() {
        let content = fixture::content();

        assert_eq!(content.book_slugs().unwrap(), ["fixture-book"]);
        assert_eq!(
            content.lesson_folders("fixture-book").unwrap(),
            ["01-free-lesson", "02-split-lesson", "03-draft-lesson"]
        );
    }

    #[test]
    fn a_missing_book_names_the_path_it_looked_in() {
        let error = fixture::content()
            .lesson_folders("no-such-book")
            .unwrap_err();

        assert!(error.to_string().contains("no-such-book"), "{error}");
    }
}
