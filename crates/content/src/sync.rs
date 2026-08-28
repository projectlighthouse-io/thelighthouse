//! Ohara into postgres: the rows that let other tables point at content.
//!
//! **The database is not where content is read from.** Bodies stay on disk and
//! the catalogue stays in memory — `docs/rebuild.md` is explicit that lesson
//! prose does not live here. What these rows exist for is everything that has
//! to *reference* a lesson or a book: `notes.lesson_id`,
//! `lesson_bookmarks.lesson_id`, `lesson_completions.lesson_id`,
//! `entitlements.book_id`. A foreign key needs a row to point at, and that row
//! is the whole job.
//!
//! Which is why there is no lesson translation written here at all, and why
//! the table is gone: a lesson's title, description and seo are served from the
//! catalogue, and its prose is read off disk per request so that entitlement
//! decides which half a reader gets. A `content` column would have put the paid
//! half of every lesson in a table no entitlement check guards.
//!
//! Translation lives in the content repo instead. Each `lesson.yaml` maps a
//! locale to a markdown file beside it, so a translation ships with the prose
//! it translates rather than in a second row here.
//!
//! **Read-only against the content repo.** Nothing here opens a file for
//! writing. A book or lesson with no `id` is refused by name rather than having
//! one minted for it — minting would mean either writing back into the yaml, or
//! a row whose id the repo does not know, which the next run would insert a
//! second copy of.
//!
//! **Upserts, and nothing else.** No statement deletes and none flips a row
//! back to draft. A lesson that leaves ohara keeps its row, which is what keeps
//! the notes written against it: `notes_lesson_id_foreign` carries no
//! `ON DELETE`, deliberately. The cost is a stale row after an unpublish —
//! accepted, because the alternative is a half-cloned content repo quietly
//! unpublishing the site.

use ohara::{
    book::Book,
    catalog::{BookEntry, LessonEntry, Snapshot},
};
use serde::Serialize;
use sqlx::{PgConnection, postgres::PgPool};
use uuid::Uuid;

/// The locale the one remaining translation row is written under.
///
/// Only `book_translations` has one now — a lesson's prose and metadata both
/// come from the content repo, so `lesson_translations` was dropped rather
/// than filled. Scoping the statement to `en` means a `bn` book row somebody
/// wrote by hand is never touched by a sync.
const LOCALE: &str = "en";

/// The columns are `timestamp without time zone` and laravel wrote UTC into
/// them. `now()` alone would be cast using the server's `TimeZone` setting,
/// which is whatever the connection happens to have.
const STAMP: &str = "(now() AT TIME ZONE 'utc')";

#[derive(Debug)]
pub(crate) enum Error {
    /// Content that cannot be synced as it stands, one line per file. Every
    /// offender is listed rather than the first: fixing them one run at a time
    /// is the slow way to find out there were six.
    Incomplete(Vec<String>),
    /// Names what was being written, because "duplicate key" without a slug
    /// beside it is a grep through the whole repo.
    Database { writing: String, cause: sqlx::Error },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Incomplete(_) => None,
            Self::Database { cause, .. } => Some(cause),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete(offenders) => {
                write!(f, "the content repo is not ready to sync:")?;
                for offender in offenders {
                    write!(f, "\n  {offender}")?;
                }
                Ok(())
            }
            Self::Database { writing, cause } => {
                write!(f, "failed to write {writing}: {cause}")
            }
        }
    }
}

/// What a run wrote, for the caller to print.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Synced {
    pub(crate) books: usize,
    pub(crate) lessons: usize,
}

/// Chapter titles, which have no table of their own.
///
/// `lessons.chapter_id` is a bare integer and the titles are prose, so they
/// belong with the other prose — `book_translations.metadata`, per the note on
/// [`ohara::book::Chapter`]. Serialised whole and written whole, so a chapter
/// removed from the yaml leaves the column rather than lingering in it.
#[derive(Debug, Serialize)]
struct Metadata<'a> {
    chapters: Vec<ChapterEntry<'a>>,
}

#[derive(Debug, Serialize)]
struct ChapterEntry<'a> {
    id: i32,
    title: &'a str,
}

/// Writes every published book and lesson in the snapshot.
///
/// One transaction. A sync that committed the books and then failed on a lesson
/// would leave a book whose reader hits a 404 on the first thing they click,
/// and re-running would not tell you that had happened.
///
/// Only what the snapshot holds, which is only what is published — see
/// `catalog`. Drafts have no readers, so they have nothing to reference them,
/// so they need no row.
///
/// # Errors
///
/// [`Error::Incomplete`] when any book or lesson has no `id`, checked before
/// anything is written. Otherwise whatever postgres said, with the book or
/// lesson it was writing named.
pub(crate) async fn run(
    pool: &PgPool,
    snapshot: &Snapshot,
) -> Result<Synced, Error> {
    identified(snapshot)?;

    let mut tx = pool.begin().await.map_err(|cause| Error::Database {
        writing: "the content sync".to_owned(),
        cause,
    })?;

    let mut synced = Synced::default();

    for entry in snapshot.books() {
        // Skipped rather than refused, and unreachable: `identified` has
        // already turned a missing id into an error naming the file. The
        // `Option` is the file format's, not this loop's.
        let Some(book_id) = entry.book.id else {
            continue;
        };

        book(&mut tx, entry, book_id).await?;
        synced.books += 1;

        for held in entry.lessons() {
            let Some(lesson_id) = held.lesson.id else {
                continue;
            };

            lesson(&mut tx, held, lesson_id, book_id, &entry.book.slug).await?;
            synced.lessons += 1;
        }
    }

    tx.commit().await.map_err(|cause| Error::Database {
        writing: "the content sync".to_owned(),
        cause,
    })?;

    Ok(synced)
}

/// One book, and its `en` translation.
///
/// Two statements rather than one: the columns a reader sees are prose and live
/// in `book_translations`, which is what makes a Bengali edition a second row
/// instead of a second table.
async fn book(
    tx: &mut PgConnection,
    entry: &BookEntry,
    id: Uuid,
) -> Result<(), Error> {
    let book = &entry.book;
    let writing = || format!("book {:?}", book.slug);

    sqlx::query(&format!(
        "INSERT INTO books
             (id, slug, thumbnail_url, price, first_lesson_slug,
              status, tier, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, {STAMP}, {STAMP})
         ON CONFLICT (id) DO UPDATE SET
             slug              = EXCLUDED.slug,
             thumbnail_url     = EXCLUDED.thumbnail_url,
             price             = EXCLUDED.price,
             first_lesson_slug = EXCLUDED.first_lesson_slug,
             status            = EXCLUDED.status,
             tier              = EXCLUDED.tier,
             updated_at        = EXCLUDED.updated_at"
    ))
    .bind(id)
    .bind(&book.slug)
    .bind(book.thumbnail_url.as_deref())
    // Minor units, and zero is free rather than null. Null would be a third
    // state beside "free" and "priced" that nothing here means.
    .bind(book.price.amount)
    .bind(entry.first_lesson().map(|first| first.lesson.slug.as_str()))
    .bind(book.status.as_db())
    .bind(book.tier.as_db())
    .execute(&mut *tx)
    .await
    .map_err(|cause| Error::Database {
        writing: writing(),
        cause,
    })?;

    // Bound as text and cast, not as a json value: the column is `json` and
    // sqlx encodes `serde_json::Value` in jsonb's binary framing, which
    // postgres reads as a corrupt document rather than as json.
    sqlx::query(&format!(
        "INSERT INTO book_translations
             (book_id, locale, title, description,
              meta_title, meta_description, meta_keywords,
              og_title, og_description, og_image,
              metadata, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::json,
                 {STAMP}, {STAMP})
         ON CONFLICT (book_id, locale) DO UPDATE SET
             title            = EXCLUDED.title,
             description      = EXCLUDED.description,
             meta_title       = EXCLUDED.meta_title,
             meta_description = EXCLUDED.meta_description,
             meta_keywords    = EXCLUDED.meta_keywords,
             og_title         = EXCLUDED.og_title,
             og_description   = EXCLUDED.og_description,
             og_image         = EXCLUDED.og_image,
             metadata         = EXCLUDED.metadata,
             updated_at       = EXCLUDED.updated_at"
    ))
    .bind(id)
    .bind(LOCALE)
    .bind(&book.title)
    .bind(book.description.as_deref())
    .bind(book.seo.meta_title.as_deref())
    .bind(book.seo.meta_description.as_deref())
    .bind(book.seo.meta_keywords.as_deref())
    .bind(book.seo.og_title.as_deref())
    .bind(book.seo.og_description.as_deref())
    .bind(book.seo.og_image.as_deref())
    .bind(metadata(book))
    .execute(&mut *tx)
    .await
    .map_err(|cause| Error::Database {
        writing: writing(),
        cause,
    })?;

    Ok(())
}

/// One lesson: identity, ordering, and nothing a reader sees.
///
/// One statement, unlike [`book`], because a lesson has no translation row —
/// see the note at the top of this file. Everything readable about a lesson
/// comes from the content repo.
///
/// `chapter_id` and `sort_order` come from the entry rather than the yaml: the
/// chapter is whichever one listed this folder, and the order is the folder's
/// own number. A lesson never states either, so it cannot contradict the book.
async fn lesson(
    tx: &mut PgConnection,
    held: &LessonEntry,
    id: Uuid,
    book_id: Uuid,
    book_slug: &str,
) -> Result<(), Error> {
    let lesson = &held.lesson;
    let writing = || format!("lesson {:?} in {book_slug:?}", lesson.slug);

    sqlx::query(&format!(
        "INSERT INTO lessons
             (id, book_id, slug, chapter_id, sort_order,
              status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, {STAMP}, {STAMP})
         ON CONFLICT (id) DO UPDATE SET
             book_id    = EXCLUDED.book_id,
             slug       = EXCLUDED.slug,
             chapter_id = EXCLUDED.chapter_id,
             sort_order = EXCLUDED.sort_order,
             status     = EXCLUDED.status,
             updated_at = EXCLUDED.updated_at"
    ))
    .bind(id)
    .bind(book_id)
    .bind(&lesson.slug)
    .bind(held.chapter_id)
    .bind(held.sort_order)
    .bind(lesson.status.as_db())
    .execute(&mut *tx)
    .await
    .map_err(|cause| Error::Database {
        writing: writing(),
        cause,
    })?;

    Ok(())
}

/// The chapter titles, as the json the column holds.
///
/// Falls back to an empty list rather than propagating: serialising an `i32`
/// and two borrowed strings has no failure mode short of the allocator, and the
/// lints rule out asserting that with an `expect`.
fn metadata(book: &Book) -> String {
    serde_json::to_string(&Metadata {
        chapters: book
            .chapters
            .iter()
            .map(|chapter| ChapterEntry {
                id: chapter.id,
                title: &chapter.title,
            })
            .collect(),
    })
    .unwrap_or_else(|_| "{\"chapters\":[]}".to_owned())
}

/// Refuses a repo where anything to be written has no id.
///
/// Before the transaction rather than during it, so the answer is the whole
/// list in one run. `Book::id` is optional because a book without one is still
/// readable — it is only *ownable* that it cannot be. Syncing is where that
/// stops being enough: the id is the primary key, so there is no row to write
/// without it.
fn identified(snapshot: &Snapshot) -> Result<(), Error> {
    let mut offenders = Vec::new();

    for entry in snapshot.books() {
        let slug = &entry.book.slug;

        if entry.book.id.is_none() {
            offenders.push(format!("books/{slug}/book.yaml has no id"));
        }

        for held in entry.lessons() {
            if held.lesson.id.is_none() {
                offenders.push(format!(
                    "books/{slug}/lessons/{}/lesson.yaml has no id",
                    held.folder
                ));
            }
        }
    }

    if offenders.is_empty() {
        Ok(())
    } else {
        Err(Error::Incomplete(offenders))
    }
}

#[cfg(test)]
mod tests {
    use ohara::fixture;

    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot::load(&fixture::content()).unwrap()
    }

    #[test]
    fn a_fully_identified_repo_is_ready_to_sync() {
        assert!(identified(&snapshot()).is_ok());
    }

    #[test]
    fn every_missing_id_is_named_rather_than_the_first() {
        // Hand-built rather than a fixture: the shipped one is deliberately
        // correct, and a second broken repo on disk for one assertion is a
        // directory nobody would find their way back to.
        let message = Error::Incomplete(vec![
            "books/a/book.yaml has no id".to_owned(),
            "books/a/lessons/01-b/lesson.yaml has no id".to_owned(),
        ])
        .to_string();

        assert!(message.contains("books/a/book.yaml"), "{message}");
        assert!(message.contains("01-b/lesson.yaml"), "{message}");
    }

    #[test]
    fn the_chapter_metadata_is_the_json_the_column_holds() {
        let snapshot = snapshot();
        let book = snapshot.book("fixture-book").unwrap();

        let json = serde_json::to_string(&Metadata {
            chapters: book
                .book
                .chapters
                .iter()
                .map(|chapter| ChapterEntry {
                    id: chapter.id,
                    title: &chapter.title,
                })
                .collect(),
        })
        .unwrap();

        assert!(json.starts_with("{\"chapters\":["), "{json}");
        assert!(json.contains("\"id\":1"), "{json}");
    }

    #[test]
    fn a_database_failure_says_which_book_it_was_writing() {
        let message = Error::Database {
            writing: "book \"rust-101s\"".to_owned(),
            cause: sqlx::Error::RowNotFound,
        }
        .to_string();

        assert!(message.starts_with("failed to write book"), "{message}");
        assert!(message.contains("rust-101s"), "{message}");
    }
}
