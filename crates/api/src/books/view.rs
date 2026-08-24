//! What a reader sees, as json.
//!
//! Separate from `ohara`, which describes what is *on disk*. A yaml field is
//! not automatically a wire field: `status` never ships (nothing unpublished is
//! in the catalogue), and the folder name never ships (a url carries the slug).

use serde::Serialize;

use crate::ohara::{
    catalog::{BookEntry, LessonEntry},
    price::Price,
};

/// A book on a listing page.
///
/// No chapters and no lessons — a listing of twenty books should not carry
/// four hundred lesson titles nobody scrolled to.
#[derive(Debug, Serialize)]
pub(crate) struct BookSummary<'b> {
    slug: &'b str,
    title: &'b str,
    description: Option<&'b str>,
    thumbnail_url: Option<&'b str>,
    price: Price,
    lesson_count: usize,
    /// Where "start reading" goes, or `None` for a book with no published
    /// lessons yet.
    first_lesson: Option<&'b str>,
}

impl<'b> BookSummary<'b> {
    pub(crate) fn of(entry: &'b BookEntry) -> Self {
        Self {
            slug: &entry.book.slug,
            title: &entry.book.title,
            description: entry.book.description.as_deref(),
            thumbnail_url: entry.book.thumbnail_url.as_deref(),
            price: entry.book.price,
            lesson_count: entry.lessons().count(),
            first_lesson: entry
                .first_lesson()
                .map(|lesson| lesson.lesson.slug.as_str()),
        }
    }
}

/// A book's own page: everything above, plus how it is organised.
#[derive(Debug, Serialize)]
pub(crate) struct BookDetail<'b> {
    #[serde(flatten)]
    summary: BookSummary<'b>,
    chapters: Vec<ChapterView<'b>>,
    seo: SeoView<'b>,
}

impl<'b> BookDetail<'b> {
    pub(crate) fn of(entry: &'b BookEntry) -> Self {
        // Built by walking the lessons in reading order and starting a chapter
        // whenever the id changes, rather than by grouping into a map — the
        // reading order is already correct and a map would lose it.
        let mut chapters: Vec<ChapterView<'b>> = Vec::new();

        for lesson in entry.lessons() {
            let title = entry
                .book
                .chapters
                .iter()
                .find(|chapter| chapter.id == lesson.chapter_id)
                .map(|chapter| chapter.title.as_str())
                .unwrap_or_default();

            match chapters.last_mut() {
                Some(open) if open.id == lesson.chapter_id => {
                    open.lessons.push(LessonSummary::of(lesson));
                }
                _ => chapters.push(ChapterView {
                    id: lesson.chapter_id,
                    title,
                    lessons: vec![LessonSummary::of(lesson)],
                }),
            }
        }

        Self {
            summary: BookSummary::of(entry),
            chapters,
            seo: SeoView::of_book(entry),
        }
    }
}

/// A chapter has no page of its own; it only groups lessons on the book's.
#[derive(Debug, Serialize)]
pub(crate) struct ChapterView<'b> {
    id: i32,
    title: &'b str,
    lessons: Vec<LessonSummary<'b>>,
}

/// A lesson in a table of contents.
///
/// No `has_paid_part`: knowing it means reading the markdown, and a book page
/// would then open every lesson file to render one list.
#[derive(Debug, Serialize)]
pub(crate) struct LessonSummary<'b> {
    slug: &'b str,
    title: &'b str,
    description: Option<&'b str>,
    sort_order: i32,
}

impl<'b> LessonSummary<'b> {
    fn of(entry: &'b LessonEntry) -> Self {
        Self {
            slug: &entry.lesson.slug,
            title: &entry.lesson.title,
            description: entry.lesson.description.as_deref(),
            sort_order: entry.sort_order,
        }
    }
}

/// A lesson as it is read: the free half, rendered, and how to get around.
///
/// The paid half is *not* here and cannot be — this response is
/// `CachePolicy::Shared`, identical for everyone, and held at the edge.
#[derive(Debug, Serialize)]
pub(crate) struct LessonView<'b> {
    slug: &'b str,
    title: &'b str,
    description: Option<&'b str>,
    chapter_id: i32,
    sort_order: i32,
    /// The free half, as html.
    html: String,
    /// Whether anything is being withheld. What the "read the rest" call to
    /// action keys off, and false for a lesson with no paywall at all.
    has_paid_part: bool,
    book: BookRef<'b>,
    previous: Option<&'b str>,
    next: Option<&'b str>,
    seo: SeoView<'b>,
}

impl<'b> LessonView<'b> {
    pub(crate) fn of(
        book: &'b BookEntry,
        entry: &'b LessonEntry,
        html: String,
        has_paid_part: bool,
    ) -> Self {
        let (previous, next) = book.neighbours(&entry.lesson.slug);

        Self {
            slug: &entry.lesson.slug,
            title: &entry.lesson.title,
            description: entry.lesson.description.as_deref(),
            chapter_id: entry.chapter_id,
            sort_order: entry.sort_order,
            html,
            has_paid_part,
            book: BookRef {
                slug: &book.book.slug,
                title: &book.book.title,
            },
            previous,
            next,
            seo: SeoView::of_lesson(entry),
        }
    }
}

/// Enough of the book to render a breadcrumb without a second request.
#[derive(Debug, Serialize)]
pub(crate) struct BookRef<'b> {
    slug: &'b str,
    title: &'b str,
}

/// The paid half, and nothing else.
///
/// Its own tiny shape rather than a second copy of [`LessonView`]: the reader
/// already has everything else, and repeating it here would be a second place
/// for the title to be wrong.
#[derive(Debug, Serialize)]
pub(crate) struct PaidView {
    pub(crate) html: String,
}

/// What a crawler reads.
///
/// Falls back to the title and description a reader sees, so a lesson with no
/// `meta_title` still gets a useful one rather than the site's name repeated.
#[derive(Debug, Serialize)]
pub(crate) struct SeoView<'b> {
    meta_title: &'b str,
    meta_description: Option<&'b str>,
    meta_keywords: Option<&'b str>,
    og_title: Option<&'b str>,
    og_description: Option<&'b str>,
    og_image: Option<&'b str>,
}

impl<'b> SeoView<'b> {
    fn of_book(entry: &'b BookEntry) -> Self {
        let seo = &entry.book.seo;

        Self {
            meta_title: seo.meta_title.as_deref().unwrap_or(&entry.book.title),
            meta_description: seo
                .meta_description
                .as_deref()
                .or(entry.book.description.as_deref()),
            meta_keywords: seo.meta_keywords.as_deref(),
            og_title: seo.og_title.as_deref(),
            og_description: seo.og_description.as_deref(),
            og_image: seo
                .og_image
                .as_deref()
                .or(entry.book.thumbnail_url.as_deref()),
        }
    }

    /// A lesson has no `og_*` of its own — a shared link to one shows the
    /// book's card, which is the thing worth recognising.
    fn of_lesson(entry: &'b LessonEntry) -> Self {
        let seo = &entry.lesson.seo;

        Self {
            meta_title: seo
                .meta_title
                .as_deref()
                .unwrap_or(&entry.lesson.title),
            meta_description: seo
                .meta_description
                .as_deref()
                .or(entry.lesson.description.as_deref()),
            meta_keywords: seo.meta_keywords.as_deref(),
            og_title: None,
            og_description: None,
            og_image: None,
        }
    }
}
