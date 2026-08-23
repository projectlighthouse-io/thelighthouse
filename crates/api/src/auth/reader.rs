//! Who the reader is, and how they stop being one.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::{READER_COOKIE, SESSION_COOKIE};
use crate::{
    api::AppState,
    cache::{self, CachePolicy},
    cookie::{self, Cookie},
    session, users,
};

/// What the frontend renders chrome from. Never the source of an entitlement —
/// anything paid is decided server side, against the session row, not this.
#[derive(Debug, Serialize)]
pub(crate) struct ReaderResponse {
    /// The `users.id`, as a string.
    ///
    /// It used to be `provider:id`, because there was no users row to point at.
    /// The frontend never parses it — it is an opaque identity — so the shape
    /// did not have to change when the meaning did.
    sub: String,
    provider: String,
    name: String,
    email: String,
    avatar: Option<String>,
    /// The CSRF token for this session, which the frontend echoes in a header
    /// on every write — see `middleware::csrf::CSRF_HEADER`.
    ///
    /// Handed out here rather than in a second readable cookie. A cookie can be
    /// written by any sibling subdomain, so comparing a header against a cookie
    /// proves less than comparing it against the session row; this endpoint
    /// already requires the session cookie to answer at all, so the token only
    /// ever reaches the reader it belongs to.
    ///
    /// It is exactly as exposed to XSS as any token a page has to send, which
    /// is to say a script running on this origin defeats CSRF protection
    /// whatever shape it takes.
    csrf: String,
}

/// Who is this, or `null`.
///
/// 200 with a null body rather than a 401: "nobody" is a correct answer to the
/// question, not a failure to answer it. A 401 would make the anonymous case —
/// the common one — an error in every client that calls this.
///
/// Two queries: the session, then the reader it points at. A join would make it
/// one, at the cost of putting the expiry and sliding-refresh rules in two
/// places. This runs once per page load for signed-in readers only.
pub(crate) async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let reader = resolve(&state, &headers).await;

    let mut response = Json(reader).into_response();
    // Describes one reader. No ETag either: revalidation would hand a shared
    // cache a reason to hold it.
    cache::apply(response.headers_mut(), CachePolicy::NoStore, None);
    response
}

/// The cookie, resolved all the way to a reader, or `None` at the first step
/// that does not answer.
pub(crate) async fn resolve(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<ReaderResponse> {
    let id = cookie::read(headers, SESSION_COOKIE)?;
    let session = session::load(&state.db, id).await?;

    let reader = users::find(&state.db, session.user_id)
        .await
        .inspect_err(
            |error| tracing::error!(%error, user_id = session.user_id, "failed to read the reader"),
        )
        .ok()??;

    Some(ReaderResponse {
        sub: reader.id.to_string(),
        provider: session.provider,
        name: reader.name,
        email: reader.email,
        avatar: reader.avatar,
        csrf: session.csrf,
    })
}

/// Clears the cookie and deletes the row. POST, not GET, so a prefetcher or an
/// `<img>` on another site cannot sign a reader out.
///
/// **Deleting the row is what makes this real revocation.** The stateless
/// session this replaced could only clear the reader's own copy; a copy lifted
/// off that browser stayed valid until it expired.
///
/// `SameSite=Lax` already stops the cross-site form POST from carrying the
/// cookie, so this needs no CSRF token of its own. The mutations that do —
/// anything that changes stored state — get the double-submit token described
/// in docs/rebuild.md when they exist.
pub(crate) async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Some(id) = cookie::read(&headers, SESSION_COOKIE) {
        session::delete(&state.db, id).await;
    }

    let secure = state.config.cookie_secure();
    let mut response = StatusCode::NO_CONTENT.into_response();
    let headers = response.headers_mut();

    // The path must match the one it was set with, or the browser keeps the
    // original and the reader stays signed in.
    cookie::attach(
        headers,
        &[
            Cookie::expiring(SESSION_COOKIE).to_header(secure),
            // Cleared with the session, or the frontend keeps drawing a signed-in
            // header for somebody who is not.
            Cookie::expiring(READER_COOKIE)
                .script_readable()
                .to_header(secure),
        ],
    );
    cache::apply(headers, CachePolicy::NoStore, None);

    response
}
