//! `lighthouse-content` — ohara into postgres.
//!
//! ```text
//!   lighthouse-content sync    upsert every published book and lesson
//! ```
//!
//! A separate binary for the reason [`lighthouse-migrate`] is one: writing to
//! the database is a decision, not a side effect of starting a server. The api
//! reads the content repo from disk on every boot and rereads it on SIGHUP —
//! neither of those should ever write a row, or a container restart would be a
//! data change nobody asked for.
//!
//! It is also why this does not live in `ohara`. That crate knows what the
//! content *is*, and nothing that only reads content should be linking a
//! database writer.
//!
//! # What this is not
//!
//! Not a migration runner. `lighthouse-migrate` owns the schema; this fills
//! rows in tables that already exist, and fails loudly if they do not.
//!
//! Not a content deploy either. What a reader reads is served from disk by the
//! api — `make content-sync` is what makes a running process reread it. These
//! rows exist so that notes, bookmarks, completions and entitlements have
//! something to point at.

mod sync;

use std::process::ExitCode;

use ohara::{Content, Drafts, catalog::Catalog};

const USAGE: &str = "usage: lighthouse-content <sync>";

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
    // Checked before touching anything, as `lighthouse-migrate` does: a typo in
    // the subcommand should say so rather than open a connection first.
    if std::env::args().nth(1).as_deref() != Some("sync") {
        return Err(USAGE.to_owned());
    }

    // Same rule as everywhere else in this workspace: a missing .env is fine —
    // the runtime image ships without one — and an unreadable one is not.
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        return Err(format!("cannot read .env: {error}"));
    }

    let path = std::env::var("CONTENT_PATH").map_err(|_| {
        "CONTENT_PATH is not set in .env or the environment".to_owned()
    })?;
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        "DATABASE_URL is not set in .env or the environment".to_owned()
    })?;

    // Parsed before connected, deliberately. A content repo with a typo in it
    // should fail naming the file, without a transaction having been opened and
    // without the database having been touched at all.
    // The same policy the api boots with, read the same way. A draft row in
    // production is the thing this is meant to prevent, and the sync is what
    // would create it — so `SHOW_DRAFTS` has to reach here too, not just the
    // server.
    let drafts = Drafts::from_env(std::env::var("SHOW_DRAFTS").ok().as_deref());

    let catalog = Catalog::load(Content::at(&path), drafts)
        .map_err(|error| format!("{error}"))?;

    // A single connection's worth of work, but sqlx's transaction API takes a
    // pool. One connection, so this cannot hold more of the database open than
    // it is using.
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|error| {
            format!("cannot connect to {}: {error}", redacted(&url))
        })?;

    let synced = sync::run(&db, &catalog.current())
        .await
        .map_err(|error| format!("{error}"))?;

    // Names the counts rather than printing "done", for the reason
    // `lighthouse-migrate` names the migrations it applied: a write command
    // that is silent about what it wrote is one you have to go and check.
    println!(
        "synced {} books and {} lessons from {path} ({drafts:?} drafts)",
        synced.books, synced.lessons
    );

    Ok(())
}

/// The connection target, without the password.
///
/// Lifted from `lighthouse-migrate`, and duplicated rather than shared: eight
/// lines with no logic in them is not a crate, and pointing at the wrong host
/// is the most common reason this fails.
fn redacted(url: &str) -> String {
    match url.rsplit_once('@') {
        Some((_, target)) => target.to_owned(),
        None => url.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_password_never_reaches_a_message() {
        let url = "postgres://lighthouse:hunter2@127.0.0.1:5433/lighthouse";

        assert_eq!(redacted(url), "127.0.0.1:5433/lighthouse");
        assert!(!redacted(url).contains("hunter2"));
    }

    #[test]
    fn a_url_with_no_credentials_is_left_as_it_is() {
        assert_eq!(
            redacted("postgres:///lighthouse"),
            "postgres:///lighthouse"
        );
    }
}
