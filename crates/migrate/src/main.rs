//! `artisan migrate`, for this stack.
//!
//! A separate binary because applying a schema change is a decision, not a side
//! effect of starting a server. The api never runs migrations: if it did, a
//! deploy would double as a schema change, a failed migration would be a
//! container that will not start, and two containers coming up together would
//! race each other through the same files.
//!
//! ```text
//!   lighthouse-migrate run       apply everything pending
//!   lighthouse-migrate status    what is applied, what is pending
//! ```
//!
//! The migrations are compiled into this binary by `sqlx::migrate!`, so the
//! deployed artifact carries them and there is nothing to copy onto the box and
//! no `sqlx-cli` to install. The macro reads the directory at build time and
//! needs no database to do it, which is what keeps the docker build honest.
//!
//! # The database that already exists
//!
//! Migration 0 is the laravel schema. Any database laravel has already touched
//! has those tables, so `run` against one fails on the first `CREATE TABLE` —
//! correctly. Such a database needs baselining instead: its migration 0 is
//! recorded as applied without being executed, so the first forward migration
//! is the first thing that runs there. That is a cutover step, deliberately not
//! automated here — a command that can mark migrations "already done" is one
//! fat-finger away from skipping a real one.

use std::process::ExitCode;

use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

/// Compiled in at build time from the workspace's `migrations/`.
static MIGRATIONS: Migrator = sqlx::migrate!("../../migrations");

const USAGE: &str = "usage: lighthouse-migrate <run|status>";

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let command = std::env::args().nth(1).unwrap_or_default();

    // Checked before touching the database: `lighthouse-migrate` with a typo in
    // the subcommand should say so, not open a connection first.
    if !matches!(command.as_str(), "run" | "status") {
        return Err(USAGE.to_owned());
    }

    // Same rules as the api: a missing .env is fine, an unreadable one is not.
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        return Err(format!("cannot read .env: {error}"));
    }

    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not set in .env or the environment".to_owned())?;

    // One connection. This is a short-lived command run by a person, and a pool
    // would only mean more sockets doing the same single job.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|error| format!("cannot connect: {error}"))?;

    match command.as_str() {
        "run" => apply(&pool).await,
        _ => status(&pool).await,
    }
}

async fn apply(pool: &sqlx::PgPool) -> Result<(), String> {
    let before = applied_versions(pool).await;

    MIGRATIONS
        .run(pool)
        .await
        .map_err(|error| format!("migration failed: {error}"))?;

    let after = applied_versions(pool).await;

    // Names what ran rather than printing "done". A migration command that is
    // silent about which files it applied is one you have to go and check.
    let fresh: Vec<_> = MIGRATIONS
        .iter()
        .filter(|m| !before.contains(&m.version) && after.contains(&m.version))
        .collect();

    if fresh.is_empty() {
        println!("nothing to migrate");
    } else {
        for migration in fresh {
            println!("applied  {} {}", migration.version, migration.description);
        }
    }

    Ok(())
}

async fn status(pool: &sqlx::PgPool) -> Result<(), String> {
    let applied = applied_versions(pool).await;

    for migration in MIGRATIONS.iter() {
        let mark = if applied.contains(&migration.version) {
            "applied"
        } else {
            "pending"
        };
        println!("{mark}  {} {}", migration.version, migration.description);
    }

    // An applied version with no file is a deployment running migrations this
    // build has never seen — a rollback past a migration, usually. Worth saying
    // out loud rather than leaving it to be inferred from a short list.
    let known: Vec<_> = MIGRATIONS.iter().map(|m| m.version).collect();
    for version in applied.iter().filter(|v| !known.contains(v)) {
        println!("unknown  {version} (applied, but not in this binary)");
    }

    Ok(())
}

/// Applied versions, or none when the bookkeeping table does not exist yet —
/// which is simply a database that has never been migrated.
async fn applied_versions(pool: &sqlx::PgPool) -> Vec<i64> {
    sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations WHERE success")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}
