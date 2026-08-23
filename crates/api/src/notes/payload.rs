//! What a reader sends, and whether it is acceptable.
//!
//! Decided without touching the database. What needs a query — does this
//! lesson exist, may this note reply to that one — is `super::target`'s job.

use serde::Deserialize;

use super::refusal::Refusal;

/// Characters, matching `note_content.max:500` in `StoreNoteRequest`. The
/// column is `text` and would take far more — this is editorial, not storage.
pub(crate) const MAX_NOTE: usize = 500;

/// Characters, matching `selected_text.max:5000`.
pub(crate) const MAX_SELECTION: usize = 5000;

/// What a reader may send when saving a note.
///
/// **No `user_id`.** It comes from the session, so there is no field here for a
/// caller to file a note under somebody else's name. The laravel controller got
/// there by spreading the input *then* overwriting `user_id` — one reordering
/// away from a bug.
#[derive(Debug, Deserialize)]
pub(crate) struct NewNoteRequest {
    pub(crate) book: String,
    pub(crate) lesson: String,
    pub(crate) note_content: String,
    pub(crate) selected_text: Option<String>,
    /// Character offsets over the rendered lesson body. Both or neither.
    pub(crate) start_offset: Option<i32>,
    pub(crate) end_offset: Option<i32>,
    /// Absent means public, matching `NoteController::store` — *not* the
    /// column's own default of `false`. During the crossover both stacks write
    /// this table, and disagreeing would make the same button mean two things.
    pub(crate) is_public: Option<bool>,
    /// The note this replies to. One level only — see `super::target`.
    pub(crate) parent_id: Option<i64>,
}

/// The only field an update may touch, matching laravel's `UpdateNoteRequest`.
/// Not `is_public` — a note that was public cannot go private after others have
/// replied to it. Not the selection — a note whose passage moved is a different
/// note.
#[derive(Debug, Deserialize)]
pub(crate) struct EditNoteRequest {
    pub(crate) note_content: String,
}

/// A new note's fields, checked, borrowing the request they came from.
pub(crate) struct ValidNote<'a> {
    pub(crate) content: &'a str,
    pub(crate) selection: Option<&'a str>,
    /// Start and end together, or neither.
    pub(crate) offsets: Option<(i32, i32)>,
}
pub(crate) fn validate_new_note(
    payload: &NewNoteRequest,
) -> Result<ValidNote<'_>, Refusal> {
    let content = validate_note_body(&payload.note_content)?;

    let selection = payload
        .selected_text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());

    if selection.is_some_and(|text| text.chars().count() > MAX_SELECTION) {
        return Err(Refusal::SelectionTooLong);
    }

    // Both or neither, and in order. Laravel checks each offset is a
    // non-negative integer and stops, so it accepts a lone start, or an end
    // before its start — anchors that can never resolve.
    let offsets = match (payload.start_offset, payload.end_offset) {
        (None, None) => None,
        (Some(start), Some(end)) if start >= 0 && end >= start => {
            Some((start, end))
        }
        _ => return Err(Refusal::BadAnchor),
    };

    Ok(ValidNote {
        content,
        selection,
        offsets,
    })
}

/// A note body, trimmed and checked.
///
/// Two deliberate differences from laravel: whitespace-only is refused, where
/// `required` accepts `"   "`; and the limit is characters, not bytes, so a
/// 500-character Bengali note is 500 rather than the 1500 bytes it stores as.
pub(crate) fn validate_note_body(content: &str) -> Result<&str, Refusal> {
    let content = content.trim();

    if content.is_empty() {
        return Err(Refusal::EmptyNote);
    }

    if content.chars().count() > MAX_NOTE {
        return Err(Refusal::NoteTooLong);
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_is_trimmed_and_must_have_something_in_it() {
        assert_eq!(validate_note_body("  a thought  "), Ok("a thought"));
        assert!(validate_note_body("   ").is_err());
        assert!(validate_note_body("").is_err());
        assert!(validate_note_body("\n\t").is_err());
    }

    #[test]
    fn a_note_is_measured_in_characters_not_bytes() {
        assert!(validate_note_body(&"a".repeat(MAX_NOTE)).is_ok());
        assert!(validate_note_body(&"a".repeat(MAX_NOTE + 1)).is_err());

        // Three bytes each in utf-8, so this passes only if characters count.
        let bengali = "আ".repeat(MAX_NOTE);
        assert!(
            bengali.len() > MAX_NOTE,
            "the fixture stopped being multibyte"
        );
        assert!(validate_note_body(&bengali).is_ok());
    }

    /// The offset rule, as `create` applies it. Laravel checks each offset is a
    /// non-negative integer and stops, so it accepts a start with no end and an
    /// end before its start — both anchors that can never resolve.
    fn anchor(
        start: Option<i32>,
        end: Option<i32>,
    ) -> Result<Option<(i32, i32)>, ()> {
        match (start, end) {
            (None, None) => Ok(None),
            (Some(start), Some(end)) if start >= 0 && end >= start => {
                Ok(Some((start, end)))
            }
            _ => Err(()),
        }
    }

    #[test]
    fn a_note_may_have_no_anchor_at_all() {
        assert_eq!(anchor(None, None), Ok(None));
    }

    #[test]
    fn an_anchor_needs_both_ends_in_order() {
        assert_eq!(anchor(Some(10), Some(40)), Ok(Some((10, 40))));
        // Zero-width is allowed: a caret with no selection is still a place.
        assert_eq!(anchor(Some(10), Some(10)), Ok(Some((10, 10))));

        assert!(anchor(Some(10), None).is_err());
        assert!(anchor(None, Some(40)).is_err());
        assert!(anchor(Some(40), Some(10)).is_err());
        assert!(anchor(Some(-1), Some(10)).is_err());
    }
}
