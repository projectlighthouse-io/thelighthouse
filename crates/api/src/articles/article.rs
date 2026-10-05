//! An article as it is read out of postgres and written onto the wire.
//!
//! Two shapes, because two audiences see different things. [`Article`] is what
//! anybody may read; [`OwnArticle`] is what its author reads, and is the only
//! one carrying a takedown.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

use crate::response::as_utc;

/// One article, as /blog and an article page see it.
///
/// Both a row and a response, which is why it is named after neither — the
/// field names are the column names and the wire names at once, as `Note`'s
/// are, so every query has to alias to them and a missing alias fails at
/// decode rather than shifting every field by one.
///
/// **No takedown columns.** This shape is only ever built from live rows, and
/// leaving the columns off means a query that forgot the filter cannot leak a
/// takedown reason through a public route — it fails to decode instead.
#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Article {
    pub(crate) id: i64,
    /// How the article is addressed. Minted once and never changed.
    pub(crate) slug: String,
    pub(crate) title: String,
    /// The line under the title — what a reader decides from on the listing,
    /// and what a link preview shows. Author-written, never derived.
    pub(crate) subtitle: String,
    /// What the article is about, from `article_topics`. Each is one of
    /// `payload::CATEGORIES`; there is always at least one.
    pub(crate) topics: Vec<String>,
    /// Markdown as the author typed it. Rendering it is the frontend's job,
    /// and so is escaping whatever html is in it — see this module's `mod.rs`.
    pub(crate) body: String,
    /// Who wrote it, for the byline. Read through a join rather than stored:
    /// a reader who changes their name changes it on everything they wrote.
    pub(crate) author: String,
    pub(crate) author_username: Option<String>,
    #[serde(serialize_with = "as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
    #[serde(serialize_with = "as_utc")]
    pub(crate) updated_at: Option<NaiveDateTime>,
}

/// One of the author's own articles, live or not.
///
/// The takedown is flattened onto it rather than being a nested object,
/// because the two columns are written together and are null together — see
/// `articles_takedown_has_a_reason`. Nothing in this crate writes them; a
/// takedown is an `UPDATE` run by hand, and this shape is how the author finds
/// out it happened.
#[derive(Debug, Serialize, FromRow)]
pub(crate) struct OwnArticle {
    pub(crate) id: i64,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) topics: Vec<String>,
    pub(crate) body: String,
    /// The same byline [`Article`] carries. Redundant on `/api/articles/mine`,
    /// where the author is the reader — but `/api/articles?author=` answers in
    /// this shape too, and a listing whose rows sometimes have a byline is a
    /// listing the frontend has to special-case.
    pub(crate) author: String,
    pub(crate) author_username: Option<String>,
    #[serde(serialize_with = "as_utc")]
    pub(crate) created_at: Option<NaiveDateTime>,
    #[serde(serialize_with = "as_utc")]
    pub(crate) updated_at: Option<NaiveDateTime>,
    /// Null while the article is live. Set means it is not being served, and
    /// [`Self::taken_down_reason`] says why. Written by hand, never by a
    /// handler — see this module's `mod.rs`.
    #[serde(serialize_with = "as_utc")]
    pub(crate) taken_down_at: Option<NaiveDateTime>,
    /// Why it was taken down, shown to the author and to nobody else.
    pub(crate) taken_down_reason: Option<String>,
    /// Set when the author archived it: off the public listing and its page,
    /// still on their own shelf, and theirs to bring back.
    #[serde(serialize_with = "as_utc")]
    pub(crate) archived_at: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A json pointer, so a missing field names itself rather than tripping
    /// the indexing lint or panicking three lines later as a null comparison.
    fn field<'v>(
        value: &'v serde_json::Value,
        name: &str,
    ) -> &'v serde_json::Value {
        value
            .pointer(&format!("/{name}"))
            .unwrap_or_else(|| panic!("no {name} in {value}"))
    }

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn live() -> Article {
        Article {
            id: 1,
            slug: "why-fork-is-strange-a3f19c".to_owned(),
            title: "Why fork is strange".to_owned(),
            subtitle: "A syscall that returns twice.".to_owned(),
            topics: vec!["systems".to_owned(), "containers".to_owned()],
            body: "# Heading\n\nsome *markdown*".to_owned(),
            author: "Ada".to_owned(),
            author_username: Some("ada".to_owned()),
            created_at: Some(at("2026-09-04 10:00:00")),
            updated_at: Some(at("2026-09-04 10:30:00")),
        }
    }

    #[test]
    fn an_article_carries_every_topic_it_is_about() {
        let json = serde_json::to_value(live()).unwrap();
        let topics = field(&json, "topics").as_array().expect("not an array");

        assert_eq!(
            topics
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>(),
            vec!["systems", "containers"]
        );
    }

    #[test]
    fn a_public_article_cannot_carry_a_takedown() {
        let json = serde_json::to_string(&live()).unwrap();

        assert!(!json.contains("taken_down"), "{json}");
    }

    #[test]
    fn the_body_goes_out_as_the_markdown_it_was_written_as() {
        let json = serde_json::to_value(live()).unwrap();

        assert_eq!(field(&json, "body"), "# Heading\n\nsome *markdown*");
    }

    #[test]
    fn a_timestamp_goes_out_as_utc_with_the_offset_stated() {
        let json = serde_json::to_value(live()).unwrap();

        assert_eq!(field(&json, "created_at"), "2026-09-04T10:00:00Z");
        assert_eq!(field(&json, "updated_at"), "2026-09-04T10:30:00Z");
    }

    #[test]
    fn an_authors_own_article_carries_why_it_was_taken_down() {
        let json = serde_json::to_value(OwnArticle {
            id: 1,
            slug: "why-fork-is-strange-a3f19c".to_owned(),
            title: "Why fork is strange".to_owned(),
            subtitle: "A syscall that returns twice.".to_owned(),
            topics: vec!["systems".to_owned()],
            body: "text".to_owned(),
            author: "Ada".to_owned(),
            author_username: Some("ada".to_owned()),
            created_at: Some(at("2026-09-04 10:00:00")),
            updated_at: Some(at("2026-09-04 10:00:00")),
            taken_down_at: Some(at("2026-09-04 12:00:00")),
            taken_down_reason: Some("Reposted without attribution.".to_owned()),
            archived_at: None,
        })
        .unwrap();

        assert_eq!(field(&json, "taken_down_at"), "2026-09-04T12:00:00Z");
        assert_eq!(
            field(&json, "taken_down_reason"),
            "Reposted without attribution."
        );
    }

    #[test]
    fn a_live_article_of_the_authors_own_says_so_with_nulls() {
        let json = serde_json::to_value(OwnArticle {
            id: 1,
            slug: "s".to_owned(),
            title: "t".to_owned(),
            subtitle: "s".to_owned(),
            topics: vec!["go".to_owned()],
            body: "b".to_owned(),
            author: "Ada".to_owned(),
            author_username: None,
            created_at: None,
            updated_at: None,
            taken_down_at: None,
            taken_down_reason: None,
            archived_at: None,
        })
        .unwrap();

        assert!(field(&json, "taken_down_at").is_null());
        assert!(field(&json, "taken_down_reason").is_null());
    }
}
