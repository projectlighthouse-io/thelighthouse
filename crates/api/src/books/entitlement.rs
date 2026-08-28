//! How much of a book a reader may read.

use sqlx::postgres::PgPool;

use crate::ohara::catalog::BookEntry;

/// What a reader gets, not whether they pass a test.
///
/// An enum rather than a bool because the caller has to choose a body either
/// way, and `if !may_read(book)` puts that choice behind a negated maybe. A
/// `match` on this says which half is being served at the point it is served,
/// and a third answer later — expired, revoked, region-locked — stops
/// compiling everywhere it matters instead of quietly folding into whichever
/// branch `false` already took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    /// Everything, paid half included.
    Full,
    /// The free half, and the flag saying there is more.
    FreeOnly,
}

/// What this reader gets of this book.
///
/// **A free book is [`Access::Full`] for everybody**, signed in or not, without
/// touching the database. A book priced at zero is given away, and there is
/// nothing to buy that would grant more than that.
///
/// **A book with no id is [`Access::FreeOnly`] once priced.** The id is what an
/// entitlement points at, so a book that has never been given one cannot be
/// owned by anybody — there is no row that could name it.
///
/// # Errors
///
/// The entitlement query. Deliberately not swallowed into `FreeOnly`: a
/// database that cannot be reached means *we do not know*, and answering "not
/// entitled" would show a paywall to a reader who paid, which is the failure
/// that generates a support ticket rather than a page refresh.
pub(crate) async fn access(
    db: &PgPool,
    reader: Option<i64>,
    book: &BookEntry,
) -> Result<Access, sqlx::Error> {
    if book.book.price.is_free() {
        return Ok(Access::Full);
    }

    let (Some(user_id), Some(book_id)) = (reader, book.book.id) else {
        return Ok(Access::FreeOnly);
    };

    if holds(db, user_id, book_id).await? {
        Ok(Access::Full)
    } else {
        Ok(Access::FreeOnly)
    }
}

/// Whether this reader holds a live entitlement to this book.
///
/// A row with no `expires_at` is forever, which is what a purchase is. One with
/// a date in the past is spent, and is left in place rather than deleted — that
/// a reader used to own a book is worth being able to see.
async fn holds(
    db: &PgPool,
    user_id: i64,
    book_id: i64,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM entitlements \
         WHERE user_id = $1 AND book_id = $2 \
         AND (expires_at IS NULL OR expires_at > now()) \
         LIMIT 1",
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_optional(db)
    .await?;

    Ok(row.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ohara::{catalog::Snapshot, fixture};

    fn fixture_book() -> Snapshot {
        Snapshot::load(&fixture::content()).unwrap()
    }

    // These reach no database: every one of them is a case `access` answers
    // before it would query, which is the point being made about each.

    #[tokio::test]
    async fn a_priced_book_shows_only_its_free_half_to_a_stranger() {
        let snapshot = fixture_book();
        let book = snapshot.book("fixture-book").unwrap();
        // Lazy, and never connected: a stranger is answered before the query
        // would happen. If that ever stops being true this fails loudly here
        // rather than passing against a stub.
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        // The fixture book is priced at 2900.
        assert_eq!(access(&db, None, book).await.unwrap(), Access::FreeOnly);
    }
}
