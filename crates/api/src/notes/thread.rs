//! A lesson's public thread: the notes its readers chose to share, and the
//! replies under them.
//!
//! Rows come out of `super::store` flat — one query for a page of top-level
//! notes, one for every reply to those — and [`thread`] hangs each reply under
//! its root. Pure, so the grouping is tested without a database.
//!
//! **One level of nesting, and only one.** A write may only reply to a root —
//! see `super::target` — so nothing written here goes deeper. Imported laravel
//! rows are not held to that, and a reply to a reply is simply not shown:
//! finding its root would take a recursive query per page for rows the write
//! path no longer produces.

use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

use crate::response::as_utc;

/// One public note joined to its author, as both queries read it.
///
/// The author columns are all `Option`, `name` included though the column is
/// `NOT NULL`: the join is a `LEFT JOIN`, so a note never drops out of the
/// thread for want of a user row, and decoding has to allow for that.
#[derive(Debug, FromRow)]
pub(crate) struct CommentRow {
    pub(crate) id: i64,
    pub(crate) parent_id: Option<i64>,
    pub(crate) selected_text: Option<String>,
    pub(crate) note_content: Option<String>,
    pub(crate) start_offset: Option<i32>,
    pub(crate) end_offset: Option<i32>,
    pub(crate) created_at: Option<NaiveDateTime>,
    /// `notes.user_id`, which is `NOT NULL` — so there is always an id to
    /// compare against, even when the user row is gone.
    pub(crate) author_id: i64,
    pub(crate) author_name: Option<String>,
    pub(crate) author_username: Option<String>,
    pub(crate) author_avatar_url: Option<String>,
}

/// Who wrote a comment, and nothing else about them.
///
/// **Four fields, chosen, not a user row passed through.** No email, no
/// provider ids, no billing — a thread is read by anybody, and anything added
/// here is published to the internet.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct Author {
    /// What a client compares with its own reader id to mark its own
    /// comments. Done there rather than here so the response is the same bytes
    /// for everybody, which is what lets the edge hold it.
    pub(crate) id: i64,
    pub(crate) name: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) avatar_url: Option<String>,
}

/// One comment as it goes out, root or reply alike.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) id: i64,
    pub(crate) selected_text: Option<String>,
    /// `note_content` in the table and in a reader's own notes. `body` here
    /// because this is a different listing with a different reader: nobody
    /// reading a thread is editing a note.
    pub(crate) body: Option<String>,
    pub(crate) start_offset: Option<i32>,
    pub(crate) end_offset: Option<i32>,
    #[serde(serialize_with = "as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
    pub(crate) author: Author,
}

/// A top-level comment and its replies, oldest reply first.
///
/// Flattened rather than nested under a key, so a root and a reply read the
/// same and a reply is a root without `replies`.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct Comment {
    #[serde(flatten)]
    pub(crate) entry: Entry,
    pub(crate) replies: Vec<Entry>,
}

impl From<CommentRow> for Entry {
    fn from(row: CommentRow) -> Self {
        Self {
            id: row.id,
            selected_text: row.selected_text,
            body: row.note_content,
            start_offset: row.start_offset,
            end_offset: row.end_offset,
            created_at: row.created_at,
            author: Author {
                id: row.author_id,
                name: row.author_name,
                username: row.author_username,
                avatar_url: row.author_avatar_url,
            },
        }
    }
}

/// Hangs each reply under its root, keeping both orders as they came.
///
/// `roots` in the page's order, `replies` in the order they are to be shown
/// under each root. A reply whose parent is not among `roots` is dropped —
/// it belongs to another page, or to a reply (see the module docs).
pub(crate) fn thread(
    roots: Vec<CommentRow>,
    replies: Vec<CommentRow>,
) -> Vec<Comment> {
    let mut by_parent: HashMap<i64, Vec<Entry>> = HashMap::new();

    for reply in replies {
        if let Some(parent) = reply.parent_id {
            by_parent.entry(parent).or_default().push(reply.into());
        }
    }

    roots
        .into_iter()
        .map(|root| {
            let replies = by_parent.remove(&root.id).unwrap_or_default();

            Comment {
                entry: root.into(),
                replies,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, parent_id: Option<i64>) -> CommentRow {
        CommentRow {
            id,
            parent_id,
            selected_text: None,
            note_content: Some(format!("note {id}")),
            start_offset: None,
            end_offset: None,
            created_at: None,
            author_id: 7,
            author_name: Some("Ada".to_owned()),
            author_username: None,
            author_avatar_url: None,
        }
    }

    /// Each root's reply ids, in order — what every test here asserts on.
    fn replies(comments: &[Comment]) -> Vec<Vec<i64>> {
        comments
            .iter()
            .map(|c| c.replies.iter().map(|reply| reply.id).collect())
            .collect()
    }

    #[test]
    fn replies_hang_under_their_root_in_the_order_given() {
        let comments = thread(
            vec![row(3, None), row(1, None)],
            vec![row(10, Some(1)), row(11, Some(3)), row(12, Some(1))],
        );

        let roots: Vec<i64> = comments.iter().map(|c| c.entry.id).collect();
        assert_eq!(roots, vec![3, 1], "the page order was not kept");
        assert_eq!(replies(&comments), vec![vec![11], vec![10, 12]]);
    }

    #[test]
    fn a_root_with_no_replies_has_an_empty_list_not_none() {
        let comments = thread(vec![row(1, None)], vec![]);

        assert_eq!(replies(&comments), vec![Vec::<i64>::new()]);

        let json = serde_json::to_value(&comments).unwrap();
        assert_eq!(json.pointer("/0/replies"), Some(&serde_json::json!([])));
    }

    #[test]
    fn a_reply_to_something_not_on_this_page_is_dropped() {
        // 99 is on another page; 5 is itself a reply.
        let comments =
            thread(vec![row(1, None)], vec![row(2, Some(99)), row(6, Some(5))]);

        assert_eq!(replies(&comments), vec![Vec::<i64>::new()]);
    }

    #[test]
    fn no_roots_is_no_comments() {
        assert!(thread(vec![], vec![row(2, Some(1))]).is_empty());
    }

    #[test]
    fn a_comment_goes_out_flat_with_its_author_and_nothing_else() {
        let mut root = row(1, None);
        root.author_username = Some("ada".to_owned());
        root.created_at = Some(
            NaiveDateTime::parse_from_str(
                "2026-08-22 23:27:31",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
        );

        let json =
            serde_json::to_value(thread(vec![root], vec![row(2, Some(1))]))
                .unwrap();

        assert_eq!(
            json,
            serde_json::json!([{
                "id": 1,
                "selected_text": null,
                "body": "note 1",
                "start_offset": null,
                "end_offset": null,
                "created_at": "2026-08-22T23:27:31Z",
                "author": {
                    "id": 7,
                    "name": "Ada",
                    "username": "ada",
                    "avatar_url": null,
                },
                "replies": [{
                    "id": 2,
                    "selected_text": null,
                    "body": "note 2",
                    "start_offset": null,
                    "end_offset": null,
                    "created_at": null,
                    "author": {
                        "id": 7,
                        "name": "Ada",
                        "username": null,
                        "avatar_url": null,
                    },
                }],
            }])
        );
    }
}
