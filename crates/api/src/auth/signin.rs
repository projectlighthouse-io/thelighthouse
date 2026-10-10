//! The two routes a browser walks to sign in.
//!
//! `loginwith` owns the OAuth mechanics — the authorize URL, the state check,
//! the token exchange, the profile mapping. What is here is the half it refuses
//! to guess at: where the state lives between the redirect and the callback,
//! and where a browser is sent afterwards.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use loginwith::{Callback, Error as LoginError};
use serde::Deserialize;

use super::{
    OAUTH_COOKIE, OAUTH_PATH, READER_COOKIE, SESSION_COOKIE,
    redirect::{DEFAULT_REDIRECT, redirect, safe_redirect, to_login},
};
use crate::{
    api::AppState,
    cookie::{self, Cookie},
    payments, session, users,
};

/// Long enough for a reader to fill in a provider's login form and a 2FA
/// prompt, short enough that an abandoned attempt does not linger.
pub(crate) const OAUTH_MAX_AGE: i64 = 60 * 10;

/// Deliberately vague, and the same for every failure that is not a cancelled
/// sign-in. What actually happened is in the logs; telling the browser whether
/// a code was rejected or a provider timed out helps nobody but a prober.
pub(crate) const GENERIC_FAILURE: &str =
    "Sign-in did not complete. Please try again.";

pub(crate) const EXPIRED_FAILURE: &str =
    "That sign-in link expired. Please try again.";

pub(crate) const CANCELLED: &str = "Sign-in was cancelled.";

/// The one failure specific enough to be worth naming, because the reader can
/// act on it: an account is found by email, so there has to be one. Telling
/// somebody to "try again" when trying again cannot possibly work is worse than
/// telling them what is wrong.
pub(crate) const NO_EMAIL: &str = "That account shared no email address. Make one public with the provider, or use the other one.";

#[derive(Debug, Deserialize)]
pub(crate) struct StartQuery {
    /// Where the reader was headed before being asked to sign in. Attacker
    /// controlled — see [`safe_redirect`].
    redirect: Option<String>,
}

/// Mint a state, remember it, and hand the browser to the provider.
pub(crate) async fn start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<StartQuery>,
) -> Response {
    // 404, not 400: an unregistered provider is a URL that does not exist here,
    // and saying otherwise confirms which providers are configured.
    let Some(driver) = state.socials.driver(&provider) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(oauth_state) = loginwith::random_state() else {
        // No fallback. A predictable state is worse than a failed login.
        tracing::error!(%provider, "failed to read the system random source");
        return to_login(GENERIC_FAILURE, &[]);
    };

    let target = safe_redirect(query.redirect.as_deref());
    let secure = state.config.cookie_secure();

    redirect(
        &driver.authorize_url(&oauth_state),
        &[Cookie::new(OAUTH_COOKIE, &pack(&oauth_state, &target))
            .path(OAUTH_PATH)
            .max_age(OAUTH_MAX_AGE)
            .to_header(secure)],
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    /// Providers send this instead of `code` when the reader declines, or when
    /// the app's registration is wrong.
    error: Option<String>,
}

/// The other half: verify the state, trade the code, sign a session.
pub(crate) async fn callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(driver) = state.socials.driver(&provider) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let secure = state.config.cookie_secure();
    // The attempt is over either way, so the state cookie goes on every path
    // out of here — including the failures, so a stale state cannot be replayed
    // against a second attempt.
    let discard = [expired_oauth(secure)];

    if let Some(error) = query.error.as_deref() {
        tracing::info!(%provider, %error, "the provider refused the authorization request");
        return to_login(CANCELLED, &discard);
    }

    // A missing cookie is an expired attempt, a session that was never started
    // here, or a browser that dropped it. All three are the same retry.
    let Some(packed) = cookie::read(&headers, OAUTH_COOKIE) else {
        return to_login(EXPIRED_FAILURE, &discard);
    };
    let (expected_state, target) = unpack(packed);

    let Some(code) = query.code.as_deref() else {
        return to_login(GENERIC_FAILURE, &discard);
    };

    let social = match driver
        .user(Callback {
            code,
            // Absent rather than empty is still a state that does not match, and
            // `matches` refuses an empty expectation outright.
            state: query.state.as_deref().unwrap_or_default(),
            expected_state: &expected_state,
        })
        .await
    {
        Ok(user) => user,
        Err(LoginError::InvalidState) => {
            // The one failure that is routinely innocent: a bookmarked callback,
            // a reader who took too long, a second tab. Also what CSRF looks
            // like, which is why it is refused either way.
            tracing::info!(%provider, "the callback state did not match");
            return to_login(EXPIRED_FAILURE, &discard);
        }
        Err(error) => {
            tracing::error!(%provider, error = ?error, "social sign-in failed");
            return to_login(GENERIC_FAILURE, &discard);
        }
    };

    // The provider has proved who this is. Everything from here is ours: which
    // row that person is, and a session pointing at it.
    let reader = match users::find_or_create(
        &state.db,
        driver.provider(),
        &social,
    )
    .await
    {
        Ok(reader) => reader,
        Err(users::Error::NoEmail) => {
            // Not an error to hide behind the generic message: the reader has
            // to change something with the provider before this can ever work.
            tracing::info!(%provider, "the provider shared no email address");
            return to_login(NO_EMAIL, &discard);
        }
        Err(users::Error::Database(error)) => {
            tracing::error!(%provider, %error, "failed to resolve the reader");
            return to_login(GENERIC_FAILURE, &discard);
        }
        Err(users::Error::UsernameRace) => {
            tracing::error!(%provider, "gave up choosing a username");
            return to_login(GENERIC_FAILURE, &discard);
        }
    };

    let Some(opened) =
        session::create(&state.db, reader.id, driver.provider().as_str()).await
    else {
        return to_login(GENERIC_FAILURE, &discard);
    };

    tracing::info!(%provider, user_id = reader.id, "signed in");

    // Whether this reader is new or returning. Never awaited: a sign-in must
    // not wait on Stripe, or fail because of it. Checkout ensures it again.
    payments::customer::ensure_in_background(&state, reader.id);

    redirect(
        &target,
        &[
            // The cookie carries the row's id and nothing else.
            Cookie::new(SESSION_COOKIE, &opened.id)
                .max_age(session::MAX_AGE)
                .to_header(secure),
            Cookie::new(READER_COOKIE, "1")
                .max_age(session::MAX_AGE)
                .script_readable()
                .to_header(secure),
            expired_oauth(secure),
        ],
    )
}

/// The oauth cookie, expired the same way it was set. Every path out of the
/// callback carries one, so a stale state cannot be replayed against a second
/// attempt.
pub(crate) fn expired_oauth(secure: bool) -> String {
    Cookie::expiring(OAUTH_COOKIE)
        .path(OAUTH_PATH)
        .to_header(secure)
}

/// The state and the landing path, in one cookie.
///
/// One cookie rather than two because they have one lifetime: both are written
/// at redirect and both are dead at callback. The path is base64 so its own
/// `/`, `=` and `&` cannot end the cookie value early.
pub(crate) fn pack(state: &str, redirect_to: &str) -> String {
    format!("{state}.{}", URL_SAFE_NO_PAD.encode(redirect_to))
}

/// Splits [`pack`], and treats anything unreadable as "no state, default
/// destination" — which fails the state check below rather than here.
pub(crate) fn unpack(packed: &str) -> (String, String) {
    let Some((state, encoded)) = packed.split_once('.') else {
        return (String::new(), DEFAULT_REDIRECT.to_owned());
    };

    let target = URL_SAFE_NO_PAD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());

    // Validated again on the way out, not only on the way in. The cookie is
    // HttpOnly, but a value that only gets checked once is a value that stops
    // being checked the day something else starts writing it.
    (state.to_owned(), safe_redirect(target.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_and_the_destination_survive_one_cookie() {
        let packed = pack("the-state", "/books/redis?page=2");

        assert_eq!(
            unpack(&packed),
            ("the-state".to_owned(), "/books/redis?page=2".to_owned())
        );
    }

    #[test]
    fn a_tampered_cookie_yields_a_state_that_cannot_match() {
        // Empty expected state is refused outright by loginwith::state::matches,
        // so a torn cookie is a failed login rather than a waved-through one.
        assert_eq!(unpack("").0, "");
        assert_eq!(unpack("no-separator").0, "");
        assert_eq!(unpack("").1, DEFAULT_REDIRECT);
    }

    #[test]
    fn a_destination_smuggled_into_the_cookie_is_still_checked() {
        let packed = pack("the-state", "https://evil.test");

        assert_eq!(unpack(&packed).1, DEFAULT_REDIRECT);
    }
}
