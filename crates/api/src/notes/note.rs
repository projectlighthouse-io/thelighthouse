//! A note as it is read out of postgres and written onto the wire.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::response::as_utc;

/// One note, with enough of its lesson and book to link back to it.
///
/// Both a row and a response, which is why it is named after neither. The field
/// names are the column names and the wire names at once: `FromRow` maps by
/// name, so every query has to alias to these, and a missing alias fails at
/// decode rather than shifting every field by one.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Note {
    pub(crate) id: i64,
    pub(crate) selected_text: Option<String>,
    pub(crate) note_content: Option<String>,
    /// Character offsets over the rendered lesson body — where the passage
    /// this note was taken against sits. Null on a note written with nothing
    /// selected, and the reader page draws a highlight only when both are set.
    pub(crate) start_offset: Option<i32>,
    pub(crate) end_offset: Option<i32>,
    #[serde(serialize_with = "as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
    /// A uuid the content repo mints — see
    /// `20260828020000_content_owns_its_ids.sql`. Not a number, and never
    /// one the database issued.
    pub(crate) lesson_id: Uuid,
    /// A slug is unique only within a book — `lessons_book_id_slug_unique` —
    /// so both are needed to name one lesson.
    pub(crate) lesson_slug: String,
    pub(crate) book_slug: String,
    /// Whether the lesson's thread shows this to other readers.
    pub(crate) is_public: bool,
    pub(crate) parent_id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialised whole, so the timestamp is checked through the path the
    /// endpoint uses rather than by calling the serialiser directly.
    fn json_of(created_at: Option<NaiveDateTime>) -> String {
        serde_json::to_string(&Note {
            id: 1,
            selected_text: None,
            note_content: None,
            start_offset: None,
            end_offset: None,
            created_at,
            lesson_id: Uuid::nil(),
            lesson_slug: "goroutines".to_owned(),
            book_slug: "go-fundamentals".to_owned(),
            is_public: true,
            parent_id: None,
        })
        .unwrap()
    }

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn a_timestamp_goes_out_as_utc_with_the_offset_stated() {
        assert!(
            json_of(Some(at("2026-08-22 23:27:31")))
                .contains(r#""created_at":"2026-08-22T23:27:31Z""#)
        );
    }

    #[test]
    fn a_timestamp_is_not_shifted_on_the_way_out() {
        assert!(
            json_of(Some(at("2026-01-01 00:00:00")))
                .contains(r#""created_at":"2026-01-01T00:00:00Z""#)
        );
        assert!(
            json_of(Some(at("2026-12-31 23:59:59")))
                .contains(r#""created_at":"2026-12-31T23:59:59Z""#)
        );
    }

    #[test]
    fn a_note_with_no_timestamp_is_null_rather_than_an_epoch() {
        assert!(json_of(None).contains(r#""created_at":null"#));
    }
}
