//! A bookmark as it is read out of postgres and written onto the wire.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::response::as_utc;

/// One bookmark: the place a reader left off in one lesson.
///
/// **At most one per reader per lesson** — `lesson_bookmarks_user_id_lesson_id_unique`
/// — so saving a second replaces the first rather than adding to it. That is
/// what makes it a different thing from a note: notes accumulate, a bookmark
/// moves.
///
/// Both a row and a response, like `notes::Note`, and for the same reason: the
/// field names are the column names and the wire names at once, so every query
/// has to alias to them and a missing alias fails at decode.
#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Bookmark {
    pub(crate) id: i64,
    pub(crate) selected_text: String,
    /// Character offsets over the rendered lesson body. `NOT NULL` here, unlike
    /// a note's: a bookmark with no anchor is not a place.
    pub(crate) start_offset: i32,
    pub(crate) end_offset: i32,
    #[serde(serialize_with = "as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
    /// A uuid the content repo mints — see
    /// `20260828020000_content_owns_its_ids.sql`.
    pub(crate) lesson_id: Uuid,
    /// A slug is unique only within a book — `lessons_book_id_slug_unique` — so
    /// both are needed to name one lesson.
    pub(crate) lesson_slug: String,
    pub(crate) book_slug: String,
}
