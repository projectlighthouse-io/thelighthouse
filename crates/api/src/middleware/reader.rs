//! Resolves the session cookie to a reader — refusing, or shrugging.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{api::AppState, auth, cookie, session, session::Session};

/// Rejects anything without a live session, and puts the resolved one in
/// request extensions.
///
/// A layer rather than a call in each handler: an endpoint is protected because
/// of where it is mounted, so forgetting is not possible. A handler behind it
/// takes `Extension<Session>` and can be sure of it.
///
/// **401, not 404.** Unlike the luxctl surface these routes are the frontend's
/// and the browser already knows they exist, so hiding them buys nothing and
/// would make an expired session look like a typo in a URL.
pub(crate) async fn require_reader(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let deny = || StatusCode::UNAUTHORIZED.into_response();

    let Some(id) = cookie::read(request.headers(), auth::SESSION_COOKIE) else {
        return deny();
    };

    // Cloned because `load` borrows the pool out of `state` while the cookie is
    // still borrowed out of `request`, which is then handed on mutably.
    let id = id.to_owned();

    let Some(session) = session::load(&state.db, &id).await else {
        return deny();
    };

    request.extensions_mut().insert(session);

    next.run(request).await
}

/// The reader behind the session cookie, if there is one.
///
/// The shrugging half of [`require_reader`], for a route that serves everybody
/// but serves a reader more. There is no layer for it because there is nothing
/// to refuse: a handler that wants this wants the `Option`, and hiding that in
/// an extension would let a route quietly stop checking it.
pub(crate) async fn optional(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<Session> {
    let id = cookie::read(headers, auth::SESSION_COOKIE)?;

    session::load(&state.db, id).await
}
