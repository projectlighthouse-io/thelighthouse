//! The token lookup, and the only thing in this module that talks to postgres.
//!
//! Nothing here knows about HTTP. It answers "which reader, if any" and the
//! middleware decides what a caller is told.

use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;

use super::{Holder, Presented};

/// The `tokenable_type` laravel wrote, and the only one this accepts.
///
/// Bound rather than assumed. Sanctum's table is polymorphic — a token can
/// point at any model — and the primary key is unique across all of them, so a
/// row for some other tokenable would otherwise have its `tokenable_id` read
/// as a user id. There is no second tokenable today; there does not need to be
/// for this to be the wrong thing to leave out.
const USER: &str = r"App\Models\User";

/// How stale `last_used_at` may get before a read refreshes it.
///
/// The same hour `session::TOUCH_AFTER` uses, for the same reason: the column
/// exists so a reader can see which of their tokens is live on the settings
/// page, and writing a new row version on every request to keep that accurate
/// to the second buys nobody anything.
///
/// A postgres interval literal, because the comparison happens in the
/// statement — see [`holder`].
const TOUCH_AFTER: &str = "1 hour";

/// The reader a presented token belongs to.
///
/// `Ok(None)` for every way a token can fail to be one: no such row, a row
/// belonging to something that is not a user, an expired one, or the right id
/// with the wrong secret. The caller has one answer to all four, and a caller
/// that could tell them apart could use the difference to enumerate ids.
///
/// **The secret is compared here rather than in SQL.** `WHERE token = $2`
/// would hand the comparison to postgres, where it is not constant time and
/// where the value ends up in `pg_stat_statements` and in any slow-query log
/// that catches the statement.
///
/// **Whether the row is stale is decided in SQL.** `last_used_at` is written
/// by `now()` on the server, so it is the server's clock that should be
/// deciding how old it is — comparing against this process's clock would make
/// the answer depend on how far the two have drifted. It also keeps `chrono`'s
/// `clock` feature off, which this crate goes out of its way not to need.
///
/// # Errors
///
/// The query. Deliberately not folded into `Ok(None)`: a database that cannot
/// be reached means *we do not know*, and answering "not signed in" would log
/// out every luxctl user for as long as the outage lasted.
pub(crate) async fn holder(
    db: &PgPool,
    presented: &Presented,
) -> Result<Option<Holder>, sqlx::Error> {
    let found = sqlx::query_as::<_, (i64, String, bool)>(&format!(
        "SELECT tokenable_id,
                token,
                last_used_at IS NULL
                  OR last_used_at
                     < (now() AT TIME ZONE 'utc') - interval '{TOUCH_AFTER}'
                  AS stale
           FROM personal_access_tokens
          WHERE id = $1
            AND tokenable_type = $2
            AND (expires_at IS NULL
                 OR expires_at > (now() AT TIME ZONE 'utc'))"
    ))
    .bind(presented.id)
    .bind(USER)
    .fetch_optional(db)
    .await?;

    let Some((user_id, stored, stale)) = found else {
        return Ok(None);
    };

    if !presented.matches(&stored) {
        return Ok(None);
    }

    if stale {
        touch(db, presented.id).await;
    }

    Ok(Some(Holder { user_id }))
}

/// Records that a token was used. Best effort, always.
///
/// A failed touch is a settings page showing a slightly old date. It must
/// never turn a valid token into a refused request, so the result is logged
/// and dropped rather than returned.
async fn touch(db: &PgPool, id: i64) {
    let _ = sqlx::query(
        "UPDATE personal_access_tokens
            SET last_used_at = (now() AT TIME ZONE 'utc')
          WHERE id = $1",
    )
    .bind(id)
    .execute(db)
    .await
    .inspect_err(|error| {
        tracing::warn!(%error, id, "failed to record a token's use");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tokenable_type_is_the_string_laravel_wrote() {
        // A raw string, so each backslash is literal and there is exactly one
        // of each. Getting this wrong is every token in the table failing.
        assert_eq!(USER, "App\\Models\\User");
        assert_eq!(USER.matches('\\').count(), 2);
    }
}

/// One of a reader's tokens, as the settings page lists them.
///
/// **No `token` column.** The hash is not a secret worth leaking and is not
/// useful to anybody, and a struct that carries it is one somebody eventually
/// serialises.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Record {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) last_used_at: Option<NaiveDateTime>,
    pub(crate) created_at: Option<NaiveDateTime>,
}

/// Every token this reader holds, newest first.
///
/// # Errors
///
/// The query.
pub(crate) async fn list(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<Record>, sqlx::Error> {
    sqlx::query_as::<_, Record>(
        "SELECT id, name, last_used_at, created_at
           FROM personal_access_tokens
          WHERE tokenable_id = $1
            AND tokenable_type = $2
          ORDER BY id DESC",
    )
    .bind(user_id)
    .bind(USER)
    .fetch_all(db)
    .await
}

/// Writes a token for this reader and answers the row.
///
/// **The hash arrives already computed.** Minting is `token::mint`, which makes
/// the secret and the hash together; this only writes what it is given, so
/// there is no path where a row is stored with a hash of something else.
///
/// `abilities` is `["*"]`, which is what every token this platform has ever
/// issued holds — see the module note about not reading it yet.
///
/// # Errors
///
/// The query. A duplicate hash would be one too, and is not worth a branch:
/// two 32-byte secrets colliding is not a case that happens.
pub(crate) async fn create(
    db: &PgPool,
    user_id: i64,
    name: &str,
    hash: &str,
) -> Result<Record, sqlx::Error> {
    sqlx::query_as::<_, Record>(
        "INSERT INTO personal_access_tokens
                (tokenable_type, tokenable_id, name, token, abilities,
                 created_at, updated_at)
         VALUES ($1, $2, $3, $4, '[\"*\"]',
                 (now() AT TIME ZONE 'utc'), (now() AT TIME ZONE 'utc'))
         RETURNING id, name, last_used_at, created_at",
    )
    .bind(USER)
    .bind(user_id)
    .bind(name)
    .bind(hash)
    .fetch_one(db)
    .await
}

/// Deletes one of this reader's tokens. `false` when there was none to delete.
///
/// **Scoped by owner in the statement, not checked afterwards.** A token
/// belonging to somebody else and a token that does not exist are the same
/// `false`, so the caller has one answer for both and cannot be used to find
/// out which ids are real.
///
/// # Errors
///
/// The query.
pub(crate) async fn revoke(
    db: &PgPool,
    user_id: i64,
    id: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "DELETE FROM personal_access_tokens
          WHERE id = $1
            AND tokenable_id = $2
            AND tokenable_type = $3",
    )
    .bind(id)
    .bind(user_id)
    .bind(USER)
    .execute(db)
    .await?;

    Ok(done.rows_affected() > 0)
}
