//! A lesson's yaml, and the markdown beside it.

use std::collections::HashMap;

use serde::Deserialize;
use uuid::Uuid;

use super::{
    Content, Error, Locale, Status,
    body::{Access, Body},
    read, within_the_folder,
};

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
    /// How much of this lesson is free.
    ///
    /// Absent is [`Access::Free`] — the marker decides — so every lesson
    /// written before this field existed keeps the behaviour it had.
    ///
    /// This is about *this lesson*. Whether a reader clears it at all is the
    /// book's price and their entitlement, and a lesson marked `paid` inside a
    /// free book is still served in full, because a book given away has
    /// nothing to buy.
    #[serde(default)]
    pub access: Access,
    /// Which file holds the prose, per language, relative to this lesson's own
    /// folder.
    ///
    /// **Required, and required to carry `en`.** A lesson with no body is not a
    /// lesson, and English is the fallback every other language resolves
    /// through — see [`Self::body_file`]. Both are checked in
    /// [`Self::validate`], so neither can be discovered at read time.
    ///
    /// A map rather than a list of pairs: two entries for one language are then
    /// impossible to write down, instead of being a duplicate that some
    /// validation has to go looking for.
    ///
    /// Stated per lesson rather than derived from a naming convention, because
    /// a convention is a rule the files can break silently. `lesson.bn.md`
    /// sitting beside a lesson that never names it is a translation nobody is
    /// served; here it is either listed or it does not exist.
    pub content_path: HashMap<Locale, String>,
    #[serde(default)]
    pub seo: LessonSeo,
}

impl Lesson {
    /// The file to read for `locale`, falling back to English.
    ///
    /// Infallible because [`Self::validate`] has already refused a lesson with
    /// no `en` entry. A reader whose language is not translated yet gets the
    /// English prose, which is what `docs/rebuild.md` specifies — never a 404.
    #[must_use]
    pub fn body_file(&self, locale: Locale) -> &str {
        self.content_path
            .get(&locale)
            .or_else(|| self.content_path.get(&Locale::En))
            .map_or("lesson.md", String::as_str)
    }

    /// Every language this lesson is actually written in.
    #[must_use]
    pub fn locales(&self) -> Vec<Locale> {
        let mut locales: Vec<Locale> =
            self.content_path.keys().copied().collect();
        locales.sort_unstable();
        locales
    }

    /// Rejects a lesson whose body could not be read, or could be read from
    /// somewhere it has no business reaching.
    fn validate(&self) -> Result<(), String> {
        if !self.content_path.contains_key(&Locale::En) {
            return Err(
                "content_path has no `en`, which is the fallback every \
                 other language resolves through"
                    .to_owned(),
            );
        }

        for (locale, path) in &self.content_path {
            within_the_folder(path, "lesson")
                .map_err(|cause| format!("content_path.{locale}: {cause}"))?;
        }

        Ok(())
    }
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

        lesson
            .validate()
            .map_err(|cause| Error::Malformed { path, cause })?;

        Ok(lesson)
    }

    /// The prose, split at the paywall.
    ///
    /// Read per call and not cached here — see the note on [`Content`]. The
    /// markdown is *not* rendered: whichever half a reader is entitled to is
    /// rendered when it is served, so the paid half never has to exist as html
    /// in a process that might hand it to the wrong person.
    ///
    /// Takes the file rather than a locale, because resolving a language to a
    /// file needs the lesson's yaml and this type does not hold one.
    /// [`super::catalog::Catalog::body`] is where the two meet.
    ///
    /// # Errors
    ///
    /// The markdown being absent or unreadable.
    pub fn body(
        &self,
        book: &str,
        folder: &str,
        file: &str,
        access: Access,
    ) -> Result<Body, Error> {
        let path = self.lesson_dir(book, folder).join(file);

        Ok(Body::under(&read(&path)?, access))
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture;
    use super::*;

    fn parsed(yaml: &str) -> Lesson {
        serde_norway::from_str(yaml).unwrap()
    }

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
            .body("fixture-book", "01-free-lesson", "lesson.md", Access::Free)
            .unwrap();

        assert!(!body.has_paid_part());
        assert!(body.free.contains("wholly free"));
    }

    #[test]
    fn a_lesson_with_a_marker_splits_and_keeps_the_paid_half_out_of_free() {
        let body = fixture::content()
            .body("fixture-book", "02-split-lesson", "lesson.md", Access::Free)
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
    fn a_lesson_names_its_body_per_language() {
        let lesson = fixture::content()
            .lesson("fixture-book", "01-free-lesson")
            .unwrap();

        assert_eq!(lesson.body_file(Locale::En), "lesson.md");
        assert_eq!(lesson.body_file(Locale::Bn), "lesson.bn.md");
        assert_eq!(lesson.locales(), [Locale::En, Locale::Bn]);
    }

    #[test]
    fn an_untranslated_language_falls_back_to_english() {
        // Not a 404. A reader whose language is half translated should read
        // the book, not hit holes in it.
        let lesson = fixture::content()
            .lesson("fixture-book", "02-split-lesson")
            .unwrap();

        assert_eq!(lesson.locales(), [Locale::En]);
        assert_eq!(lesson.body_file(Locale::Bn), "lesson.md");
    }

    #[test]
    fn the_translated_body_is_the_one_that_gets_read() {
        // The whole point of the field: a different file, not a different
        // rendering of the same one.
        let english = fixture::content()
            .body("fixture-book", "01-free-lesson", "lesson.md", Access::Free)
            .unwrap();
        let bengali = fixture::content()
            .body(
                "fixture-book",
                "01-free-lesson",
                "lesson.bn.md",
                Access::Free,
            )
            .unwrap();

        assert!(english.free.contains("wholly free"));
        assert!(bengali.free.contains("বিনামূল্যে"));
        assert_ne!(english.free, bengali.free);
    }

    #[test]
    fn a_lesson_with_no_english_is_refused() {
        // English is the fallback every other language resolves through, so a
        // lesson without it has languages that resolve to nothing.
        let lesson =
            parsed("slug: a\ntitle: A\ncontent_path:\n  bn: lesson.bn.md\n");

        assert!(lesson.validate().unwrap_err().contains("no `en`"));
    }

    #[test]
    fn a_body_path_cannot_climb_out_of_its_lesson() {
        // `join` on a relative path that climbs is a real read of a real file
        // somewhere else on the box, and nothing downstream would notice.
        let lesson = parsed(
            "slug: a\ntitle: A\ncontent_path:\n               en: ../../../../etc/passwd\n",
        );

        let cause = lesson.validate().unwrap_err();

        assert!(cause.contains("climbs out"), "{cause}");
        assert!(cause.contains("content_path.en"), "{cause}");
    }

    #[test]
    fn an_absolute_body_path_is_refused() {
        // `PathBuf::join` *replaces* the base when the argument is absolute,
        // so this would silently resolve to /etc/passwd rather than error.
        let lesson =
            parsed("slug: a\ntitle: A\ncontent_path:\n  en: /etc/passwd\n");

        assert!(lesson.validate().unwrap_err().contains("absolute"));
    }

    #[test]
    fn an_empty_body_path_is_refused() {
        let lesson = parsed("slug: a\ntitle: A\ncontent_path:\n  en: ''\n");

        assert!(lesson.validate().unwrap_err().contains("is empty"));
    }

    #[test]
    fn a_subfolder_is_allowed_because_it_stays_inside() {
        let lesson =
            parsed("slug: a\ntitle: A\ncontent_path:\n  en: parts/lesson.md\n");

        assert!(lesson.validate().is_ok());
    }

    #[test]
    fn a_language_the_site_does_not_publish_names_the_file() {
        // The reason this is an enum: as a string key it would have parsed,
        // and `end` would be a language nothing ever asks for.
        let error = serde_norway::from_str::<Lesson>(
            "slug: a\ntitle: A\ncontent_path:\n  end: lesson.md\n",
        );

        assert!(error.is_err());
    }

    #[test]
    fn a_lesson_with_no_content_path_is_refused_rather_than_assumed() {
        // A convention — "it is always lesson.md" — is a rule the files can
        // break silently. Stating it is what makes a missing body loud.
        assert!(
            serde_norway::from_str::<Lesson>("slug: a\ntitle: A\n").is_err()
        );
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
