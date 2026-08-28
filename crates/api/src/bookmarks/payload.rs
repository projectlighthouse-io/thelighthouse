//! What a reader sends when placing a bookmark, and whether it is acceptable.
//!
//! Decided without touching the database. Which lesson it lands on is a slug
//! pair in the path, and resolving that is `super::store`'s job.

use serde::Deserialize;

use super::refusal::Refusal;

/// Characters. `selected_text` is `varchar(500)`, so anything longer is a write
/// postgres refuses with a 500 — this turns that into a refusal a reader can
/// act on. Notes allow far more because their column is `text`.
pub(crate) const MAX_SELECTION: usize = 500;

/// The passage a bookmark points at.
///
/// **No lesson, and no `user_id`.** The lesson is in the path and the reader is
/// the session, so there is no field here for a caller to bookmark somebody
/// else's lesson under somebody else's name.
#[derive(Debug, Deserialize)]
pub(crate) struct NewBookmarkRequest {
    pub(crate) selected_text: String,
    /// Character offsets over the rendered lesson body. Both required — a
    /// bookmark with no anchor is not a place.
    pub(crate) start_offset: i32,
    pub(crate) end_offset: i32,
}

/// A bookmark's fields, checked, borrowing the request they came from.
pub(crate) struct ValidBookmark<'a> {
    pub(crate) selection: &'a str,
    pub(crate) start: i32,
    pub(crate) end: i32,
}

pub(crate) fn validate(
    payload: &NewBookmarkRequest,
) -> Result<ValidBookmark<'_>, Refusal> {
    let selection = payload.selected_text.trim();

    if selection.is_empty() {
        return Err(Refusal::EmptySelection);
    }

    if selection.chars().count() > MAX_SELECTION {
        return Err(Refusal::SelectionTooLong);
    }

    // In order and on the page. A zero-width anchor is allowed for the same
    // reason a note's is: a caret with nothing selected is still a place.
    if payload.start_offset < 0 || payload.end_offset < payload.start_offset {
        return Err(Refusal::BadAnchor);
    }

    Ok(ValidBookmark {
        selection,
        start: payload.start_offset,
        end: payload.end_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        selected_text: &str,
        start_offset: i32,
        end_offset: i32,
    ) -> NewBookmarkRequest {
        NewBookmarkRequest {
            selected_text: selected_text.to_owned(),
            start_offset,
            end_offset,
        }
    }

    #[test]
    fn a_selection_is_trimmed_and_must_have_something_in_it() {
        let padded = request("  a passage  ", 0, 9);
        assert_eq!(validate(&padded).unwrap().selection, "a passage");

        assert!(validate(&request("   ", 0, 9)).is_err());
        assert!(validate(&request("", 0, 9)).is_err());
    }

    #[test]
    fn a_selection_is_measured_in_characters_not_bytes() {
        assert!(validate(&request(&"a".repeat(MAX_SELECTION), 0, 1)).is_ok());
        assert!(
            validate(&request(&"a".repeat(MAX_SELECTION + 1), 0, 1)).is_err()
        );

        // Three bytes each in utf-8, so this passes only if characters count.
        let bengali = "আ".repeat(MAX_SELECTION);
        assert!(
            bengali.len() > MAX_SELECTION,
            "the fixture stopped being multibyte"
        );
        assert!(validate(&request(&bengali, 0, 1)).is_ok());
    }

    #[test]
    fn an_anchor_must_be_in_order_and_on_the_page() {
        assert!(validate(&request("a passage", 10, 40)).is_ok());
        // Zero-width: a caret with no selection is still a place.
        assert!(validate(&request("a passage", 10, 10)).is_ok());

        assert!(validate(&request("a passage", 40, 10)).is_err());
        assert!(validate(&request("a passage", -1, 10)).is_err());
    }
}
