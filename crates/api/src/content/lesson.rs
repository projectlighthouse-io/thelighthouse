//! A lesson's yaml, and the markdown beside it.

use serde::Deserialize;

use super::{Content, Error, Status, body::Body, read};

/// `books/<book>/lessons/<slug>/<slug>.yaml`.
#[derive(Debug, Deserialize)]
pub(crate) struct Lesson {
    /// Checked against the directory name, for the reason `book` gives.
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    /// Matches a `Chapter::id` in the book's yaml. A bare integer in the
    /// database too — `lessons.chapter_id` has no foreign key.
    pub(crate) chapter_id: i32,
    pub(crate) sort_order: i32,
    #[serde(default)]
    pub(crate) status: Status,
    #[serde(default)]
    pub(crate) seo: LessonSeo,
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
    /// # Errors
    ///
    /// As [`Content::book`]: absent, unparseable, or a slug that disagrees with
    /// its directory.
    pub(crate) fn lesson(
        &self,
        book: &str,
        slug: &str,
    ) -> Result<Lesson, Error> {
        let path = self.lesson_dir(book, slug).join(format!("{slug}.yaml"));
        let raw = read(&path)?;

        let lesson: Lesson =
            serde_norway::from_str(&raw).map_err(|cause| Error::Malformed {
                path: path.clone(),
                cause: cause.to_string(),
            })?;

        if lesson.slug != slug {
            return Err(Error::Malformed {
                path,
                cause: format!(
                    "slug is {:?} but the directory is {slug:?}",
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
    pub(crate) fn body(&self, book: &str, slug: &str) -> Result<Body, Error> {
        let path = self.lesson_dir(book, slug).join(format!("{slug}.md"));

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
            .lesson("fixture-book", "free-lesson")
            .unwrap();

        assert_eq!(lesson.slug, "free-lesson");
        assert_eq!(lesson.title, "A Free Lesson");
        assert_eq!(lesson.chapter_id, 1);
        assert_eq!(lesson.sort_order, 1);
        assert_eq!(lesson.status, Status::Published);
    }

    #[test]
    fn a_lesson_with_no_marker_is_wholly_free() {
        let body = fixture::content()
            .body("fixture-book", "free-lesson")
            .unwrap();

        assert!(!body.has_paid_part());
        assert!(body.free.contains("wholly free"));
    }

    #[test]
    fn a_lesson_with_a_marker_splits_and_keeps_the_paid_half_out_of_free() {
        let body = fixture::content()
            .body("fixture-book", "split-lesson")
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
            .lesson("fixture-book", "no-such-lesson")
            .unwrap_err();

        assert!(error.to_string().contains("no-such-lesson"), "{error}");
    }
}
