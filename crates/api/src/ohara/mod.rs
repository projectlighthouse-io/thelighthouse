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
//!     pricing/
//!       ppp.yaml
//! ```
//!
//! **The directory names the instance, the file names the type.** Every lesson
//! is `lesson.yaml` and `lesson.md`; which lesson it is comes from the folder.
//! Renumbering a book is then one rename per lesson rather than three, and a
//! folder whose name drifts from its files is not expressible.
//!
//! **A lesson folder is `<sort_order>-<slug>`.** Both halves are load-bearing:
//! the number orders the book, the rest is checked against the yaml's `slug`.
//! Neither is repeated in a yaml field, so neither can disagree with itself.
//!
//! **This repo knows the structure. It never contains the content.** The prose
//! is the product, it lives in a private repo, and nothing here should tempt
//! anyone to check a copy in. Tests run against `content-fixture/`, which is
//! deliberately thin and deliberately fake.
//!
//! ```text
//!   mod.rs     the layout above, and walking it
//!   status.rs  the smallints `status` and `tier` store
//!   book.rs    a book's yaml
//!   lesson.rs  a lesson's yaml, and its folder name
//!   body.rs    markdown: the paywall split, and rendering both halves
//!   price.rs   what a book costs
//!   ppp.rs     what it costs somewhere poorer
//! ```

// Nothing reads the content repo yet — the sync binary and the lesson endpoint
// are the next two commits. Holding the layout in someone's head until then is
// how a directory convention ends up implemented twice, differently. This allow
// goes with the first reader.
#![allow(dead_code)]

pub(crate) mod body;
pub(crate) mod book;
pub(crate) mod lesson;
pub(crate) mod ppp;
pub(crate) mod price;
pub(crate) mod status;

use std::{
    io,
    path::{Path, PathBuf},
};

pub(crate) use status::{Status, Tier};

/// What went wrong reading the content repo.
///
/// Every variant names the path, because the first question about any of these
/// is "which file" and a bare `io::Error` does not say.
#[derive(Debug)]
pub(crate) enum Error {
    /// The path does not exist, or cannot be read.
    Unreadable { path: PathBuf, cause: io::Error },
    /// The file is there and is not what it claims to be.
    Malformed { path: PathBuf, cause: String },
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
pub(crate) struct Content {
    root: PathBuf,
}

impl Content {
    pub(crate) fn at(root: impl Into<PathBuf>) -> Self {
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
    pub(crate) fn book_slugs(&self) -> Result<Vec<String>, Error> {
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
    pub(crate) fn lesson_folders(
        &self,
        book: &str,
    ) -> Result<Vec<String>, Error> {
        dirs_in(&self.book_dir(book).join("lessons"))
    }

    pub(crate) fn book_dir(&self, book: &str) -> PathBuf {
        self.books_dir().join(book)
    }

    pub(crate) fn lesson_dir(&self, book: &str, folder: &str) -> PathBuf {
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
pub(crate) fn read(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|cause| Error::Unreadable {
        path: path.to_owned(),
        cause,
    })
}

#[cfg(test)]
pub(crate) mod fixture {
    use super::Content;

    /// The thin, fake content repo this crate ships for tests.
    pub(crate) fn content() -> Content {
        Content::at(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../content-fixture"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_lists_its_books_and_lessons() {
        let content = fixture::content();

        assert_eq!(
            content.book_slugs().unwrap(),
            ["fixture-book", "mislabelled"]
        );
        assert_eq!(
            content.lesson_folders("fixture-book").unwrap(),
            ["01-free-lesson", "02-split-lesson"]
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
