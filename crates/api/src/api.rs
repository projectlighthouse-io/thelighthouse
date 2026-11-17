//! What is mounted where, and behind which gates.
//!
//! `main` starts this; the gates themselves are `middleware`. Reading `app`
//! top to bottom should be enough to say what any request can reach — which is
//! why the layering lives here and not spread across the modules being mounted.
//!
//! Each feature module — `projects`, `books`, `payments` and the rest — owns
//! its own routes; this builds the router from them and puts the gates around
//! them.

use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
};
use loginwith::Providers;
use ohara::catalog::Catalog;
use sqlx::postgres::PgPool;

use crate::{
    articles, auth, bookmarks, books,
    config::Config,
    db,
    limit::RateLimit,
    middleware::{rate::limit_requests, signature::require_signature},
    newsletter, notes, payments, profiles, projects, settings, telemetry,
};

/// Everything needed to take money: the drivers registered at boot, and the
/// plans on sale.
///
/// One field on the state rather than two, because they are only ever used
/// together — a driver with no plan to sell has nothing to do, and a plan with
/// no driver cannot be bought.
#[derive(Clone, Debug)]
pub(crate) struct Billing {
    /// Every provider this process can take money through, by name.
    pub(crate) providers: billing::Providers,
    /// Shared rather than cloned: the plan list is read on every checkout and
    /// never changes after boot.
    pub(crate) plans: Arc<billing::Plans>,
}

#[cfg(test)]
impl Billing {
    /// A registry with the Stripe driver in it, pointed at nothing.
    ///
    /// Enough to prove what is mounted and which gates it sits behind. A test
    /// that needs the driver to answer stands a server up in front of it — see
    /// the `billing` crate's own tests.
    pub(crate) fn sample() -> Self {
        const PLANS: &str = "
plans:
  - id: yearly
    price: price_yearly
    interval: year
";

        let providers = billing::providers([billing::Stripe::register(
            billing::StripeConfig {
                secret_key: "sk_test".into(),
                webhook_secret: "whsec_test".into(),
                strategy: billing::RequestStrategy::Once,
            },
        )
        .unwrap()])
        .unwrap();

        Self {
            providers,
            plans: Arc::new(billing::Plans::from_yaml(PLANS).unwrap()),
        }
    }
}

/// What every handler can reach. Cheap to clone — `PgPool` and `Providers` are
/// both handles to something shared, and `Config` is a handful of strings.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) socials: Providers,
    pub(crate) db: PgPool,
    /// Shared, not cloned: an `Arc` so every clone of this state counts against
    /// the same buckets. A `RateLimit` per clone would be a limit per request.
    pub(crate) limits: Arc<RateLimit>,
    /// The limit every route inherits, keyed by reader or address. Shared for
    /// the same reason `limits` is: a per-clone counter is a per-request one.
    pub(crate) requests: Arc<RateLimit>,
    /// Shared for the same reason, and more so: a SIGHUP reload swaps the
    /// snapshot inside this one, and a per-clone catalogue would leave most
    /// requests reading a copy nothing ever reloads.
    pub(crate) catalog: Arc<Catalog>,
    /// How money is taken, and what is on sale.
    pub(crate) billing: Billing,
}

pub(crate) fn app(
    config: Config,
    socials: Providers,
    db: PgPool,
    catalog: Arc<Catalog>,
    billing: Billing,
) -> Router {
    // Read before `config` moves into the state.
    let (requests, note_writes) =
        (config.request_limit, config.note_write_limit);

    let state = AppState {
        config,
        socials,
        db,
        limits: Arc::new(RateLimit::note_writes(note_writes)),
        requests: Arc::new(RateLimit::requests(requests)),
        catalog,
        billing,
    };

    // luxctl's surface. Everything mounted here inherits the signature check,
    // and each route decides for itself whether a bearer token is required or
    // merely read — see `projects::routes`.
    let signed = projects::routes(&state).route_layer(from_fn_with_state(
        state.config.clone(),
        require_signature,
    ));

    // Everything a caller off the internet can reach, under one limit.
    //
    // `RateLimiter::for('api')` from the laravel app — sixty a minute — and it
    // is the floor, not the whole story: the note writes keep their own
    // stricter budget inside this one, exactly as a laravel route carrying
    // both `throttle:api` and `throttle:notes` does.
    let public = Router::new()
        // Absolute paths throughout, so these merge alongside each other rather
        // than nesting under one prefix. Each carries its own gates — see their
        // `routes`.
        .merge(signed)
        // The website's own project endpoints, which carry no signature — a
        // browser cannot make one. See `projects::page_routes`.
        .merge(projects::page_routes(&state))
        .merge(books::routes())
        // The blog: markdown readers wrote. Open to read, behind the session
        // cookie to write — see `articles::routes`.
        .merge(articles::routes(&state))
        // The subscribe form, open to anybody — no session, no CSRF token and
        // no reader. See `newsletter`, which says why each of those is absent.
        .merge(newsletter::routes(&state))
        .merge(notes::routes(&state))
        .merge(bookmarks::routes(&state))
        .merge(payments::routes(&state))
        .merge(settings::routes(&state))
        // Public profiles: open to anybody, no session. See `profiles`.
        .merge(profiles::routes())
        // Deliberately outside the reader gate: an OAuth callback is a browser
        // navigation and cannot carry an HMAC or a session. Those routes
        // authenticate themselves — see `auth`.
        .merge(auth::routes(&state))
        // Outside the reader gate for the same reason the OAuth callbacks
        // are: a payment provider has no session and no CSRF token. It
        // authenticates itself by signing what it sends.
        .merge(payments::webhook_routes())
        .layer(from_fn_with_state(state.clone(), limit_requests));

    Router::new()
        // Outside the limit. `/health` is polled on a timer by whatever is
        // watching the container, and a probe that starts failing because it
        // probed too often is worse than no probe.
        .route("/health", get(health))
        .merge(public)
        .layer(axum::middleware::from_fn(telemetry::trace_request))
        // Outermost, and in this order: axum applies layers bottom-up, so the
        // hub exists before the http layer starts a transaction in it. The
        // Sentry docs warn the reverse order leaks memory.
        .layer(
            sentry::integrations::tower::SentryHttpLayer::new()
                .enable_transaction(),
        )
        .layer(sentry::integrations::tower::NewSentryLayer::<
            axum::extract::Request,
        >::new_from_top())
        .with_state(state)
}

/// Unsigned on purpose. Caddy never routes to it, so it is reachable only from
/// inside the container, which is where the health probe runs.
///
/// It checks the database, because the probe's job is to answer "should this
/// container keep serving traffic" and a process that cannot reach postgres
/// cannot serve a single page. A health check that only proves the process is
/// running is the kind that stays green through an outage.
async fn health(State(state): State<AppState>) -> Response {
    if db::is_reachable(&state.db).await {
        (StatusCode::OK, "ok").into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "database unreachable")
            .into_response()
    }
}
