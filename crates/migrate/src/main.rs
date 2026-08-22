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
//!   lighthouse-migrate baseline  adopt a database laravel already built
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
//! correctly, because that database does not need building, it needs adopting.
//! `baseline` adopts it: migration 0 is recorded as applied without being
//! executed, and the next forward migration is the first thing that runs there.
//!
//! The obvious alternative is to make migration 0 idempotent with
//! `IF NOT EXISTS`. It does not work here and would not be right if it did.
//! Postgres has no `ADD CONSTRAINT IF NOT EXISTS`, and the dump carries 115 of
//! them, so each would need wrapping in a `DO` block that swallows
//! `duplicate_object` — turning a verbatim `pg_dump` into a hand-edited file.
//! Worse, `IF NOT EXISTS` does not compare anything: it succeeds against a
//! table that exists with entirely different columns, reporting a clean
//! migration over a schema that has drifted. Baselining states the true thing
//! instead, and leaves migration 0 meaning "build this from nothing" — which is
//! what a fresh clone and CI need it to mean.
//!
//! `baseline` is deliberately narrow, because a command that marks migrations
//! "already done" is otherwise one fat-finger from skipping a real one:
//!
//! - it only ever records the **first** migration, never a later one;
//! - it refuses unless the schema is genuinely already there;
//! - it refuses if that migration is already recorded.
//!
//! Skipping a forward migration is therefore not something it can be talked
//! into doing.

use std::process::ExitCode;

use sqlx::{
    Connection as _,
    migrate::{Migrate as _, Migrator},
    postgres::PgConnection,
};

/// Compiled in at build time from the workspace's `migrations/`.
static MIGRATIONS: Migrator = sqlx::migrate!("../../migrations");

const USAGE: &str = "usage: lighthouse-migrate <run|status|baseline>";

/// The table `baseline` looks for to decide whether a schema is already here.
///
/// `users` because it is in migration 0, it is not something anything else
/// would create by accident, and it is the last table anyone would drop.
const SENTINEL: &str = "public.users";

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
    if !matches!(command.as_str(), "run" | "status" | "baseline") {
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

    // A single connection, not a pool. This is a short-lived command run by a
    // person, so a pool buys nothing — and it costs the error message: a pool
    // that cannot connect reports "pool timed out while waiting for an open
    // connection", which hides the refusal and never says where it was trying
    // to reach. A plain connection says "Connection refused".
    let mut conn = PgConnection::connect(&url)
        .await
        .map_err(|error| format!("cannot connect to {}: {error}", redacted(&url)))?;

    match command.as_str() {
        "run" => apply(&mut conn).await,
        "baseline" => baseline(&mut conn).await,
        _ => status(&mut conn).await,
    }
}

/// The connection target, without the password.
///
/// Printed on a failed connect, because "cannot connect" without saying where
/// is the difference between a two-second fix and a puzzled ten minutes — and
/// pointing at the wrong host is the most common reason this fails.
fn redacted(url: &str) -> String {
    // postgres://user:password@host:port/db — everything up to and including
    // the last '@' is credentials.
    match url.rsplit_once('@') {
        Some((_, target)) => target.to_owned(),
        None => url.to_owned(),
    }
}

/// Adopts a database that already has the schema migration 0 would create.
///
/// Records the first migration as applied without running it. Everything after
/// it is left pending, so the next `run` applies the forward migrations and
/// nothing else.
async fn baseline(conn: &mut PgConnection) -> Result<(), String> {
    let Some(first) = MIGRATIONS.iter().next() else {
        return Err("this binary carries no migrations".to_owned());
    };

    // Refuse on an empty database. Baselining one would record a schema as
    // present that nothing ever created, and the failure would surface much
    // later as a missing table rather than here.
    let schema_exists: Option<String> =
        sqlx::query_scalar(&format!("SELECT to_regclass('{SENTINEL}')::text"))
            .fetch_one(&mut *conn)
            .await
            .map_err(|error| format!("cannot inspect the schema: {error}"))?;

    if schema_exists.is_none() {
        return Err(format!(
            "{SENTINEL} does not exist, so this database has nothing to adopt — use `run`"
        ));
    }

    // sqlx creates its bookkeeping table on first run; here nothing has run.
    conn.ensure_migrations_table()
        .await
        .map_err(|error| format!("cannot create the migrations table: {error}"))?;

    if applied_versions(&mut *conn).await.contains(&first.version) {
        println!("already baselined  {} {}", first.version, first.description);
        return Ok(());
    }

    // The checksum has to be the migration's own, or every later `run` fails
    // validation against a row it cannot match.
    sqlx::query(
        "INSERT INTO _sqlx_migrations
             (version, description, installed_on, success, checksum, execution_time)
         VALUES ($1, $2, now(), true, $3, 0)",
    )
    .bind(first.version)
    .bind(&*first.description)
    .bind(&*first.checksum)
    .execute(&mut *conn)
    .await
    .map_err(|error| format!("cannot record the baseline: {error}"))?;

    println!("baselined  {} {}", first.version, first.description);
    println!("(recorded as applied, not executed — the schema was already here)");

    Ok(())
}

async fn apply(conn: &mut PgConnection) -> Result<(), String> {
    let before = applied_versions(&mut *conn).await;

    MIGRATIONS
        .run(&mut *conn)
        .await
        .map_err(|error| format!("migration failed: {error}"))?;

    let after = applied_versions(&mut *conn).await;

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

async fn status(conn: &mut PgConnection) -> Result<(), String> {
    let applied = applied_versions(&mut *conn).await;

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
async fn applied_versions(conn: &mut PgConnection) -> Vec<i64> {
    sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations WHERE success")
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default()
}
