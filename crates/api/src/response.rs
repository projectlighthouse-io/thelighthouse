//! What every endpoint answers with.
//!
//! The mirror of `request`. Notes, lessons, projects and the admin screens all
//! return the same three shapes — a page of things, one thing, or a
//! refusal — and the parts easy to get wrong are the same every time: a body
//! that forgot its cache policy, or a refusal whose message reads like a log
//! line.
//!
//! So a handler picks a status and a value, and the policy comes with the
//! helper rather than being remembered.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{
    cache::{self, CachePolicy},
    request::Paging,
};

/// One page of results, as every listing answers.
///
/// `items` rather than a name per endpoint, so a client can read the envelope
/// without knowing what is in it — and so the next listing does not copy this
/// struct with one word changed.
///
/// `page` and `per_page` echo the request back after clamping, under the same
/// two names the query parameters use — a caller comparing what it asked for
/// with what it got should not have to translate between them.
#[derive(Debug, Serialize)]
pub(crate) struct PaginatedResponse<T> {
    pub(crate) items: Vec<T>,
    pub(crate) page: i64,
    pub(crate) per_page: i64,
    /// Across the whole filter, not this page. Without it a client cannot tell
    /// a full last page from there being more.
    pub(crate) total: i64,
}

impl<T> PaginatedResponse<T> {
    pub(crate) fn new(items: Vec<T>, paging: Paging, total: i64) -> Self {
        Self {
            items,
            page: paging.page,
            per_page: paging.per_page,
            total,
        }
    }
}

/// A refusal, in the one shape every client can read.
///
/// One field, always named `error`, so a frontend has a single place to look
/// however the request failed — see `useNotes`'s `problem()`, which reads it
/// and falls back to its own wording when it is absent.
#[derive(Debug, Serialize)]
struct ErrorResponse<'m> {
    /// Stable and never translated — what a client branches on.
    code: &'m str,
    /// English, and a fallback. A client with its own wording for `code` shows
    /// that instead; one without shows this rather than nothing.
    error: &'m str,
}

/// A json body with the cache policy stated rather than defaulted.
///
/// Every response goes through here so no handler can return a body with no
/// `Cache-Control` — the failure that does not show up in testing and then
/// hands one reader's page to a shared cache.
pub(crate) fn json<T: Serialize>(
    status: StatusCode,
    value: T,
    cache_policy: CachePolicy,
) -> Response {
    let mut response = (status, Json(value)).into_response();
    cache::apply(response.headers_mut(), cache_policy, None);

    response
}

/// A response with no body — a 204, or a status-only refusal.
///
/// Separate from [`json`] because a 204 with a body is not a 204. The policy is
/// still required: an empty 404 that a cache holds is a 404 that outlives the
/// row being created.
pub(crate) fn empty(status: StatusCode, cache_policy: CachePolicy) -> Response {
    let mut response = status.into_response();
    cache::apply(response.headers_mut(), cache_policy, None);

    response
}

/// What a 500 says, and the only thing it says.
///
/// No detail, ever. Whatever actually broke — a constraint name, a column, a
/// connection string in a driver message — is a description of this system's
/// insides, and the caller can do nothing with it. It goes to the log, where
/// somebody who can fix it will read it, with the ids that make it findable.
const SERVER_ERROR: &str = "Something went wrong. Please try again.";

/// 500, said the same way every time.
///
/// The error itself is logged at the call site — this is only what the caller
/// is told, and there is exactly one version of it so no handler can invent a
/// more revealing one.
pub(crate) fn server_error() -> Response {
    json(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorResponse {
            code: "server_error",
            error: SERVER_ERROR,
        },
        CachePolicy::NoStore,
    )
}

/// 404, with no body.
///
/// Nothing to say: whether the thing never existed, was deleted, or belongs to
/// somebody else, the answer is the same and saying more would tell a prober
/// which it was.
pub(crate) fn not_found() -> Response {
    empty(StatusCode::NOT_FOUND, CachePolicy::NoStore)
}

/// 400, with something to show the person who typed it.
///
/// The message is for a reader, not for whoever reads the log — the laravel
/// app's `messages()` on each form request does the same job. Never cached: a
/// refusal is about one attempt.
pub(crate) fn bad_request(code: &str, message: &str) -> Response {
    json(
        StatusCode::BAD_REQUEST,
        ErrorResponse {
            code,
            error: message,
        },
        CachePolicy::NoStore,
    )
}

#[cfg(test)]
mod tests {
    use axum::http::header::CACHE_CONTROL;

    use super::*;

    fn cache_control(response: &Response) -> String {
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned()
    }

    #[test]
    fn a_refusal_is_a_400_that_nothing_may_store() {
        let response = bad_request("note_empty", "Please enter a note.");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(cache_control(&response).contains("no-store"));
    }

    #[test]
    fn a_body_carries_the_policy_it_was_given() {
        let shared = json(StatusCode::OK, "body", CachePolicy::public_content());
        assert!(cache_control(&shared).contains("s-maxage"));

        let private = json(StatusCode::OK, "body", CachePolicy::Private);
        assert!(cache_control(&private).starts_with("private"));
    }

    #[test]
    fn a_page_echoes_back_what_the_paging_resolved_to() {
        let paging = crate::request::ListQuery {
            page: Some(3),
            per_page: Some(5_000),
            q: None,
        }
        .paging(crate::request::PageSize::DEFAULT);

        let page = PaginatedResponse::new(vec!["a", "b"], paging, 107);

        assert_eq!(page.page, 3);
        assert_eq!(page.per_page, 50, "the clamp did not reach the response");
        assert_eq!(page.total, 107);
        assert_eq!(page.items.len(), 2);
    }
}
