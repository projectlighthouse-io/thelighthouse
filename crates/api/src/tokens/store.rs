//! The token lookup, and the only thing in this module that talks to postgres.
//!
//! Nothing here knows about HTTP. It answers "which reader, if any" and the
//! middleware decides what a caller is told.

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
