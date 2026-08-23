//! The double-submit token every state-changing request carries.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

use crate::session::Session;

/// Where the frontend sends the token it read from `/api/auth/session`.
///
/// A header rather than a form field: a cross-site form POST can carry any body
/// it likes, but it cannot set a custom header without a CORS preflight that
/// this origin never answers.
pub(crate) const CSRF_HEADER: &str = "x-csrf-token";

/// Refuses a request whose token does not match the session's.
///
/// `SameSite=Lax` already stops the common cross-site form POST and is not
/// enough on its own (docs/rebuild.md). The token is minted with the session and
/// lives in its row, so this compares what the caller sent against what this
/// process stored — a plain double-submit trusts a cookie an attacker on a
/// sibling subdomain can write.
///
/// **Mount it inside [`super::reader`].** With no `Session` in extensions there
/// is nothing to compare against, and this refuses rather than skips, so the
/// wrong order shows up as uniform 403s rather than a check that silently
/// stopped running.
pub(crate) async fn require_csrf(request: Request, next: Next) -> Response {
    let forbid = || StatusCode::FORBIDDEN.into_response();

    let Some(session) = request.extensions().get::<Session>().cloned() else {
        tracing::error!(
            "csrf ran without a session: it is mounted outside require_reader"
        );
        return forbid();
    };

    let provided = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !matches(&session.csrf, provided) {
        tracing::info!(
            user_id = session.user_id,
            "a write arrived without a valid csrf token"
        );
        // 403, not 401: the reader is signed in. A 401 would send the frontend
        // to re-authenticate, which cannot fix a missing header and would look
        // like being signed out at random.
        return forbid();
    }

    next.run(request).await
}

/// Constant time, and an empty expectation never matches — the same rules the
/// oauth state comparison follows, for the same reason.
fn matches(expected: &str, provided: &str) -> bool {
    if expected.is_empty() {
        return false;
    }

    let (expected, provided) = (expected.as_bytes(), provided.as_bytes());

    expected.len() == provided.len() && bool::from(expected.ct_eq(provided))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_csrf_token_must_match_exactly() {
        assert!(matches("token", "token"));
        assert!(!matches("token", "tokeN"));
        assert!(!matches("token", "toke"));
        assert!(!matches("token", "tokens"));
    }

    #[test]
    fn a_session_with_no_token_accepts_nothing() {
        // Including another empty string. A session that somehow carries no
        // token must refuse every write, not accept the one that sends none.
        assert!(!matches("", ""));
        assert!(!matches("", "anything"));
        assert!(!matches("token", ""));
    }
}
