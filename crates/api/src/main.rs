//! Boot. Logging, then config, then bind and serve.
//!
//! Nothing about the HTTP surface lives here — routes, the signature boundary
//! and the cache policy are all in `api`. This file is only the order the
//! process comes up in, which is the one thing that has to be read top to
//! bottom.

mod api;
mod auth;
mod cache;
mod config;
mod content;
mod cookie;
mod db;
mod limit;
mod middleware;
mod notes;
mod request;
mod response;
mod session;
mod telemetry;
mod users;

use std::net::{Ipv4Addr, SocketAddr};

use loginwith::{GithubProvider, GoogleProvider, Providers, Registration};

use config::Config;

/// Returning an error rather than panicking: a failure to bind should print
/// something a human can act on and exit non-zero, not dump a backtrace. The
/// lints deny `expect` for this reason.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    telemetry::init();

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

    axum::serve(listener, api::app(config, socials, db))
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
}

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
