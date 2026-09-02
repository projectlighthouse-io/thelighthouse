//! What a reader is paying for, and how they start or stop paying.
//!
//! ```text
//!   GET  /api/billing/membership
//!   POST /api/billing/{provider}/checkout
//!   POST /api/billing/{provider}/cancel
//!   POST /api/billing/{provider}/resume
//!   POST /api/billing/{provider}/swap
//!   POST /webhooks/{provider}
//! ```
//!
//! ```text
//!   mod.rs         the routes, and this map
//!   handler.rs     one function per route, and no sql
//!   payload.rs     what a reader sends, and whether it is acceptable
//!   store.rs       every query, and the only thing that talks to postgres
//!   membership.rs  a membership, as a row and as json
//!   refusal.rs     why a write was refused, and its wire code
//! ```
//!
//! Named `payments` while the routes say `billing`, deliberately. The module
//! is what this process does — take payments and record them — and `billing`
//! is what a reader calls the page they manage a subscription on. The name is
//! also not `billing` because the crate that talks to providers already is,
//! and two `billing`s in one file is a path nobody can read.
//!
//! **Talking to a provider is not this module's job.** That is the `billing`
//! crate, which knows the wire format and nothing about readers, sessions or
//! postgres. This module owns the `memberships` table and the decision of what
//! a reader is told; between them the line is that no Stripe noun crosses it.
//!
//! **`{provider}` is in the path for the same reason `{provider}` is in the
//! OAuth routes.** A second way to pay is a registration at boot rather than a
//! second set of endpoints, and an unknown one is a 404 — a url that does not
//! exist, not a malformed request.
//!
//! **Price is not consulted here, and entitlement is not decided here.** What
//! a plan costs lives at the provider; what an active membership unlocks is
//! `books::entitlement`. This module answers only whether one is live.
//!
//! Nothing is cacheable. Every response depends on what somebody has paid for,
//! and `private` alone still permits a browser cache on a shared machine.

mod handler;
mod membership;
mod payload;
mod refusal;
mod store;
#[cfg(test)]
mod tests;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
};

use crate::{
    api::AppState,
    middleware::{
        csrf::require_csrf, reader::require_reader, throttle::throttle,
    },
};

/// Whether a reader is paying for anything, for `books::entitlement`.
///
/// Re-exported rather than reaching into `store` from outside, so this module
/// keeps owning its table: one caller elsewhere means one function here, not a
/// second module writing queries against `memberships`.
pub(crate) use store::live;

/// The reader-facing routes. Absolute paths, so these merge alongside the
/// others rather than nesting under one prefix.
///
/// Every route needs a reader; the four that change something also need a CSRF
/// token and a place in the write limit. A checkout that could be triggered
/// cross-site is a reader sent to a payment page they did not ask for.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let reads =
        Router::new().route("/api/billing/membership", get(handler::show));

    let writes = Router::new()
        .route("/api/billing/{provider}/checkout", post(handler::checkout))
        .route("/api/billing/{provider}/cancel", post(handler::cancel))
        .route("/api/billing/{provider}/resume", post(handler::resume))
        .route("/api/billing/{provider}/swap", post(handler::swap))
        .route_layer(from_fn_with_state(state.clone(), throttle))
        .route_layer(from_fn(require_csrf));

    reads
        .merge(writes)
        .route_layer(from_fn_with_state(state.clone(), require_reader))
}

/// Where providers report what happened.
///
/// Deliberately outside the reader gate and the CSRF check, exactly as the
/// OAuth callbacks are: a provider has no session and no token. It
/// authenticates itself — the driver verifies a signature over the raw body,
/// in constant time — and that check is the only thing standing in front of
/// it, which is why it is not caddy's to make.
pub(crate) fn webhook_routes() -> Router<AppState> {
    Router::new().route("/webhooks/{provider}", post(handler::delivered))
}
