//! A book's yaml.

use std::collections::HashSet;

use serde::Deserialize;

use super::{Content, Error, Status, Tier, lesson::Folder, price::Price, read};

/// `books/<slug>/book.yaml`.
///
/// The fields are the columns, near enough: everything here lands in `books` or
/// `book_translations`. What the old course yaml carried and this does not —
/// `is_published`, `required_tier`, `lab_slug`, `details_component`,
/// `visibility_level` — went with the migrations that dropped those columns.
#[derive(Debug, Deserialize)]
pub(crate) struct Book {
    /// Checked against the directory name. They can disagree, and a book whose
    /// yaml claims a different slug would sync into the wrong row.
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) status: Status,
    #[serde(default)]
    pub(crate) tier: Tier,
    pub(crate) thumbnail_url: Option<String>,
    /// Absent is free.
    #[serde(default)]
    pub(crate) price: Price,
    #[serde(default)]
    pub(crate) chapters: Vec<Chapter>,
    #[serde(default)]
    pub(crate) seo: Seo,
}

impl Book {
    /// Every lesson folder in the book, in reading order.
    ///
    /// Reading order is chapter order then position within the chapter, both of
    /// which are array position. A lesson on disk that no chapter lists is not
    /// here, and is not published — see [`Content::book`].
    pub(crate) fn lesson_folders(&self) -> impl Iterator<Item = &str> {
        self.chapters
            .iter()
            .flat_map(|chapter| chapter.lessons.iter().map(String::as_str))
    }

    /// Where "start reading" goes.
    pub(crate) fn first_lesson_folder(&self) -> Option<&str> {
        self.lesson_folders().next()
    }
}

/// A chapter has no table of its own — `lessons.chapter_id` is a bare integer.
/// The titles live in `book_translations.metadata`, which is the json column
/// that exists for exactly this kind of thing.
///
/// No `sort_order`: chapters are ordered by their position in the file, which
/// is the order they are read in anyway.
#[derive(Debug, Deserialize)]
pub(crate) struct Chapter {
    /// Stable across reordering, because `lessons.chapter_id` in the database
    /// points at it and moving a chapter must not repoint every lesson.
    pub(crate) id: i32,
    pub(crate) title: String,
    /// Lesson folder names — `07-borrowing`, the thing on disk.
    #[serde(default)]
    pub(crate) lessons: Vec<String>,
}

/// Everything a crawler reads, and nothing a reader does.
///
/// Separate from the fields above because it is optional in a way they are not:
/// a book without a `meta_title` falls back to its title, and a book without a
/// title is a bug.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Seo {
    pub(crate) meta_title: Option<String>,
    pub(crate) meta_description: Option<String>,
    pub(crate) meta_keywords: Option<String>,
    pub(crate) og_title: Option<String>,
    pub(crate) og_description: Option<String>,
    pub(crate) og_image: Option<String>,
}

impl Book {
    /// Rejects a book that would sync into the wrong shape.
    fn validate(&self) -> Result<(), String> {
        let mut chapter_ids = HashSet::new();
        let mut folders = HashSet::new();
        let mut previous = 0;

        for chapter in &self.chapters {
            // Two chapters sharing an id merge in the database, because
            // `lessons.chapter_id` is all that distinguishes them.
            if !chapter_ids.insert(chapter.id) {
                return Err(format!("two chapters share id {}", chapter.id));
            }

            for folder in &chapter.lessons {
                if !folders.insert(folder.as_str()) {
                    return Err(format!("{folder:?} is listed twice"));
                }

                let order = Folder::parse(folder)?.sort_order;

                // The number and the list both claim to be the order. Letting
                // them drift means the book reads in one order and every
                // filesystem listing shows another.
                if order <= previous {
                    return Err(format!(
                        "{folder:?} is listed after {previous}, \
                         so the numbers and the chapters disagree"
                    ));
                }

                previous = order;
            }
        }

        Ok(())
    }
}

impl Content {
    /// Reads and parses `books/<slug>/book.yaml`.
    ///
    /// A lesson folder on disk that no chapter lists is *not* an error here —
    /// it is an unpublished draft, and the sync binary is what reports them.
    ///
    /// # Errors
    ///
    /// The file being absent or unparseable, or its `slug` disagreeing with the
    /// directory it sits in — which would otherwise sync a book into the wrong
    /// row, or create a second one nobody meant. Also chapters that share an
    /// id, a lesson listed twice, or lesson numbers that disagree with the
    /// order the chapters put them in.
    pub(crate) fn book(&self, slug: &str) -> Result<Book, Error> {
        let path = self.book_dir(slug).join("book.yaml");
        let raw = read(&path)?;

        let book: Book =
            serde_norway::from_str(&raw).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause: cause.to_string(),
            })?;

        if book.slug != slug {
            return Err(Error::Malformed {
                path,
                cause: format!(
                    "slug is {:?} but the directory is {slug:?}",
                    book.slug
                ),
            });
        }

        book.validate()
            .map_err(|cause| Error::Malformed { path, cause })?;

        Ok(book)
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::*;

    fn parsed(yaml: &str) -> Book {
        serde_norway::from_str(yaml).unwrap()
    }

    #[test]
    fn a_book_parses_into_its_columns() {
        let book = fixture::content().book("fixture-book").unwrap();

        assert_eq!(book.slug, "fixture-book");
        assert_eq!(book.title, "A Fixture Book");
        assert_eq!(book.status, Status::Published);
        assert_eq!(book.tier, Tier::Foundation);
        assert_eq!(book.price.amount, 2900);
        assert_eq!(book.chapters.len(), 1);
        assert_eq!(book.seo.meta_title.as_deref(), Some("A Fixture Book"));
    }

    #[test]
    fn a_slug_that_disagrees_with_its_directory_is_refused() {
        let error = fixture::broken().book("mislabelled").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("mislabelled"), "{message}");
    }

    #[test]
    fn a_book_with_no_price_block_is_free() {
        let book = parsed("slug: b\ntitle: B\n");

        assert!(book.price.is_free());
    }

    #[test]
    fn lessons_read_in_chapter_order_then_position() {
        let book = parsed(
            "slug: b\ntitle: B\nchapters:\n\
             - id: 1\n  title: One\n  lessons: [\"01-a\", \"02-b\"]\n\
             - id: 2\n  title: Two\n  lessons: [\"03-c\"]\n",
        );

        assert_eq!(
            book.lesson_folders().collect::<Vec<_>>(),
            ["01-a", "02-b", "03-c"]
        );
        assert_eq!(book.first_lesson_folder(), Some("01-a"));
    }

    #[test]
    fn a_book_with_no_chapters_has_nowhere_to_start() {
        assert_eq!(parsed("slug: b\ntitle: B\n").first_lesson_folder(), None);
    }

    #[test]
    fn two_chapters_sharing_an_id_are_refused() {
        let book = parsed(
            "slug: b\ntitle: B\nchapters:\n\
             - id: 1\n  title: One\n  lessons: [\"01-a\"]\n\
             - id: 1\n  title: Two\n  lessons: [\"02-b\"]\n",
        );

        let cause = book.validate().unwrap_err();

        assert!(cause.contains("share id 1"), "{cause}");
    }

    #[test]
    fn a_lesson_listed_in_two_chapters_is_refused() {
        let book = parsed(
            "slug: b\ntitle: B\nchapters:\n\
             - id: 1\n  title: One\n  lessons: [\"01-a\"]\n\
             - id: 2\n  title: Two\n  lessons: [\"01-a\"]\n",
        );

        assert!(book.validate().unwrap_err().contains("listed twice"));
    }

    #[test]
    fn numbers_that_disagree_with_the_chapter_order_are_refused() {
        // Renaming a folder without moving it in the file — the book would read
        // in one order and every filesystem listing would show another.
        let book = parsed(
            "slug: b\ntitle: B\nchapters:\n\
             - id: 1\n  title: One\n  lessons: [\"02-b\", \"01-a\"]\n",
        );

        let cause = book.validate().unwrap_err();

        assert!(cause.contains("disagree"), "{cause}");
    }

    #[test]
    fn a_lesson_folder_with_no_number_is_refused() {
        let book = parsed(
            "slug: b\ntitle: B\nchapters:\n\
             - id: 1\n  title: One\n  lessons: [\"borrowing\"]\n",
        );

        assert!(book.validate().unwrap_err().contains("<number>-<slug>"));
    }
}
