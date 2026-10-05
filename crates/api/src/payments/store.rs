//! Every membership query, and the only place that talks to postgres.
//!
//! Nothing here knows about HTTP. These return rows or `sqlx::Error`, and
//! `super::handler` decides what a caller is told.

use billing::{Status, Subscription};
use sqlx::postgres::PgPool;

use super::membership::{ACTIVE, ENDED, GRACE, Membership};

/// Aliased to [`Membership`]'s field names, which `FromRow` maps by.
const COLUMNS: &str = "
    plan,
    status,
    provider,
    period_ends_at,
    cancel_at,
    lifetime
";

/// The membership a reader currently has, live or in grace.
///
/// At most one, which the partial unique index in
/// `20260902000000_add_memberships.sql` enforces rather than trusting this
/// `LIMIT`. Ended rows are excluded: they are history, and a reader who
/// cancelled and resubscribed has both.
pub(crate) async fn live(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<Membership>, sqlx::Error> {
    sqlx::query_as::<_, Membership>(&format!(
        "SELECT {COLUMNS} FROM memberships \
         WHERE user_id = $1 AND status <> $2 \
         LIMIT 1"
    ))
    .bind(user_id)
    .bind(ENDED)
    .fetch_optional(db)
    .await
}

/// The provider's customer id for this reader, if one has been minted.
///
/// `users.stripe_id` is inherited from the laravel schema and is Stripe's, in
/// name and in fact. A second provider needs its own column rather than a
/// reuse of this one: two providers' customer ids in one column is a value
/// whose meaning depends on a row somewhere else.
pub(crate) async fn customer(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    let found: Option<(Option<String>,)> =
        sqlx::query_as("SELECT stripe_id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(db)
            .await?;

    Ok(found.and_then(|(id,)| id))
}

/// The reader's email, which the provider needs to send a receipt to.
pub(crate) async fn email(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    let found: Option<(String,)> =
        sqlx::query_as("SELECT email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(db)
            .await?;

    Ok(found.map(|(email,)| email))
}

/// Remember the provider's customer id against the reader.
///
/// Written when a checkout completes. Without it the next checkout mints a
/// second customer and the reader's payment history is split between them.
pub(crate) async fn remember_customer(
    db: &PgPool,
    user_id: i64,
    customer: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET stripe_id = $2 WHERE id = $1")
        .bind(user_id)
        .bind(customer)
        .execute(db)
        .await?;

    Ok(())
}

/// The reader a subscription already on file belongs to.
pub(crate) async fn reader_of(
    db: &PgPool,
    provider: &str,
    provider_ref: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let found: Option<(i64,)> = sqlx::query_as(
        "SELECT user_id FROM memberships \
         WHERE provider = $1 AND provider_ref = $2",
    )
    .bind(provider)
    .bind(provider_ref)
    .fetch_optional(db)
    .await?;

    Ok(found.map(|(id,)| id))
}

/// The reader a Stripe customer id belongs to.
///
/// `users.stripe_id` is Stripe's and nobody else's (see [`customer`]), so this
/// is only ever asked on behalf of the stripe provider.
///
/// Two readers holding the same customer id is a state the column does not
/// prevent, and guessing between them would hand one reader's subscription to
/// the other. Either one reader, or nobody.
pub(crate) async fn reader_by_customer(
    db: &PgPool,
    customer: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let found: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE stripe_id = $1 LIMIT 2")
            .bind(customer)
            .fetch_all(db)
            .await?;

    match found.as_slice() {
        [(id,)] => Ok(Some(*id)),
        [] => Ok(None),
        _ => {
            tracing::warn!(
                customer,
                "more than one reader holds this stripe customer; naming none"
            );
            Ok(None)
        }
    }
}

/// Grant a reader every book in a bundle, forever.
///
/// One statement rather than one per book: a purchase that granted four books
/// and failed on the fifth would leave a reader holding a bundle they paid for
/// in full. `unnest` makes the whole grant one row-set, so it lands or it does
/// not.
///
/// `ON CONFLICT DO NOTHING` because owning a book twice is not a state to
/// represent — a reader who buys the Rust track after already owning one of
/// its books keeps the row they had, which may well be an outright purchase
/// with its own history.
pub(crate) async fn grant(
    db: &PgPool,
    user_id: i64,
    books: &[uuid::Uuid],
) -> Result<u64, sqlx::Error> {
    if books.is_empty() {
        return Ok(0);
    }

    let done = sqlx::query(
        "INSERT INTO entitlements (user_id, book_id, source, granted_at) \
         SELECT $1, book_id, $2, now() FROM unnest($3::uuid[]) AS book_id \
         ON CONFLICT (user_id, book_id) DO NOTHING",
    )
    .bind(user_id)
    // `1` is the bundle source from the entitlements migration: got it as part
    // of something larger, rather than bought on its own or granted.
    .bind(1_i16)
    .bind(books)
    .execute(db)
    .await?;

    Ok(done.rows_affected())
}

/// Every book this reader holds outright, by id.
///
/// The purchases, not the subscription: a track bought once expands into one
/// row per book, and those rows outlive any membership.
pub(crate) async fn owned(
    db: &PgPool,
    user_id: i64,
) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
    let rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT book_id FROM entitlements \
         WHERE user_id = $1 AND (expires_at IS NULL OR expires_at > now())",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// Write what the provider says a subscription now is.
///
/// **An upsert on `(provider, provider_ref)`, which is what makes this safe to
/// call twice.** Providers retry a delivery they did not get a 2xx for, so the
/// same event arriving again is ordinary traffic rather than an anomaly, and
/// the second write is the first statement over again.
///
/// Ordering between deliveries is not guaranteed either. Nothing here reads
/// the row first and decides: every field is taken from the event, so two
/// deliveries landing out of order settle on whichever was applied last rather
/// than on a comparison this code got wrong.
pub(crate) async fn record(
    db: &PgPool,
    user_id: i64,
    provider: &str,
    subscription: &Subscription,
) -> Result<(), sqlx::Error> {
    let plan = super::track::plan_of(subscription);

    sqlx::query(
        "INSERT INTO memberships \
            (user_id, plan, status, provider, provider_ref, \
             started_at, period_ends_at, cancel_at, ended_at) \
         VALUES ($1, $2, $3, $4, $5, now(), $6, $7, $8) \
         ON CONFLICT (provider, provider_ref) \
         WHERE provider_ref IS NOT NULL \
         DO UPDATE SET \
            plan = EXCLUDED.plan, \
            status = EXCLUDED.status, \
            period_ends_at = EXCLUDED.period_ends_at, \
            cancel_at = EXCLUDED.cancel_at, \
            ended_at = EXCLUDED.ended_at",
    )
    .bind(user_id)
    .bind(plan)
    .bind(status_of(subscription.status))
    .bind(provider)
    .bind(&subscription.reference)
    .bind(subscription.period_ends_at)
    .bind(subscription.cancel_at)
    .bind(ended_at(subscription))
    .execute(db)
    .await?;

    Ok(())
}

/// The provider's four states as this schema stores them.
const fn status_of(status: Status) -> i16 {
    match status {
        Status::Active => ACTIVE,
        Status::PastDue => GRACE,
        // `Incomplete` is a checkout nobody finished. It grants nothing and it
        // is not coming back, which is what `ENDED` means here.
        Status::Canceled | Status::Incomplete => ENDED,
    }
}

/// When it stopped, which is only knowable once it has.
fn ended_at(subscription: &Subscription) -> Option<chrono::NaiveDateTime> {
    match subscription.status {
        Status::Canceled | Status::Incomplete => subscription.cancel_at,
        Status::Active | Status::PastDue => None,
    }
}
