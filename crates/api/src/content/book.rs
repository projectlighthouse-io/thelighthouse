//! A book's yaml.

use serde::Deserialize;

use super::{Content, Error, Status, Tier, read};

/// `books/<slug>/<slug>.yaml`.
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
    /// Minor units, as `books.price` stores it. Absent is free.
    #[serde(default)]
    pub(crate) price: i32,
    /// Where "start reading" goes. Absent means the lowest `sort_order`.
    pub(crate) first_lesson_slug: Option<String>,
    #[serde(default)]
    pub(crate) chapters: Vec<Chapter>,
    #[serde(default)]
    pub(crate) seo: Seo,
}

/// A chapter has no table of its own — `lessons.chapter_id` is a bare integer.
/// The titles live in `book_translations.metadata`, which is the json column
/// that exists for exactly this kind of thing.
#[derive(Debug, Deserialize)]
pub(crate) struct Chapter {
    pub(crate) id: i32,
    pub(crate) title: String,
    pub(crate) sort_order: i32,
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

impl Content {
    /// Reads and parses `books/<slug>/<slug>.yaml`.
    ///
    /// # Errors
    ///
    /// The file being absent or unparseable, or its `slug` disagreeing with the
    /// directory it sits in — which would otherwise sync a book into the wrong
    /// row, or create a second one nobody meant.
    pub(crate) fn book(&self, slug: &str) -> Result<Book, Error> {
        let path = self.book_dir(slug).join(format!("{slug}.yaml"));
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

        Ok(book)
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::*;

    #[test]
    fn a_book_parses_into_its_columns() {
        let book = fixture::content().book("fixture-book").unwrap();

        assert_eq!(book.slug, "fixture-book");
        assert_eq!(book.title, "A Fixture Book");
        assert_eq!(book.status, Status::Published);
        assert_eq!(book.tier, Tier::Foundation);
        assert_eq!(book.chapters.len(), 1);
        assert_eq!(book.seo.meta_title.as_deref(), Some("A Fixture Book"));
    }

    #[test]
    fn a_slug_that_disagrees_with_its_directory_is_refused() {
        let error = fixture::content().book("mislabelled").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("mislabelled"), "{message}");
    }
}
