//! What every listing endpoint takes, and the shape it answers with.
//!
//! Paging and search are not a notes problem. Books, projects, blog posts and
//! the admin screens all end up asking the same three questions — which page,
//! how big, matching what — and the wrong answers are the same every time: an
//! unclamped `per_page` is a caller choosing how much of a table to read, and
//! an unescaped term is a search for `%` returning everything.
//!
//! So the rules live here once. An endpoint declares its [`PageSize`] and
//! extracts a [`ListQuery`]; it does not get to re-decide what a page is.
//!
//! What comes back is `response`'s job — [`Paging`] is handed to
//! `response::PaginatedResponse`, which is where the envelope lives.

use axum::{
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::Response,
};
use serde::Deserialize;

use crate::response;

/// Past this there is nothing to show, and it is what stops
/// `(page - 1) * per_page` from overflowing on a hostile page number.
///
/// A hundred thousand pages of anything is not a reader paging through it.
const MAX_PAGE: i64 = 100_000;

/// The query parameters every listing accepts.
///
/// One struct rather than one per endpoint, so `?page=` means the same thing
/// everywhere and a new listing cannot invent `?p=` by accident. Endpoints that
/// need a filter of their own add it beside this, not inside it.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ListQuery {
    pub(crate) page: Option<i64>,
    pub(crate) per_page: Option<i64>,
    /// The search box. Reader input — [`ListQuery::pattern`] is the only
    /// sanctioned way to turn it into SQL.
    pub(crate) q: Option<String>,
}

impl ListQuery {
    /// The `%term%` pattern for an `ILIKE`, or `None` when nothing was typed.
    ///
    /// `None` rather than `%%` so the caller omits the clause entirely: a
    /// wildcard-only pattern makes postgres scan and match every row to reach
    /// the same answer as not filtering at all.
    pub(crate) fn pattern(&self) -> Option<String> {
        let term = self.q.as_deref().map(str::trim).unwrap_or_default();

        (!term.is_empty()).then(|| format!("%{}%", escape_like(term)))
    }

    pub(crate) fn paging(&self, size: PageSize) -> Paging {
        Paging::resolve(self.page, self.per_page, size)
    }
}

/// How big a page may be on one endpoint.
///
/// Per endpoint because the rows are not the same size: a note carries the
/// passage it was taken against and the note written on it, while a book
/// listing is a title and a slug. Fifty of one is not fifty of the other.
/// `max` is the ceiling whatever the caller asks for; `default` is what they
/// get for not asking. A plain struct literal rather than a constructor — an
/// endpoint that differs writes `PageSize { default: 10, max: 25 }`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PageSize {
    pub(crate) default: i64,
    pub(crate) max: i64,
}

impl PageSize {
    /// What a listing gets if it has no opinion.
    pub(crate) const DEFAULT: Self = Self {
        default: 20,
        max: 50,
    };
}

/// A page request after clamping, with the offset already worked out.
///
/// Constructed only by [`Paging::resolve`], so there is no way to hold one
/// carrying a `per_page` nobody checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Paging {
    /// What the page actually is, not what was asked for. Echoed back in
    /// [`PaginatedResponse`], so a caller sees `per_page=5000` became 50.
    pub(crate) page: i64,
    pub(crate) per_page: i64,
    pub(crate) offset: i64,
}

impl Paging {
    fn resolve(
        page: Option<i64>,
        per_page: Option<i64>,
        size: PageSize,
    ) -> Self {
        // Clamped rather than refused. A bad page number is a stale link or a
        // fat finger, and the first page is a better answer than an error.
        let per_page = per_page.unwrap_or(size.default).clamp(1, size.max);
        let page = page.unwrap_or(1).clamp(1, MAX_PAGE);

        Self {
            page,
            per_page,
            // Saturating as well as clamped: belt and braces on the one bit of
            // arithmetic here that a caller's number reaches.
            offset: (page - 1).saturating_mul(per_page),
        }
    }
}

/// Makes a search term mean literally what was typed.
///
/// `%` and `_` are `LIKE` wildcards, so a reader searching for `%` would
/// otherwise match every row they can see, and `_` would match any single
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

/// A json body, refused in the api's own shape when it cannot be read.
///
/// `axum::Json` answers a body it cannot parse with plain text, which no form
/// can show. This answers `{code, error}` like every other refusal. A missing
/// key is not this refusal's business: a form's required fields default to
/// empty, so the field is named by its own validation instead.
pub(crate) struct JsonBody<T>(pub(crate) T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(
        request: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(request, state).await {
            Ok(axum::Json(value)) => Ok(Self(value)),
            Err(rejection) => {
                tracing::info!(%rejection, "unreadable request body");
                Err(response::bad_request(
                    "body_unreadable",
                    "That request could not be read. Reload the page and try again.",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(
        page: Option<i64>,
        per_page: Option<i64>,
        q: Option<&str>,
    ) -> ListQuery {
        ListQuery {
            page,
            per_page,
            q: q.map(str::to_owned),
        }
    }

    #[test]
    fn absent_paging_is_the_first_page_at_the_default_size() {
        let paging = query(None, None, None).paging(PageSize::DEFAULT);

        assert_eq!(
            paging,
            Paging {
                page: 1,
                per_page: 20,
                offset: 0
            }
        );
    }

    #[test]
    fn a_page_size_is_clamped_at_both_ends() {
        let size = PageSize::DEFAULT;

        assert_eq!(query(None, Some(5), None).paging(size).per_page, 5);
        assert_eq!(query(None, Some(5_000), None).paging(size).per_page, 50);
        assert_eq!(query(None, Some(0), None).paging(size).per_page, 1);
        assert_eq!(query(None, Some(-1), None).paging(size).per_page, 1);
    }

    #[test]
    fn an_endpoint_can_set_its_own_size() {
        let small = PageSize {
            default: 5,
            max: 10,
        };

        assert_eq!(query(None, None, None).paging(small).per_page, 5);
        assert_eq!(query(None, Some(50), None).paging(small).per_page, 10);
    }

    #[test]
    fn a_hostile_page_number_cannot_overflow_the_offset() {
        let size = PageSize::DEFAULT;

        assert_eq!(query(Some(0), None, None).paging(size).page, 1);
        assert_eq!(query(Some(-9), None, None).paging(size).page, 1);
        assert_eq!(
            query(Some(i64::MAX), None, None).paging(size).page,
            MAX_PAGE
        );
        assert_eq!(query(Some(2), Some(10), None).paging(size).offset, 10);
    }

    #[test]
    fn nothing_typed_is_no_pattern_at_all() {
        assert_eq!(query(None, None, None).pattern(), None);
        assert_eq!(query(None, None, Some("")).pattern(), None);
        assert_eq!(query(None, None, Some("   ")).pattern(), None);
    }

    #[test]
    fn a_term_is_trimmed_and_wrapped() {
        assert_eq!(
            query(None, None, Some("  fork exec  "))
                .pattern()
                .as_deref(),
            Some("%fork exec%")
        );
    }

    #[test]
    fn wildcards_stop_being_wildcards() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("_"), "\\_");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
        assert_eq!(escape_like("%_\\"), "\\%\\_\\\\");

        // ...and the wrapping wildcards are still the pattern's own.
        assert_eq!(
            query(None, None, Some("%")).pattern().as_deref(),
            Some("%\\%%")
        );
    }

    #[test]
    fn an_ordinary_term_is_left_alone() {
        assert_eq!(escape_like("fork exec"), "fork exec");
        assert_eq!(escape_like("O'Brien"), "O'Brien");
        assert_eq!(escape_like(""), "");
    }
}
