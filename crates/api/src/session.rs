//! The session: a row in `sessions`, and the opaque id that points at it.
//!
//! The cookie carries 32 random bytes and nothing else. The row is the truth,
//! so there is nothing in the cookie to tamper with, nothing to leak, and — the
//! reason this replaced a signed stateless cookie — **a session can be
//! revoked**. Signing out deletes the row, and a copy lifted off a browser dies
//! with it rather than staying valid until its `exp`.
//!
//! No HMAC over the id. 32 bytes from the OS CSPRNG are not guessable, and
//! signing them would only save a primary-key lookup that has to happen anyway
//! to find out who the reader is.
//!
//! ```text
//!   id             the cookie value. varchar(255), 64 hex characters here.
//!   user_id        the reader. A row without one is not a session.
//!   payload        json: the provider, and the csrf token minted at login.
//!   last_activity  unix seconds. The expiry clock, refreshed on use.
//! ```
//!
//! `ip_address` and `user_agent` are left null. They are for a "your sessions"
//! screen that does not exist; filling them before then is storing a reader's
//! address for nobody to read.
//!
//! **Expired rows are only deleted when someone tries to use them.** A session
//! abandoned rather than signed out sits in the table forever. That is a
//! sweeper's job, and there is nothing to sweep with yet.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

/// 30 days *without a request*, not 30 days from sign-in. Sliding, so an active
/// reader is never signed out mid-read, while an abandoned session still dies.
pub(crate) const MAX_AGE: i64 = 60 * 60 * 24 * 30;

/// How stale `last_activity` may get before a read refreshes it.
///
/// Not every request: `last_activity` is indexed, so touching it writes a new
/// row version and an index entry. Hourly keeps the sliding window honest to
/// within an hour of a thirty-day budget, which is close enough to free.
const TOUCH_AFTER: i32 = 60 * 60;

/// A resolved session. Put in request extensions by `require_session`, so a
/// handler behind that layer can take it as an extractor and be sure of it.
#[derive(Clone, Debug)]
pub(crate) struct Session {
    pub(crate) id: String,
    pub(crate) user_id: i64,
    /// Which provider signed this session in. Not authorization — it is what
    /// the reader is shown about their own account.
    pub(crate) provider: String,
    /// The token every state-changing request has to echo back in a header.
    ///
    /// Minted per session and never leaves the row except through
    /// `/api/auth/session`, which already needs the session cookie to answer.
    /// Compared in `api::require_session`.
    pub(crate) csrf: String,
}

/// The `payload` column. A json object rather than two columns because laravel
/// owns this table's shape and adding columns to it during the crossover is a
/// migration both stacks have to agree about.
#[derive(Debug, Serialize, Deserialize)]
struct SessionPayload {
    provider: String,
    csrf: String,
}

/// Opens a session for a reader.
///
/// `None` for anything that went wrong — no entropy, no database, unserialisable
/// payload. The caller has one response to all of them: the sign-in did not
/// complete. The reason is logged here rather than returned, because nothing
/// upstream can act on the difference.
pub(crate) async fn create(
    pool: &PgPool,
    user_id: i64,
    provider: &str,
) -> Option<Session> {
    let now = now()?;

    // The same generator as the oauth state: 32 bytes of OS randomness, hex
    // encoded. No fallback if it fails — a guessable session id is worse than a
    // failed login.
    let id = loginwith::random_state().ok()?;
    let csrf = loginwith::random_state().ok()?;

    let payload = SessionPayload {
        provider: provider.to_owned(),
        csrf: csrf.clone(),
    };
    let payload = serde_json::to_string(&payload).ok()?;

    sqlx::query(
        "INSERT INTO sessions (id, user_id, payload, last_activity) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&payload)
    .bind(now)
    .execute(pool)
    .await
    .inspect_err(|error| tracing::error!(%error, user_id, "failed to open a session"))
    .ok()?;

    Some(Session {
        id,
        user_id,
        provider: provider.to_owned(),
        csrf,
    })
}

/// The only place a cookie is allowed to become a reader.
///
/// Every failure is the same `None` — unknown id, expired row, a row with no
/// user, unreadable payload, database down. A caller cannot accidentally tell
/// them apart and act on the difference.
pub(crate) async fn load(pool: &PgPool, id: &str) -> Option<Session> {
    let now = now()?;

    let (user_id, payload, last_activity) = sqlx::query_as::<
        _,
        (Option<i64>, String, i32),
    >(
        "SELECT user_id, payload, last_activity FROM sessions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .inspect_err(|error| tracing::error!(%error, "failed to read the session"))
    .ok()??;

    // `user_id` is nullable because laravel used this table for anonymous
    // sessions too. A row without a reader is not one of ours.
    let user_id = user_id?;

    if expired(now, last_activity) {
        // Gone the moment it is refused, so a stale id cannot be probed twice
        // and the row does not sit there for a sweeper that does not exist.
        delete(pool, id).await;
        return None;
    }

    if now.saturating_sub(last_activity) > TOUCH_AFTER {
        // Best effort. A failed touch shortens this session's window; it must
        // not turn a valid session into a signed-out reader.
        let _ = sqlx::query("UPDATE sessions SET last_activity = $2 WHERE id = $1")
            .bind(id)
            .bind(now)
            .execute(pool)
            .await
            .inspect_err(|error| tracing::warn!(%error, user_id, "failed to refresh the session"));
    }

    let payload: SessionPayload = serde_json::from_str(&payload).ok()?;

    Some(Session {
        id: id.to_owned(),
        user_id,
        provider: payload.provider,
        csrf: payload.csrf,
    })
}

/// Real revocation: the row goes, and every copy of that cookie is dead.
pub(crate) async fn delete(pool: &PgPool, id: &str) {
    let _ = sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .inspect_err(
            |error| tracing::error!(%error, "failed to delete the session"),
        );
}

/// Whether a session has gone too long without being used.
///
/// `saturating_sub` because `last_activity` is whatever is in the column: a
/// clock that went backwards, or a row written by something else, must not
/// overflow into "not expired".
fn expired(now: i32, last_activity: i32) -> bool {
    i64::from(now.saturating_sub(last_activity)) > MAX_AGE
}

/// Unix seconds, as the `integer` column stores them.
///
/// `None` rather than a fallback, and every caller treats it as a hard failure:
/// a clock this code cannot read is a clock it cannot expire sessions against,
/// and guessing either signs everybody out or signs nobody out. It also stops
/// being representable in January 2038, which the column type shares — the fix
/// is a wider column, in both stacks, not a cast here.
fn now() -> Option<i32> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    i32::try_from(seconds).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i32 = 60 * 60 * 24;

    #[test]
    fn a_session_used_today_is_not_expired() {
        let now = 1_800_000_000;

        assert!(!expired(now, now));
        assert!(!expired(now, now - DAY));
        assert!(!expired(now, now - 29 * DAY));
    }

    #[test]
    fn a_session_untouched_for_a_month_is_expired() {
        let now = 1_800_000_000;

        assert!(expired(now, now - 31 * DAY));
    }

    #[test]
    fn a_clock_that_went_backwards_does_not_expire_a_live_session() {
        // Future timestamps read as "used very recently", not as an overflow
        // that wraps round into expired.
        assert!(!expired(0, i32::MAX));
        assert!(!expired(1_800_000_000, 1_900_000_000));
    }

    #[test]
    fn the_payload_carries_the_provider_and_the_csrf_token() {
        let json = serde_json::to_string(&SessionPayload {
            provider: "github".to_owned(),
            csrf: "token".to_owned(),
        })
        .unwrap();

        let read: SessionPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(read.provider, "github");
        assert_eq!(read.csrf, "token");
    }

    #[test]
    fn the_clock_is_readable_and_in_range() {
        let now = now().unwrap();

        // Some time after this was written, and before the column overflows.
        assert!(now > 1_700_000_000);
    }
}
