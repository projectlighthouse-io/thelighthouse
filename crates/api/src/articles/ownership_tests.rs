//! Who may change an article, against a real database.
//!
//! Ownership is not checked in a handler and then trusted: it is a `WHERE
//! user_id = $n` inside the statement that writes. The only honest test of a
//! clause in SQL is running the SQL, so these need a migrated postgres and are
//! ignored by default:
//!
//! ```text
//! DATABASE_URL=postgres://… cargo test -p lighthouse-api -- --ignored ownership
//! ```
//!
//! Each test makes its own two authors and removes them after, so it can run
//! against a development database without touching anybody's rows.

use sqlx::postgres::PgPool;

use super::{
    payload::{ValidArticle, validate_article},
    store::{self, Filters},
};
use crate::request::Paging;

async fn db() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect(
        "these tests need DATABASE_URL pointing at a migrated postgres",
    );

    PgPool::connect(&url).await.unwrap()
}

/// Two fresh authors, unique to this run.
async fn authors(db: &PgPool) -> (i64, i64) {
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let make = async |who: &str| {
        let (id,) = sqlx::query_as::<_, (i64,)>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!("ownership {who}"))
        .bind(format!("ownership-{who}-{run}@example.test"))
        .fetch_one(db)
        .await
        .unwrap();
        id
    };

    (make("a").await, make("b").await)
}

async fn forget(db: &PgPool, users: &[i64]) {
    sqlx::query("DELETE FROM articles WHERE user_id = ANY($1)")
        .bind(users)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = ANY($1)")
        .bind(users)
        .execute(db)
        .await
        .unwrap();
}

fn article() -> ValidArticle<'static> {
    validate_article(
        "Ownership",
        "Whose is this?",
        &["go".to_owned()],
        "A body.",
    )
    .unwrap()
}

fn slug(owner: i64) -> String {
    format!("ownership-{owner}")
}

const ONE_PAGE: Paging = Paging {
    page: 1,
    per_page: 10,
    offset: 0,
};

#[tokio::test]
#[ignore = "needs a migrated postgres in DATABASE_URL"]
async fn ownership_another_author_cannot_rewrite_an_article() {
    let db = db().await;
    let (a, b) = authors(&db).await;
    store::insert(&db, a, &slug(a), &article()).await.unwrap();

    let rewritten = validate_article(
        "Taken over",
        "Not yours",
        &["rust".to_owned()],
        "B's words.",
    )
    .unwrap();

    let by_b = store::rewrite(&db, &slug(a), b, &rewritten).await.unwrap();
    let still = store::find(&db, &slug(a)).await.unwrap();
    let by_a = store::rewrite(&db, &slug(a), a, &rewritten).await.unwrap();

    forget(&db, &[a, b]).await;

    assert!(by_b.is_none(), "B rewrote A's article");
    assert_eq!(still.map(|s| s.title), Some("Ownership".to_owned()));
    assert!(by_a.is_some(), "A could not rewrite their own article");
}

#[tokio::test]
#[ignore = "needs a migrated postgres in DATABASE_URL"]
async fn ownership_another_author_cannot_delete_an_article() {
    let db = db().await;
    let (a, b) = authors(&db).await;
    store::insert(&db, a, &slug(a), &article()).await.unwrap();

    let by_b = store::delete(&db, &slug(a), b).await.unwrap();
    let survived = store::find(&db, &slug(a)).await.unwrap().is_some();
    let by_a = store::delete(&db, &slug(a), a).await.unwrap();

    forget(&db, &[a, b]).await;

    assert!(!by_b, "B deleted A's article");
    assert!(survived, "A's article is gone after B's attempt");
    assert!(by_a, "A could not delete their own article");
}

#[tokio::test]
#[ignore = "needs a migrated postgres in DATABASE_URL"]
async fn ownership_another_author_cannot_archive_an_article() {
    let db = db().await;
    let (a, b) = authors(&db).await;
    store::insert(&db, a, &slug(a), &article()).await.unwrap();

    let by_b = store::archive(&db, &slug(a), b, true).await.unwrap();
    let still_public = store::find(&db, &slug(a)).await.unwrap().is_some();

    forget(&db, &[a, b]).await;

    assert!(by_b.is_none(), "B archived A's article");
    assert!(still_public, "A's article went private after B's attempt");
}

#[tokio::test]
#[ignore = "needs a migrated postgres in DATABASE_URL"]
async fn ownership_an_archived_article_is_hidden_from_everyone_but_its_author()
{
    let db = db().await;
    let (a, b) = authors(&db).await;
    store::insert(&db, a, &slug(a), &article()).await.unwrap();

    let archived = store::archive(&db, &slug(a), a, true).await.unwrap();
    let public = store::find(&db, &slug(a)).await.unwrap();
    let listed = store::page(
        &db,
        Filters {
            author: Some(a),
            ..Filters::default()
        },
        ONE_PAGE,
    )
    .await
    .unwrap();
    let counted = store::count(
        &db,
        Filters {
            author: Some(a),
            ..Filters::default()
        },
    )
    .await
    .unwrap();
    let own = store::own(&db, a, Filters::default(), ONE_PAGE)
        .await
        .unwrap();

    let restored = store::archive(&db, &slug(a), a, false).await.unwrap();
    let public_again = store::find(&db, &slug(a)).await.unwrap();

    forget(&db, &[a, b]).await;

    assert!(archived.is_some_and(|it| it.archived_at.is_some()));
    assert!(public.is_none(), "an archived article is still public");
    assert!(listed.is_empty(), "an archived article is still listed");
    assert_eq!(counted, 0, "an archived article is still counted");
    assert!(
        own.iter()
            .any(|it| it.slug == slug(a) && it.archived_at.is_some()),
        "the author lost sight of their archived article"
    );
    assert!(restored.is_some_and(|it| it.archived_at.is_none()));
    assert!(public_again.is_some(), "unarchiving did not restore it");
}
