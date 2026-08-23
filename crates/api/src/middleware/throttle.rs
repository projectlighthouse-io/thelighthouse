//! How often one reader may write.

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header::RETRY_AFTER},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{api::AppState, session::Session};

/// Counts a write against the reader's budget, and refuses past it.
///
/// Keyed by user id rather than by address, so a university or an office is not
/// one bucket. The budget itself is `crate::limit`.
///
/// **Mount it inside [`super::reader`]**, for the reason [`super::csrf`] gives:
/// with no `Session` there is no reader to key on, and this refuses rather than
/// waving the request through.
pub(crate) async fn throttle(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(session) = request.extensions().get::<Session>() else {
        tracing::error!(
            "throttle ran without a session: it is mounted outside require_reader"
        );
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };

    let Some(retry_after) = state.limits.check(session.user_id) else {
        return next.run(request).await;
    };

    let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();

    // A 429 with no `Retry-After` tells a client to back off but not by how
    // much, so it guesses, and the guess is usually "immediately".
    if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
        response.headers_mut().insert(RETRY_AFTER, value);
    }

    response
}
