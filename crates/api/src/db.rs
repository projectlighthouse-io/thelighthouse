//! The connection pool, and the one decision that comes with it.
//!
//! There are no structs here, and none anywhere else yet, on purpose. The
//! schema this connects to is the laravel app's — forty-one tables the rebuild
//! does not use yet — and a `struct` per table would be forty-one things to
//! keep in step with a schema nobody is reading from. A table gets a type when
//! a query needs one.
//!
//! # Migrations and the database that already exists
//!
//! Migration 0 is the laravel schema. Production already has every table in it,
//! so running it there fails on the first `CREATE TABLE` — which is why nothing
//! here runs migrations at boot. That would turn a routine deploy into a schema
//! change, and a failed one into a container that will not start.
//!
//! Migrations are applied deliberately, by `cargo sqlx migrate run` against a
//! database that wants them. An existing database is *baselined* instead: the
//! version is recorded as applied without being run, so the next forward
//! migration is the first thing that actually executes there.

use std::time::Duration;

use sqlx::{
    Executor as _,
    postgres::{PgPool, PgPoolOptions},
};

/// Opens the pool and proves the credentials work.
///
/// Connecting eagerly rather than lazily: a bad `DATABASE_URL` should fail the
/// boot with the reason attached, not surface as a 500 on whichever request
/// happens to be first. This is the same argument `Config::load` makes about
/// the environment.
///
/// # Errors
///
/// Whatever the server said — unreachable host, wrong password, no such
/// database. The message is worth printing verbatim; it is nearly always
/// self-explanatory and guessing at it here would lose the detail.
pub(crate) async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        // The api is one process on one box fronted by one nuxt. Postgres pays
        // for idle connections in backend processes, so a small pool that is
        // actually used beats a large one that mostly is not.
        .max_connections(10)
        // Fail a request rather than pile up behind an exhausted pool. A
        // request that waits five seconds for a connection has already lost.
        .acquire_timeout(Duration::from_secs(5))
        // Postgres will not notice a client that vanished, so a pool held open
        // across a network blip can hand out sockets that are already dead.
        .test_before_acquire(true)
        .connect(url)
        .await
}

/// Whether the database is answering *now*.
///
/// `SELECT 1` rather than checking that the pool object exists: a pool with no
/// working connection behind it is exactly the failure this is meant to catch,
/// and it reports healthy under any weaker test.
pub(crate) async fn is_reachable(pool: &PgPool) -> bool {
    match pool.execute("SELECT 1").await {
        Ok(_) => true,
        Err(error) => {
            tracing::error!(error = %error, "database health check failed");
            false
        }
    }
}
