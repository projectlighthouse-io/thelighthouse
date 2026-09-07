//! Every article query, and the only place that talks to postgres.
//!
//! A whole `query…bind` call lives here, never a SQL string handed elsewhere
//! to be filled in: `$1` and the value that fills it stay on screen together.
//!
//! Nothing here knows about HTTP. These return rows or [`StoreError`], and
//! `super::handler` decides what a caller is told.

use sqlx::postgres::PgPool;

use super::{
    article::{Article, OwnArticle},
    payload::ValidArticle,
};
use crate::request::Paging;

/// From `20260904000000_add_articles.sql`. It lives beside the statement that
/// can trip it — a handler matching on a constraint name would be a handler
/// that knows the schema.
const SLUG_UNIQUE: &str = "articles_slug_unique";

/// What can go wrong that a caller might answer differently.
#[derive(Debug)]
pub(crate) enum StoreError {
    /// Two articles minted the same slug in the same instant. Six hex
    /// characters make this vanishingly rare, and the index is what makes it
    /// impossible rather than merely unlikely.
    SlugTaken,
    /// Everything else, and nobody outside can act on it.
    Database(sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        let sqlx::Error::Database(ref database) = error else {
            return Self::Database(error);
        };

        match database.constraint() {
            Some(SLUG_UNIQUE) => Self::SlugTaken,
            _ => Self::Database(error),
        }
    }
}

/// Aliased to [`Article`]'s field names, which `FromRow` maps by.
const COLUMNS: &str = "
    a.id,
    a.slug,
    a.title,
    a.subtitle,
    ARRAY(
        SELECT t.topic FROM article_topics t
        WHERE t.article_id = a.id
        ORDER BY t.topic
    ) AS topics,
    a.body,
    u.name AS author,
    u.username AS author_username,
    a.created_at,
    a.updated_at
";

/// Aliased to [`OwnArticle`]'s field names. Carries the byline the public
/// shape does, so a listing that mixes the two — `/blog?author=` read by the
/// author — does not have to fill one in.
const OWN_COLUMNS: &str = "
    a.id,
    a.slug,
    a.title,
    a.subtitle,
    ARRAY(
        SELECT t.topic FROM article_topics t
        WHERE t.article_id = a.id
        ORDER BY t.topic
    ) AS topics,
    a.body,
    u.name AS author,
    u.username AS author_username,
    a.created_at,
    a.updated_at,
    a.taken_down_at,
    a.taken_down_reason
";

/// What a listing is narrowed by, whoever is reading it.
///
/// One struct rather than three arguments, so [`page`] and [`count`] — and
/// [`own`] and [`own_count`] — take the same set by construction. Two calls
/// that disagree give twenty rows and a total of three.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Filters<'a> {
    /// The `%term%` an `ILIKE` matches on, from `ListQuery::pattern`.
    pub(crate) pattern: Option<&'a str>,
    pub(crate) topic: Option<&'a str>,
    /// One author's, by `users.id`. `None` is everybody's.
    ///
    /// An id rather than the username off the query string: the handler has to
    /// resolve it anyway to answer "is this the reader's own shelf", and
    /// comparing ids is that answer rather than a second lookup.
    pub(crate) author: Option<i64>,
}

/// Shared so every query below narrows the same way — see [`Filters`].
///
/// **Live-ness is not in here, deliberately.** `taken_down_at IS NULL` is
/// written at each call site instead, because the whole point of [`own`] is
/// that it is the one query without it: a shared constant that sometimes
/// applied would be a filter you have to read the parameters to understand.
///
/// Null-guarded rather than clauses that appear and disappear: a query built
/// two ways has its parameter numbering maintained in two places.
const MATCHING: &str = "
    (
        $1::text IS NULL
        OR a.title ILIKE $1
        OR a.subtitle ILIKE $1
        OR a.body ILIKE $1
    )
    AND (
        $2::text IS NULL
        OR EXISTS (
            SELECT 1 FROM article_topics t
            WHERE t.article_id = a.id AND t.topic = $2
        )
    )
    AND ($3::bigint IS NULL OR a.user_id = $3)
";

/// One page of live articles, newest first.
pub(crate) async fn page(
    db: &PgPool,
    filters: Filters<'_>,
    paging: Paging,
) -> Result<Vec<Article>, StoreError> {
    let sql = format!(
        "
        SELECT {COLUMNS}
        FROM articles a
        JOIN users u ON u.id = a.user_id
        WHERE a.taken_down_at IS NULL AND {MATCHING}
        -- `id` breaks ties, so two articles posted in the same second cannot
        -- swap between pages, which is how a listing drops and repeats rows.
        ORDER BY a.created_at DESC, a.id DESC
        LIMIT $4
        OFFSET $5
        "
    );

    Ok(sqlx::query_as::<_, Article>(&sql)
        .bind(filters.pattern)
        .bind(filters.topic)
        .bind(filters.author)
        .bind(paging.per_page)
        .bind(paging.offset)
        .fetch_all(db)
        .await?)
}

/// How many the same filter matches, across every page.
///
/// No join: `articles.user_id` is a `NOT NULL` foreign key, so [`page`]'s
/// inner join cannot drop a row and counting without it gives the same answer.
pub(crate) async fn count(
    db: &PgPool,
    filters: Filters<'_>,
) -> Result<i64, StoreError> {
    let sql = format!(
        "SELECT count(a.id) FROM articles a
         WHERE a.taken_down_at IS NULL AND {MATCHING}"
    );

    let (total,) = sqlx::query_as::<_, (i64,)>(&sql)
        .bind(filters.pattern)
        .bind(filters.topic)
        .bind(filters.author)
        .fetch_one(db)
        .await?;

    Ok(total)
}

/// The `users.id` behind a username, or `None` for a username nobody has.
pub(crate) async fn user_id_of(
    db: &PgPool,
    username: &str,
) -> Result<Option<i64>, StoreError> {
    let found =
        sqlx::query_as::<_, (i64,)>("SELECT id FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(db)
            .await?;

    Ok(found.map(|(id,)| id))
}

/// One live article by its slug. `None` for no such article *and* for one that
/// was taken down — deliberately the same, so a takedown cannot be confirmed
/// from outside.
pub(crate) async fn find(
    db: &PgPool,
    slug: &str,
) -> Result<Option<Article>, StoreError> {
    let sql = format!(
        "
        SELECT {COLUMNS}
        FROM articles a
        JOIN users u ON u.id = a.user_id
        WHERE a.slug = $1
          AND a.taken_down_at IS NULL
        "
    );

    Ok(sqlx::query_as::<_, Article>(&sql)
        .bind(slug)
        .fetch_optional(db)
        .await?)
}

/// One page of an author's own articles, taken down ones included — the only
/// place a takedown reason is read.
///
/// **`user_id` is an argument, not `Filters::author`.** It is bound as the
/// author parameter whatever `filters` says, so this cannot be called in a way
/// that reads every author's taken down articles at once. Who is allowed to be
/// that id is `handler::shelf`'s decision and nothing else's.
pub(crate) async fn own(
    db: &PgPool,
    user_id: i64,
    filters: Filters<'_>,
    paging: Paging,
) -> Result<Vec<OwnArticle>, StoreError> {
    let sql = format!(
        "
        SELECT {OWN_COLUMNS}
        FROM articles a
        JOIN users u ON u.id = a.user_id
        WHERE {MATCHING}
        ORDER BY a.created_at DESC, a.id DESC
        LIMIT $4
        OFFSET $5
        "
    );

    Ok(sqlx::query_as::<_, OwnArticle>(&sql)
        .bind(filters.pattern)
        .bind(filters.topic)
        .bind(user_id)
        .bind(paging.per_page)
        .bind(paging.offset)
        .fetch_all(db)
        .await?)
}

/// How many articles this author has written, live or not, matching the same
/// filters.
pub(crate) async fn own_count(
    db: &PgPool,
    user_id: i64,
    filters: Filters<'_>,
) -> Result<i64, StoreError> {
    let sql = format!("SELECT count(a.id) FROM articles a WHERE {MATCHING}");

    let (total,) = sqlx::query_as::<_, (i64,)>(&sql)
        .bind(filters.pattern)
        .bind(filters.topic)
        .bind(user_id)
        .fetch_one(db)
        .await?;

    Ok(total)
}

/// Publishes an article and its topics, then reads it back.
///
/// **One transaction.** The row and its topics are one thing to a reader, and
/// committing the article before the topics would leave a window — and, if the
/// second statement fails, a permanent article about nothing, which every read
/// path would then show untagged.
pub(crate) async fn insert(
    db: &PgPool,
    user_id: i64,
    slug: &str,
    article: &ValidArticle<'_>,
) -> Result<OwnArticle, StoreError> {
    let mut tx = db.begin().await?;

    let (id,) = sqlx::query_as::<_, (i64,)>(
        "
        INSERT INTO articles (user_id, slug, title, subtitle, body)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        ",
    )
    .bind(user_id)
    .bind(slug)
    .bind(article.title)
    .bind(article.subtitle)
    .bind(article.body)
    .fetch_one(&mut *tx)
    .await?;

    write_topics(&mut tx, id, &article.topics).await?;

    let saved = read_own(&mut tx, id).await?;

    tx.commit().await?;

    // The row was just inserted in this transaction, so it is there.
    saved.ok_or_else(|| StoreError::Database(sqlx::Error::RowNotFound))
}

/// Rewrites one article and its topics, and reads it back.
///
/// Scoped by owner in the statement, so there is no load-check-write window.
/// `None` means no such article *or* not this author's — deliberately the
/// same.
///
/// **A taken down article can still be edited, and stays down.** Nothing here
/// touches those two columns: fixing what was objected to is the point, and
/// letting an edit clear a takedown would make the takedown advisory. Putting
/// one back is an `UPDATE` run by hand — see this module's `mod.rs`.
pub(crate) async fn rewrite(
    db: &PgPool,
    slug: &str,
    user_id: i64,
    article: &ValidArticle<'_>,
) -> Result<Option<OwnArticle>, StoreError> {
    let mut tx = db.begin().await?;

    let updated = sqlx::query_as::<_, (i64,)>(
        "
        UPDATE articles
        SET title = $3,
            subtitle = $4,
            body = $5,
            updated_at = NOW()
        WHERE slug = $1
          AND user_id = $2
        RETURNING id
        ",
    )
    .bind(slug)
    .bind(user_id)
    .bind(article.title)
    .bind(article.subtitle)
    .bind(article.body)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((id,)) = updated else {
        // Nothing matched, so nothing to roll back — but the transaction still
        // has to end, and dropping it without this rolls back on a background
        // task rather than here.
        tx.rollback().await?;
        return Ok(None);
    };

    // Replaced wholesale rather than diffed: the set is at most four rows, and
    // working out which to add and which to drop is more code than rewriting
    // them. Inside the transaction, so a reader never sees the empty moment.
    sqlx::query("DELETE FROM article_topics WHERE article_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    write_topics(&mut tx, id, &article.topics).await?;

    let saved = read_own(&mut tx, id).await?;

    tx.commit().await?;

    Ok(saved)
}

/// Writes one article's topics.
///
/// `UNNEST` rather than a statement per topic: one round trip, and the number
/// of parameters does not depend on how many topics there are.
async fn write_topics(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    article_id: i64,
    topics: &[&str],
) -> Result<(), StoreError> {
    sqlx::query(
        "
        INSERT INTO article_topics (article_id, topic)
        SELECT $1, * FROM UNNEST($2::text[])
        ",
    )
    .bind(article_id)
    .bind(topics)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// One article by id, in the author's shape, from inside a transaction.
///
/// Its own function because both writes end the same way: change the rows,
/// then read back what is actually stored rather than echoing what was sent.
async fn read_own(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: i64,
) -> Result<Option<OwnArticle>, StoreError> {
    // The join is not optional: `OWN_COLUMNS` reads the byline off `users`,
    // and a read back without it fails at decode rather than at compile time.
    let sql = format!(
        "SELECT {OWN_COLUMNS}
         FROM articles a
         JOIN users u ON u.id = a.user_id
         WHERE a.id = $1"
    );

    Ok(sqlx::query_as::<_, OwnArticle>(&sql)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?)
}

/// Deletes one article. `false` when nothing matched — see [`rewrite`].
pub(crate) async fn delete(
    db: &PgPool,
    slug: &str,
    user_id: i64,
) -> Result<bool, StoreError> {
    let deleted =
        sqlx::query("DELETE FROM articles WHERE slug = $1 AND user_id = $2")
            .bind(slug)
            .bind(user_id)
            .execute(db)
            .await?;

    Ok(deleted.rows_affected() > 0)
}
