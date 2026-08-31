//! Boot. Logging, then config, then bind and serve.
//!
//! Nothing about the HTTP surface lives here — routes, the signature boundary
//! and the cache policy are all in `api`. This file is only the order the
//! process comes up in, which is the one thing that has to be read top to
//! bottom.

mod api;
mod auth;
mod bookmarks;
mod books;
mod cache;
mod config;
mod cookie;
mod db;
mod limit;
mod middleware;
mod notes;
mod projects;
mod request;
mod response;
mod session;
mod telemetry;
#[cfg(test)]
mod testing;
mod tokens;
mod users;

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use loginwith::{GithubProvider, GoogleProvider, Providers, Registration};

use config::Config;

/// Returning an error rather than panicking: a failure to bind should print
/// something a human can act on and exit non-zero, not dump a backtrace. The
/// lints deny `expect` for this reason.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    telemetry::init();

    // Before `Config`, deliberately: checking the content needs `CONTENT_PATH`
    // and nothing else, and a deploy gate that also demanded oauth secrets and
    // a reachable database would not be a content check.
    if std::env::args().any(|arg| arg == "--check-content") {
        return check_content();
    }

    // Every read of the environment happens here. Past this line the process
    // deals in `Config`, so a key can only be missing at startup.
    let config = Config::load()?;

    // Before the listener: a bad credential should be a startup failure with a
    // provider name on it, not a 500 the first time somebody tries to sign in.
    let socials = social_providers(&config)?;

    // Before the listener too, and for the same reason: a database this process
    // cannot reach is not a site that can serve anything, so it should fail the
    // boot loudly rather than answer 500s. Migrations are not run here — see
    // `db` for why a deploy must not double as a schema change.
    let db = db::connect(&config.database_url).await?;

    tracing::info!("database connected");

    // Before the listener, and for a third time the same reason: a site that
    // boots with no content looks broken rather than down, and there is no
    // previous snapshot to fall back on. A *reload* keeps what it has — see
    // `Catalog::reload` — but the first read has to be right.
    let catalog = Arc::new(ohara::catalog::Catalog::load(
        ohara::Content::at(&config.content_path),
        config.drafts,
    )?);

    tracing::info!(
        books = catalog.current().books().count(),
        path = %config.content_path,
        drafts = ?config.drafts,
        "content loaded"
    );

    // SIGHUP rereads the content repo in place. No endpoint, so there is no
    // authenticated write surface that reads the filesystem, and no restart, so
    // in-flight requests finish against the snapshot they already hold:
    //
    //   docker kill -s HUP lighthouse
    reload_on_hangup(Arc::clone(&catalog));

    tracing::info!(
        providers = ?socials.registered().map(loginwith::Provider::as_str).collect::<Vec<_>>(),
        "social sign-in ready"
    );

    // Loopback, not 0.0.0.0. Caddy is the only public listener in the
    // container; binding wider would let anything on the host reach the api
    // without passing the signature check.
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, config.api_port));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(%addr, "api listening");

    axum::serve(listener, api::app(config, socials, db, catalog))
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
}

/// Walks the content repo, reports what is in it, and exits.
///
/// The same walk the boot does, so a book that would stop this process from
/// starting fails in the deploy step instead — with the offending file named.
/// Run it before shipping: `make content-check`.
///
/// Also prints the lesson folders no chapter lists. Those are drafts and not
/// errors, but "I wrote a lesson and it never appeared" is the mistake this
/// design makes easy, so the check says them out loud.
fn check_content() -> Result<(), Box<dyn std::error::Error>> {
    let path =
        std::env::var("CONTENT_PATH").map_err(|_| "CONTENT_PATH is not set")?;

    let content = ohara::Content::at(&path);
    // Hidden: this gate answers the question production will ask, so a
    // draft must not pad the counts it reports.
    let catalog =
        ohara::catalog::Catalog::load(content.clone(), ohara::Drafts::Hidden)?;
    let snapshot = catalog.current();

    let mut lessons = 0;
    let mut drafts = Vec::new();

    for book in snapshot.books() {
        let published: Vec<&str> =
            book.lessons().map(|entry| entry.folder.as_str()).collect();

        lessons += published.len();

        println!(
            "{:24} {:3} lessons  {}",
            book.book.slug,
            published.len(),
            if book.book.price.is_free() {
                "free".to_owned()
            } else {
                format!("{}", book.book.price.amount)
            }
        );

        for folder in content.lesson_folders(&book.book.slug)? {
            if !published.contains(&folder.as_str()) {
                drafts.push(format!("{}/{folder}", book.book.slug));
            }
        }
    }

    println!(
        "\n{} books, {lessons} lessons, from {path}",
        snapshot.books().count()
    );

    if !drafts.is_empty() {
        println!("\nnot listed by any chapter, so not published:");
        for draft in drafts {
            println!("  {draft}");
        }
    }

    Ok(())
}

/// Rereads the content repo whenever the process is sent SIGHUP.
///
/// A failed reload is logged and nothing else: the previous snapshot stays in
/// place, so a typo in one lesson leaves the site serving what it already had
/// rather than emptying it. Whoever sent the signal reads the log to find out.
///
/// Unix only, which is every environment this runs in. On anything else there
/// is no SIGHUP to listen for and the catalogue is simply whatever booted.
#[cfg(unix)]
fn reload_on_hangup(catalog: Arc<ohara::catalog::Catalog>) {
    use tokio::signal::unix::{SignalKind, signal};

    let Ok(mut hangups) = signal(SignalKind::hangup()) else {
        tracing::warn!("cannot listen for SIGHUP; content reload unavailable");
        return;
    };

    tokio::spawn(async move {
        while hangups.recv().await.is_some() {
            match catalog.reload() {
                Ok(()) => tracing::info!(
                    books = catalog.current().books().count(),
                    "content reloaded"
                ),
                Err(cause) => tracing::error!(
                    %cause,
                    "failed to reload content; keeping the previous one"
                ),
            }
        }
    });
}

#[cfg(not(unix))]
fn reload_on_hangup(_catalog: Arc<ohara::catalog::Catalog>) {}

/// Registers each provider that has credentials.
///
/// An empty client id skips that provider rather than registering a broken one,
/// so `/api/auth/google` is a 404 on a deployment with no Google app instead of
/// a redirect that dead-ends at Google's error page. The keys stay required in
/// `Config` — the choice not to configure one has to be written down as an
/// empty value, not left out.
fn social_providers(config: &Config) -> Result<Providers, loginwith::Error> {
    let github = (!config.github_id.is_empty()).then(|| {
        GithubProvider::with(
            &config.github_id,
            &config.github_secret,
            config.callback_url("github"),
        )
    });

    let google = (!config.google_id.is_empty()).then(|| {
        GoogleProvider::with(
            &config.google_id,
            &config.google_secret,
            config.callback_url("google"),
        )
    });

    loginwith::providers(
        [github, google]
            .into_iter()
            .flatten()
            .collect::<Vec<Registration>>(),
    )
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
