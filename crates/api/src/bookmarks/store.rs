//! Every bookmark query, and the only place that talks to postgres.
//!
//! A whole `query…bind` call lives here, never a SQL string handed elsewhere to
//! be filled in: `$1` and the value that fills it stay on screen together.
//!
//! Nothing here knows about HTTP. These return rows or [`StoreError`], and
//! `super::handler` decides what a caller is told.

use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::{bookmark::Bookmark, payload::ValidBookmark};

/// From `20260822000000_baseline.sql`. It lives beside the statement that can
/// trip it — a handler matching on constraint names would be a handler that
/// knows the schema.
const LESSON_FK: &str = "lesson_bookmarks_lesson_id_foreign";

/// What can go wrong that a caller might answer differently.
#[derive(Debug)]
pub(crate) enum StoreError {
    /// Deleted between resolving the lesson and the write. Content sync removes
    /// lessons, so this is reachable.
    LessonGone,
    /// Everything else, and nobody outside can act on it.
    Database(sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        let sqlx::Error::Database(ref database) = error else {
            return Self::Database(error);
        };

        match database.constraint() {
            Some(LESSON_FK) => Self::LessonGone,
            _ => Self::Database(error),
        }
    }
}

/// Aliased to [`Bookmark`]'s field names, which `FromRow` maps by.
const COLUMNS: &str = "
    m.id,
    m.selected_text,
    m.start_offset,
    m.end_offset,
    m.created_at,
    m.lesson_id,
    l.slug AS lesson_slug,
    b.slug AS book_slug
";

/// The lesson two slugs name, or `None` when there is no such book or lesson.
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

/// This reader's bookmark in one lesson, or `None`.
///
/// Scoped by owner in the statement, so there is no way to read somebody
/// else's by naming their lesson.
pub(crate) async fn find(
    db: &PgPool,
    user_id: i64,
    lesson_id: Uuid,
) -> Result<Option<Bookmark>, StoreError> {
    let sql = format!(
        "
        SELECT {COLUMNS}
        FROM lesson_bookmarks m
        JOIN lessons l ON l.id = m.lesson_id
        JOIN books b ON b.id = l.book_id
        WHERE m.user_id = $1
          AND m.lesson_id = $2
        "
    );

    Ok(sqlx::query_as::<_, Bookmark>(&sql)
        .bind(user_id)
        .bind(lesson_id)
        .fetch_optional(db)
        .await?)
}

/// Places a bookmark, replacing whatever this reader had in this lesson.
///
/// **Upsert rather than delete-then-insert.** The unique constraint says one
/// per reader per lesson, and two statements would leave a window where a
/// second request finds none and inserts a second row — which the constraint
/// then refuses, turning a re-bookmark into a 500. `ON CONFLICT` makes moving
/// a bookmark one statement, so there is no window to lose.
///
/// `created_at` is left alone on the update: it says when this reader first
/// marked their place, and moving the mark does not unmake that.
pub(crate) async fn place(
    db: &PgPool,
    user_id: i64,
    lesson_id: Uuid,
    payload: &ValidBookmark<'_>,
) -> Result<Bookmark, StoreError> {
    let sql = format!(
        "
        WITH placed AS (
            INSERT INTO lesson_bookmarks (
                user_id,
                lesson_id,
                selected_text,
                start_offset,
                end_offset,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (user_id, lesson_id) DO UPDATE
            SET selected_text = EXCLUDED.selected_text,
                start_offset = EXCLUDED.start_offset,
                end_offset = EXCLUDED.end_offset,
                updated_at = NOW()
            RETURNING id,
                      selected_text,
                      start_offset,
                      end_offset,
                      created_at,
                      lesson_id
        )
        SELECT {COLUMNS}
        FROM placed m
        JOIN lessons l ON l.id = m.lesson_id
        JOIN books b ON b.id = l.book_id
        "
    );

    Ok(sqlx::query_as::<_, Bookmark>(&sql)
        .bind(user_id)
        .bind(lesson_id)
        .bind(payload.selection)
        .bind(payload.start)
        .bind(payload.end)
        .fetch_one(db)
        .await?)
}

/// Removes this reader's bookmark. `false` when there was none.
pub(crate) async fn clear(
    db: &PgPool,
    user_id: i64,
    lesson_id: Uuid,
) -> Result<bool, StoreError> {
    let cleared = sqlx::query(
        "DELETE FROM lesson_bookmarks WHERE user_id = $1 AND lesson_id = $2",
    )
    .bind(user_id)
    .bind(lesson_id)
    .execute(db)
    .await?;

    Ok(cleared.rows_affected() > 0)
}
