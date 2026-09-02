//! How much of a book a reader may read.

use sqlx::postgres::PgPool;
use uuid::Uuid;

use ohara::catalog::BookEntry;

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
/// Two ways to hold a book, and either is enough:
///
/// 1. **An entitlement row** naming it. That is a purchase, and it does not
///    expire unless it was granted with an end date.
/// 2. **A live subscription**, unless this book is one the subscription does
///    not cover — `config.subscription_excludes`.
///
/// The second cannot be folded into the first by writing rows at purchase
/// time. A subscription has to cover books published after it was bought, and
/// eagerly expanding it would grant exactly the catalogue that existed on the
/// day somebody paid.
///
/// **Price is not consulted, deliberately.** What a book costs decides what
/// happens at checkout; it does not decide what is readable. Whether a lesson
/// withholds anything is stated in the content — the `<paid>` regions and
/// `access: paid` — and who may see what is withheld is this: an entitlement
/// to the book. A book priced at zero can still be one you have to be
/// subscribed to read, and that is a pricing decision that must not silently
/// unlock prose.
///
/// **A book with no id cannot be *bought*.** The id is what an entitlement
/// points at, so a book that has never been given one has no row that could
/// name it. A subscription still covers it: that check is on the slug, and
/// needs no id at all.
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
    excluded: &[String],
) -> Result<Access, sqlx::Error> {
    let Some(user_id) = reader else {
        return Ok(Access::FreeOnly);
    };

    if let Some(book_id) = book.book.id
        && holds(db, user_id, book_id).await?
    {
        return Ok(Access::Full);
    }

    if covered(db, user_id, &book.book.slug, excluded).await? {
        return Ok(Access::Full);
    }

    Ok(Access::FreeOnly)
}

/// Whether a live subscription covers this book.
///
/// The exclusion is checked first, and on the slug rather than on anything in
/// the content: whether a book is sold outside the subscription is a
/// commercial decision, and `book.yaml` is read by things that have no
/// business knowing what is for sale.
///
/// Only a membership that grants access counts. One in grace — a payment
/// failed and the provider is retrying — does not, which is the rule the
/// laravel app had and the one `Membership::grants_access` keeps.
async fn covered(
    db: &PgPool,
    user_id: i64,
    slug: &str,
    excluded: &[String],
) -> Result<bool, sqlx::Error> {
    if excluded.iter().any(|excluded| excluded == slug) {
        return Ok(false);
    }

    Ok(crate::payments::live(db, user_id)
        .await?
        .is_some_and(|membership| membership.grants_access()))
}

/// Whether this reader holds a live entitlement to this book.
///
/// A row with no `expires_at` is forever, which is what a purchase is. One with
/// a date in the past is spent, and is left in place rather than deleted — that
/// a reader used to own a book is worth being able to see.
async fn holds(
    db: &PgPool,
    user_id: i64,
    book_id: Uuid,
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
    use ohara::{catalog::Snapshot, fixture};

    fn fixture_book() -> Snapshot {
        Snapshot::load(&fixture::content(), ohara::Drafts::Hidden).unwrap()
    }

    // These reach no database: every one of them is a case `access` answers
    // before it would query, which is the point being made about each.

    #[tokio::test]
    async fn a_stranger_gets_only_the_free_half() {
        let snapshot = fixture_book();
        let book = snapshot.book("fixture-book").unwrap();
        // Lazy, and never connected: a stranger is answered before the query
        // would happen. If that ever stops being true this fails loudly here
        // rather than passing against a stub.
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        assert_eq!(
            access(&db, None, book, &[]).await.unwrap(),
            Access::FreeOnly
        );
    }

    #[tokio::test]
    async fn a_free_book_is_not_a_reason_to_hand_over_the_paid_half() {
        // The rule this replaced: a book priced at zero used to short-circuit
        // to Full for everybody. Price decides checkout, not access — a book
        // given away can still be one you must be subscribed to read.
        // `unfinished-book` carries no price block at all, so it is free.
        let snapshot =
            Snapshot::load(&fixture::drafts(), ohara::Drafts::Shown).unwrap();
        let book = snapshot.book("unfinished-book").unwrap();
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        assert!(book.book.price.is_free(), "the fixture must be free");
        assert_eq!(
            access(&db, None, book, &[]).await.unwrap(),
            Access::FreeOnly
        );
    }

    #[tokio::test]
    async fn a_book_the_subscription_excludes_is_never_covered_by_one() {
        // The revenue-critical direction: a book sold separately must not come
        // free with a subscription. Answered from the slug alone, before any
        // query, which is why this needs no database — and why an excluded
        // book cannot be opened by a membership row being wrong.
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        let excluded = vec!["horizon-book".to_owned()];

        assert!(
            !covered(&db, 1, "horizon-book", &excluded).await.unwrap(),
            "an excluded book was covered by a subscription"
        );
    }
}
