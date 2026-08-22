//! A reader's own notes.
//!
//! ```text
//!   GET /api/notes?page=&per_page=&q=
//! ```
//!
//! Behind `require_session`, so there is no "whose notes" question to get
//! wrong: the layer resolved the cookie to a user id, and the `WHERE` clause
//! below binds that id rather than anything the caller sent. There is no
//! parameter that could name somebody else's notes.
//!
//! Reads only. Writes are the next slice, with the double-submit CSRF token
//! already sitting in the session payload.

use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::AppState,
    cache::{self, CachePolicy},
    session::Session,
};

/// A page nobody asked a size for.
const DEFAULT_PER_PAGE: i64 = 20;

/// The ceiling, whatever the caller asks for. Each row carries the selected
/// passage and the note written against it, so a page of these is not small.
const MAX_PER_PAGE: i64 = 50;

/// Past this there is nothing to show, and the offset multiplication has to
/// stop somewhere short of overflowing.
const MAX_PAGE: i64 = 100_000;

/// Absolute paths, so this merges alongside the signed routes rather than
/// nesting under the same `/api` prefix. The layer is applied where the router
/// is assembled — see `api::app`.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/notes", get(list))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    page: Option<i64>,
    per_page: Option<i64>,
    /// Matched against the selected passage and the note itself. Reader input,
    /// escaped by [`escape_like`] before it becomes a pattern.
    q: Option<String>,
}

/// One note, with enough of its lesson and book to link back to where it was
/// taken.
///
/// The field names are the column names. `note_content` reads oddly beside
/// `Note`, and it stays: this is the wire format, and renaming it here would
/// mean the json, the column and the laravel model all disagree.
#[derive(Debug, Serialize)]
#[allow(clippy::struct_field_names)]
struct Note {
    id: i64,
    selected_text: Option<String>,
    note_content: Option<String>,
    /// ISO-8601, formatted by postgres. The column is `timestamp without time
    /// zone` holding UTC, so the `Z` is added rather than converted — see the
    /// format string below.
    created_at: Option<String>,
    lesson_slug: String,
    lesson_title: Option<String>,
    book_slug: String,
    book_title: Option<String>,
}

/// The column order every query below selects, and the shape it decodes into.
type Row = (
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    String,
    Option<String>,
);

impl From<Row> for Note {
    fn from(
        (
            id,
            selected_text,
            note_content,
            created_at,
            lesson_slug,
            lesson_title,
            book_slug,
            book_title,
        ): Row,
    ) -> Self {
        Self {
            id,
            selected_text,
            note_content,
            created_at,
            lesson_slug,
            lesson_title,
            book_slug,
            book_title,
        }
    }
}

/// `page` and `per_page` echo back what the query resolved to after clamping,
/// so the caller can see that `per_page=5000` became 50 rather than guessing.
#[derive(Debug, Serialize)]
#[allow(clippy::struct_field_names)]
struct Page {
    notes: Vec<Note>,
    page: i64,
    per_page: i64,
    /// Across the whole filter, not this page — the frontend needs it to know
    /// whether there is another one.
    total: i64,
}

/// Titles come from the `en` translation.
///
/// Locale is a cookie the frontend resolves per request (docs/rebuild.md), and
/// passing it through to here is the content endpoints' job in a later slice.
/// Until then a note's link text is English even for a Bengali reader, which is
/// wrong but visible rather than silent.
const LOCALE: &str = "en";

const SELECT: &str = "SELECT n.id, n.selected_text, n.note_content, \
     to_char(n.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
     l.slug, lt.title, b.slug, bt.title \
     FROM notes n \
     JOIN lessons l ON l.id = n.lesson_id \
     JOIN books b ON b.id = l.book_id \
     LEFT JOIN lesson_translations lt ON lt.lesson_id = l.id AND lt.locale = $4 \
     LEFT JOIN book_translations bt ON bt.book_id = b.id AND bt.locale = $4 \
     WHERE n.user_id = $1";

const COUNT: &str = "SELECT count(*) FROM notes n WHERE n.user_id = $1";

/// `$5` in the select and `$2` in the count. Both `ILIKE`s use the same
/// pattern, so it is bound once.
const SEARCH: &str = " AND (n.selected_text ILIKE {} OR n.note_content ILIKE {})";

async fn list(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<ListQuery>,
) -> Response {
    let per_page = query
        .per_page
        .unwrap_or(DEFAULT_PER_PAGE)
        .clamp(1, MAX_PER_PAGE);
    let page = query.page.unwrap_or(1).clamp(1, MAX_PAGE);
    let offset = (page - 1).saturating_mul(per_page);

    let term = query.q.as_deref().map(str::trim).unwrap_or_default();
    let pattern = (!term.is_empty()).then(|| format!("%{}%", escape_like(term)));

    let mut select = SELECT.to_owned();
    let mut count = COUNT.to_owned();
    if pattern.is_some() {
        select.push_str(&SEARCH.replace("{}", "$5"));
        count.push_str(&SEARCH.replace("{}", "$2"));
    }
    // `id` breaks ties, so two notes saved in the same second cannot swap
    // between pages — the classic way a paginated list drops and repeats rows.
    select.push_str(" ORDER BY n.created_at DESC, n.id DESC LIMIT $2 OFFSET $3");

    let mut rows = sqlx::query_as::<_, Row>(&select)
        .bind(session.user_id)
        .bind(per_page)
        .bind(offset)
        .bind(LOCALE);
    let mut totals = sqlx::query_as::<_, (i64,)>(&count).bind(session.user_id);

    if let Some(pattern) = pattern.as_deref() {
        rows = rows.bind(pattern);
        totals = totals.bind(pattern);
    }

    let Ok(rows) = rows
        .fetch_all(&state.db)
        .await
        .inspect_err(|error| tracing::error!(%error, "cannot read notes"))
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Ok((total,)) = totals
        .fetch_one(&state.db)
        .await
        .inspect_err(|error| tracing::error!(%error, "cannot count notes"))
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let mut response = Json(Page {
        notes: rows.into_iter().map(Note::from).collect(),
        page,
        per_page,
        total,
    })
    .into_response();

    // One reader's own writing. No ETag either — revalidation would hand a
    // shared cache a reason to hold it.
    cache::apply(response.headers_mut(), CachePolicy::NoStore, None);

    response
}

/// Makes a search term mean literally what was typed.
///
/// `%` and `_` are `ILIKE` wildcards, so a reader searching for `%` would
/// otherwise match every note they own, and `_` would match any single
/// character. The backslash goes first because it is the escape character
/// itself. The value is still bound, never interpolated — this is about the
/// pattern language, not about injection.
fn escape_like(term: &str) -> String {
    let mut out = String::with_capacity(term.len());

    for c in term.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards_stop_being_wildcards() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("_"), "\\_");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
        assert_eq!(escape_like("%_\\"), "\\%\\_\\\\");
    }

    #[test]
    fn an_ordinary_term_is_left_alone() {
        assert_eq!(escape_like("fork exec"), "fork exec");
        assert_eq!(escape_like("O'Brien"), "O'Brien");
        assert_eq!(escape_like(""), "");
    }

    /// The clamping, as the handler does it. Extracted here rather than
    /// reaching into the handler, because what matters is the arithmetic: an
    /// unclamped `page` multiplied by `per_page` overflows, and an unclamped
    /// `per_page` is a caller choosing how much of the table to read.
    fn paging(page: Option<i64>, per_page: Option<i64>) -> (i64, i64, i64) {
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE);
        let page = page.unwrap_or(1).clamp(1, MAX_PAGE);

        (page, per_page, (page - 1).saturating_mul(per_page))
    }

    #[test]
    fn absent_paging_is_the_first_page() {
        assert_eq!(paging(None, None), (1, DEFAULT_PER_PAGE, 0));
    }

    #[test]
    fn a_page_size_is_clamped_at_both_ends() {
        assert_eq!(paging(None, Some(5)).1, 5);
        assert_eq!(paging(None, Some(5_000)).1, MAX_PER_PAGE);
        assert_eq!(paging(None, Some(0)).1, 1);
        assert_eq!(paging(None, Some(-1)).1, 1);
    }

    #[test]
    fn a_hostile_page_number_cannot_overflow_the_offset() {
        assert_eq!(paging(Some(0), None).0, 1);
        assert_eq!(paging(Some(-9), None).0, 1);
        assert_eq!(paging(Some(i64::MAX), None).0, MAX_PAGE);
        assert_eq!(paging(Some(2), Some(10)).2, 10);
    }
}
