//! Every note query, and the only place that talks to postgres.
//!
//! A whole `query…bind` call lives here, never a SQL string handed elsewhere to
//! be filled in: `$1` and the value that fills it stay on screen together.
//!
//! Nothing here knows about HTTP. These return rows or [`StoreError`], and
//! `super::handler` decides what a caller is told.

use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::{note::Note, payload::ValidNote};
use crate::request::Paging;

/// From `20260822000000_baseline.sql`. They live beside the statement that can
/// trip them — a handler matching on constraint names would be a handler that
/// knows the schema.
const LESSON_FK: &str = "notes_lesson_id_foreign";
const PARENT_FK: &str = "notes_parent_id_foreign";

/// What can go wrong that a caller might answer differently.
#[derive(Debug)]
pub(crate) enum StoreError {
    /// Deleted between resolving it and the insert. Content sync removes
    /// lessons, so this is reachable.
    LessonGone,
    /// Deleted between the reply check and the insert. A reader can delete a
    /// note somebody else is replying to, so this is reachable too.
    ParentGone,
    /// Everything else, and nobody outside can act on it.
    Database(sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    /// A foreign key failing is not this process misbehaving — it is a row
    /// going away mid-write, which the caller can say something useful about.
    fn from(error: sqlx::Error) -> Self {
        let sqlx::Error::Database(ref database) = error else {
            return Self::Database(error);
        };

        match database.constraint() {
            Some(LESSON_FK) => Self::LessonGone,
            Some(PARENT_FK) => Self::ParentGone,
            _ => Self::Database(error),
        }
    }
}

/// Aliased to [`Note`]'s field names, which `FromRow` maps by.
const COLUMNS: &str = "
    n.id,
    n.selected_text,
    n.note_content,
    n.start_offset,
    n.end_offset,
    n.created_at,
    n.lesson_id,
    l.slug AS lesson_slug,
    b.slug AS book_slug,
    n.is_public,
    n.parent_id
";

/// Shared so [`page`] and [`count`] cannot answer about different sets — a
/// filter added to one and not the other gives twenty rows and a total of three.
///
/// Null-guarded rather than a clause that appears and disappears: a query built
/// two ways has its parameter numbering maintained in two places.
const OWNED_AND_MATCHING: &str = "
    WHERE n.user_id = $1
      AND (
          $2::text IS NULL
          OR n.selected_text ILIKE $2
          OR n.note_content ILIKE $2
      )
      AND ($3::uuid IS NULL OR n.lesson_id = $3)
";

/// The lesson two slugs name, or `None` when there is no such book or lesson.
///
/// Its own query rather than a reuse of [`lesson_and_parent`], which exists to
/// check a reply and answers about a parent nobody asked about here.
pub(crate) async fn lesson_id(
    db: &PgPool,
    book: &str,
    lesson: &str,
) -> Result<Option<Uuid>, StoreError> {
    let found = sqlx::query_as::<_, (Uuid,)>(
        r"
        SELECT l.id
        FROM lessons l
        JOIN books b ON b.id = l.book_id
        WHERE b.slug = $1
          AND l.slug = $2
        ",
    )
    .bind(book)
    .bind(lesson)
    .fetch_optional(db)
    .await?;

    Ok(found.map(|(id,)| id))
}

/// One page of a reader's notes, newest first.
pub(crate) async fn page(
    db: &PgPool,
    user_id: i64,
    pattern: Option<&str>,
    lesson_id: Option<Uuid>,
    paging: Paging,
) -> Result<Vec<Note>, StoreError> {
    let sql = format!(
        "
        SELECT {COLUMNS}
        FROM notes n
        JOIN lessons l ON l.id = n.lesson_id
        JOIN books b ON b.id = l.book_id
        {OWNED_AND_MATCHING}
        -- `id` breaks ties, so two notes saved in the same second cannot swap
        -- between pages, which is how a listing drops and repeats rows.
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $4
        OFFSET $5
        "
    );

    Ok(sqlx::query_as::<_, Note>(&sql)
        .bind(user_id)
        .bind(pattern)
        .bind(lesson_id)
        .bind(paging.per_page)
        .bind(paging.offset)
        .fetch_all(db)
        .await?)
}

/// How many the same filter matches, across every page.
///
/// No joins: `notes.lesson_id` and `lessons.book_id` are `NOT NULL` foreign
/// keys, so [`page`]'s inner joins cannot drop a row and counting without them
/// gives the same answer.
pub(crate) async fn count(
    db: &PgPool,
    user_id: i64,
    pattern: Option<&str>,
    lesson_id: Option<Uuid>,
) -> Result<i64, StoreError> {
    let sql = format!("SELECT count(n.id) FROM notes n {OWNED_AND_MATCHING}");

    let (total,) = sqlx::query_as::<_, (i64,)>(&sql)
        .bind(user_id)
        .bind(pattern)
        .bind(lesson_id)
        .fetch_one(db)
        .await?;

    Ok(total)
}

/// Writes a note and reads it back in one statement — a client needs the
/// timestamp postgres set and the slugs the row references but does not store.
pub(crate) async fn insert(
    db: &PgPool,
    user_id: i64,
    lesson_id: Uuid,
    payload: &ValidNote<'_>,
    is_public: bool,
    parent_id: Option<i64>,
) -> Result<Note, StoreError> {
    let sql = format!(
        "
        WITH inserted AS (
            INSERT INTO notes (
                user_id,
                lesson_id,
                note_content,
                selected_text,
                start_offset,
                end_offset,
                is_public,
                parent_id,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id,
                      selected_text,
                      note_content,
                      start_offset,
                      end_offset,
                      created_at,
                      lesson_id,
                      is_public,
                      parent_id
        )
        SELECT {COLUMNS}
        FROM inserted n
        JOIN lessons l ON l.id = n.lesson_id
        JOIN books b ON b.id = l.book_id
        "
    );

    Ok(sqlx::query_as::<_, Note>(&sql)
        .bind(user_id)
        .bind(lesson_id)
        .bind(payload.content)
        .bind(payload.selection)
        .bind(payload.offsets.map(|(start, _)| start))
        .bind(payload.offsets.map(|(_, end)| end))
        .bind(is_public)
        .bind(parent_id)
        .fetch_one(db)
        .await?)
}

/// Rewrites one note's body and reads it back, as [`insert`] does.
///
/// Scoped by owner in the statement, so there is no load-check-write window.
/// `None` means no such note *or* not this reader's — deliberately the same.
pub(crate) async fn rewrite(
    db: &PgPool,
    id: i64,
    user_id: i64,
    content: &str,
) -> Result<Option<Note>, StoreError> {
    let sql = format!(
        "
        WITH updated AS (
            UPDATE notes
            SET note_content = $3,
                updated_at = NOW()
            WHERE id = $1
              AND user_id = $2
            RETURNING id,
                      selected_text,
                      note_content,
                      start_offset,
                      end_offset,
                      created_at,
                      lesson_id,
                      is_public,
                      parent_id
        )
        SELECT {COLUMNS}
        FROM updated n
        JOIN lessons l ON l.id = n.lesson_id
        JOIN books b ON b.id = l.book_id
        "
    );

    Ok(sqlx::query_as::<_, Note>(&sql)
        .bind(id)
        .bind(user_id)
        .bind(content)
        .fetch_optional(db)
        .await?)
}

/// Deletes one note. `false` when nothing matched — see [`rewrite`].
pub(crate) async fn delete(
    db: &PgPool,
    id: i64,
    user_id: i64,
) -> Result<bool, StoreError> {
    let deleted =
        sqlx::query("DELETE FROM notes WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(db)
            .await?;

    Ok(deleted.rows_affected() > 0)
}

/// The lesson named by two slugs, and the shape of a candidate parent.
///
/// One query, because a parent is only valid *relative to* the lesson. The
/// `LEFT JOIN` leaves both booleans null when no parent was asked for *and*
/// when the id names nothing — the caller knows which it meant.
///
/// `None` means no such book or lesson.
pub(crate) async fn lesson_and_parent(
    db: &PgPool,
    book: &str,
    lesson: &str,
    parent_id: Option<i64>,
) -> Result<Option<(Uuid, Option<bool>, Option<bool>)>, StoreError> {
    Ok(sqlx::query_as::<_, (Uuid, Option<bool>, Option<bool>)>(
        r"
        SELECT l.id,
               p.parent_id IS NULL AS parent_is_root,
               p.lesson_id = l.id AS parent_is_here
        FROM lessons l
        JOIN books b ON b.id = l.book_id
        LEFT JOIN notes p ON p.id = $3
        WHERE b.slug = $1
          AND l.slug = $2
        ",
    )
    .bind(book)
    .bind(lesson)
    .bind(parent_id)
    .fetch_optional(db)
    .await?)
}
