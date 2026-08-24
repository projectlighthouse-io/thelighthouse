//! A lesson's yaml, and the markdown beside it.

use serde::Deserialize;

use super::{Content, Error, Status, body::Body, read};

/// `books/<book>/lessons/<order>-<slug>/lesson.yaml`.
///
/// Neither `chapter_id` nor `sort_order` is a field here. The chapter comes
/// from which chapter's `lessons` list names this folder, and the order from
/// the folder's own number — so a lesson cannot claim a position that the book
/// disagrees with, because it never states one.
#[derive(Debug, Deserialize)]
pub(crate) struct Lesson {
    /// Checked against the folder name with its number stripped, for the reason
    /// `book` gives.
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) status: Status,
    #[serde(default)]
    pub(crate) seo: LessonSeo,
}

/// A lesson folder name, taken apart: `07-borrowing` is 7 and `borrowing`.
///
/// Both halves are load-bearing, which is why this is parsed rather than read.
/// A folder that is not `<number>-<slug>` has no place in the book's order, and
/// guessing one would put a lesson somewhere nobody chose.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Folder {
    pub(crate) sort_order: i32,
    pub(crate) slug: String,
}

impl Folder {
    /// # Errors
    ///
    /// No `-`, or a number that is not one.
    pub(crate) fn parse(name: &str) -> Result<Self, String> {
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
pub(crate) struct LessonSeo {
    pub(crate) meta_title: Option<String>,
    pub(crate) meta_description: Option<String>,
    pub(crate) meta_keywords: Option<String>,
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
    pub(crate) fn lesson(
        &self,
        book: &str,
        folder: &str,
    ) -> Result<Lesson, Error> {
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
    pub(crate) fn body(&self, book: &str, folder: &str) -> Result<Body, Error> {
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
        let error = fixture::content()
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
