//! A lesson's yaml, and the markdown beside it.

use serde::Deserialize;
use uuid::Uuid;

use super::{Content, Error, Status, body::Body, read};

/// `books/<book>/lessons/<order>-<slug>/lesson.yaml`.
///
/// Neither `chapter_id` nor `sort_order` is a field here. The chapter comes
/// from which chapter's `lessons` list names this folder, and the order from
/// the folder's own number — so a lesson cannot claim a position that the book
/// disagrees with, because it never states one.
#[derive(Debug, Deserialize)]
pub struct Lesson {
    /// The lesson's permanent identity, and the only thing about it that never
    /// changes.
    ///
    /// `notes.lesson_id` points at this. A note is anchored to a lesson by
    /// character offsets into its body, so a note that follows the wrong id
    /// lands on prose it was never written against — which is worse than a
    /// note that fails to load. That is why this is authored and immutable
    /// rather than derived from the slug or the folder number, both of which
    /// move when a book is reordered or a title is retuned.
    ///
    /// Unique across every book, not just within one: `lessons.id` is a
    /// primary key. A uuid makes that nearly free rather than something to
    /// keep track of by hand, but [`Snapshot::load`] still refuses a repo that
    /// reuses one — the way a duplicate happens here is a folder copied to
    /// start the next lesson, and copying carries the uuid along.
    ///
    /// `None` for a lesson that has not been given one yet, for the same
    /// reason [`super::book::Book::id`] is optional.
    #[serde(default)]
    pub id: Option<Uuid>,
    /// Checked against the folder name with its number stripped, for the reason
    /// `book` gives.
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub seo: LessonSeo,
}

/// A lesson folder name, taken apart: `07-borrowing` is 7 and `borrowing`.
///
/// Both halves are load-bearing, which is why this is parsed rather than read.
/// A folder that is not `<number>-<slug>` has no place in the book's order, and
/// guessing one would put a lesson somewhere nobody chose.
#[derive(Debug, PartialEq, Eq)]
pub struct Folder {
    pub sort_order: i32,
    pub slug: String,
}

impl Folder {
    /// # Errors
    ///
    /// No `-`, or a number that is not one.
    pub fn parse(name: &str) -> Result<Self, String> {
        let (order, slug) = name
            .split_once('-')
            .ok_or_else(|| format!("{name:?} is not <number>-<slug>"))?;

        let sort_order = order
            .parse()
            .map_err(|_| format!("{order:?} in {name:?} is not a number"))?;

        if slug.is_empty() {
            return Err(format!("{name:?} has a number and no slug"));
        }

        Ok(Self {
            sort_order,
            slug: slug.to_owned(),
        })
    }
}

/// A lesson has no `og_*`: a shared link to one shows the book's card, which is
/// the thing worth recognising.
///
/// The shared `meta_` prefix is the column naming, not a stutter.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Deserialize)]
pub struct LessonSeo {
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
}

impl Content {
    /// Reads and parses a lesson's yaml. The prose is [`Content::body`].
    ///
    /// Takes the folder name — `07-borrowing` — because that is what the book's
    /// chapters list and what is on disk.
    ///
    /// # Errors
    ///
    /// As [`Content::book`]: absent, unparseable, or a slug that disagrees with
    /// its folder. Also a folder name that is not `<number>-<slug>`.
    pub fn lesson(&self, book: &str, folder: &str) -> Result<Lesson, Error> {
        let path = self.lesson_dir(book, folder).join("lesson.yaml");
        let named =
            Folder::parse(folder).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause,
            })?;

        let raw = read(&path)?;

        let lesson: Lesson =
            serde_norway::from_str(&raw).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause: cause.to_string(),
            })?;

        if lesson.slug != named.slug {
            return Err(Error::Malformed {
                path,
                cause: format!(
                    "slug is {:?} but the folder is {folder:?}",
                    lesson.slug
                ),
            });
        }

        Ok(lesson)
    }

    /// The prose, split at the paywall.
    ///
    /// Read per call and not cached here — see the note on [`Content`]. The
    /// markdown is *not* rendered: whichever half a reader is entitled to is
    /// rendered when it is served, so the paid half never has to exist as html
    /// in a process that might hand it to the wrong person.
    ///
    /// # Errors
    ///
    /// The markdown being absent or unreadable.
    pub fn body(&self, book: &str, folder: &str) -> Result<Body, Error> {
        let path = self.lesson_dir(book, folder).join("lesson.md");

        Ok(Body::split(&read(&path)?))
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::*;

    #[test]
    fn a_lesson_parses_into_its_columns() {
        let lesson = fixture::content()
            .lesson("fixture-book", "01-free-lesson")
            .unwrap();

        assert_eq!(
            lesson.id,
            "019205c7-4f3a-7c21-9f4e-000000000001".parse().ok()
        );
        assert_eq!(lesson.slug, "free-lesson");
        assert_eq!(lesson.title, "A Free Lesson");
        assert_eq!(lesson.status, Status::Published);
    }

    #[test]
    fn a_lesson_with_no_marker_is_wholly_free() {
        let body = fixture::content()
            .body("fixture-book", "01-free-lesson")
            .unwrap();

        assert!(!body.has_paid_part());
        assert!(body.free.contains("wholly free"));
    }

    #[test]
    fn a_lesson_with_a_marker_splits_and_keeps_the_paid_half_out_of_free() {
        let body = fixture::content()
            .body("fixture-book", "02-split-lesson")
            .unwrap();

        assert!(body.has_paid_part());
        assert!(body.free.contains("above the marker"));
        // The check `docs/rebuild.md` asks the build to make: no paid prose may
        // appear in a free fragment.
        assert!(!body.free.contains("below the marker"));
        assert!(body.paid.as_deref().unwrap().contains("below the marker"));
    }

    #[test]
    fn a_missing_lesson_names_the_file_it_wanted() {
        let error = fixture::content()
            .lesson("fixture-book", "03-no-such-lesson")
            .unwrap_err();

        assert!(error.to_string().contains("no-such-lesson"), "{error}");
    }

    #[test]
    fn a_folder_is_its_order_and_its_slug() {
        let folder = Folder::parse("07-borrowing").unwrap();

        assert_eq!(folder.sort_order, 7);
        assert_eq!(folder.slug, "borrowing");
    }

    #[test]
    fn a_slug_with_its_own_hyphens_keeps_them() {
        let folder = Folder::parse("03-variables-and-mutability").unwrap();

        assert_eq!(folder.sort_order, 3);
        assert_eq!(folder.slug, "variables-and-mutability");
    }

    #[test]
    fn a_folder_with_no_number_is_refused_rather_than_guessed_at() {
        assert!(Folder::parse("borrowing").is_err());
        assert!(Folder::parse("ab-borrowing").is_err());
        assert!(Folder::parse("07-").is_err());
    }

    #[test]
    fn a_lesson_whose_slug_disagrees_with_its_folder_is_refused() {
        let error = fixture::broken()
            .lesson("mislabelled", "01-right-name")
            .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("wrong-name"), "{message}");
        assert!(message.contains("01-right-name"), "{message}");
    }

    #[test]
    fn an_unnumbered_folder_is_refused_before_the_file_is_opened() {
        // The old layout's folder name. It splits on the hyphen like any other,
        // so what refuses it is the number, not the shape.
        let error = fixture::content()
            .lesson("fixture-book", "free-lesson")
            .unwrap_err();

        assert!(error.to_string().contains("is not a number"), "{error}");
    }
}
