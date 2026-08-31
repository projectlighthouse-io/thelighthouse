//! luxctl's surface: projects, tasks, hints and the attempt log.
//!
//! ```text
//!   GET  /api/v1/ping                                  is anyone there
//!   GET  /api/v1/health                                and is the database
//!   GET  /api/v1/projects                              every published project
//!   GET  /api/v1/projects/{identifier}                 one, with its blueprint
//!   GET  /api/v1/projects/{identifier}/tasks           its tasks alone
//!   GET  /api/v1/tasks/{identifier}                    one task, with prose
//!
//!   GET  /api/v1/user                                  who this token is
//!   POST /api/v1/projects/attempts                     record one submission
//!   POST /api/v1/projects/{identifier}/restart         start the project again
//!   GET  /api/v1/tasks/{identifier}/hints              the hints, some worded
//!   POST /api/v1/tasks/{identifier}/hints/{id}/unlock  buy one
//! ```
//!
//! And the website's own, which is a different caller and so a different door:
//!
//! ```text
//!   GET  /api/projects                    the projects page
//!   GET  /api/projects/{slug}             one project's page
//!   GET  /api/projects/{slug}/tasks/{task}  one task's brief
//!   GET  /api/projects/{slug}/progress    what an open tab polls
//! ```
//!
//! **Unsigned, because a browser cannot carry an HMAC** — the split
//! `books::routes` already makes. The first two are the same bytes for
//! everybody and are held at the edge; the third is behind the session cookie
//! and behind `no-store`.
//!
//! ```text
//!   mod.rs          the routes, and this map
//!   handler.rs      one function per route, and no sql
//!   view.rs         what luxctl sees, as json
//!   payload.rs      what luxctl sends, and what of it is believed
//!   progress.rs     the attempt log, read back as a reader's standing
//!   entitlement.rs  which tasks a reader may work on
//!   store.rs        every query, and the only thing that talks to postgres
//!   refusal.rs      why a write was refused, and its wire code
//! ```
//!
//! **`identifier` is a uuid or a slug.** luxctl holds uuids from the project
//! listing and readers type slugs, and both reach the same handler — see
//! `ohara::catalog::Snapshot::project_by`.
//!
//! **Ten endpoints, not eleven.** `POST /tasks/{identifier}/submit` is not
//! here. It compared a typed answer against `task_validators.validator_dsl`,
//! which was synced out of the very blueprint the api hands to luxctl — the
//! answer key ships to the client, so checking it here was checking a secret we
//! had already given away. The table is dropped, nothing in this workspace
//! parses `.bp`, and luxctl already holds the blueprint: it validates locally
//! and posts the outcome to `/projects/attempts` like every other task.
//!
//! **What the client says about itself is not believed.** Points come from the
//! ladder in ohara, graded against a count and a duration read out of the log.
//! The run comes from `project_restarts`. Neither is a field on any request
//! body here, so neither can be chosen.
//!
//! **The reader is optional on the first five and required on the rest.** A
//! signed-out `lux project show` still lists the tasks; it simply carries no
//! `progress`. That is what the laravel handlers did with `auth('sanctum')` on
//! a public route, kept because a reader deciding whether to start a project
//! should not have to log in first.

mod entitlement;
mod handler;
mod payload;
mod progress;
mod refusal;
mod store;
#[cfg(test)]
mod tests;
mod view;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    api::AppState,
    middleware::{
        bearer::{optional_token, require_token},
        reader::require_reader,
    },
};

/// Absolute paths, so this merges alongside the other routers rather than
/// nesting under one prefix.
///
/// **Everything here is signed** — the layer is applied by the caller, in
/// `api::app`, because it is the whole `/api/v1` surface's gate rather than
/// this module's. What this decides is the second gate: whether a token is
/// required or merely read.
pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    let optional = Router::new()
        .route("/api/v1/ping", get(handler::ping))
        .route("/api/v1/health", get(handler::health))
        .route("/api/v1/projects", get(handler::list))
        .route("/api/v1/projects/{identifier}", get(handler::show))
        .route("/api/v1/projects/{identifier}/tasks", get(handler::tasks))
        .route("/api/v1/tasks/{identifier}", get(handler::task))
        .route_layer(from_fn_with_state(state.clone(), optional_token));

    let required = Router::new()
        .route("/api/v1/user", get(handler::me))
        .route("/api/v1/projects/attempts", post(handler::record))
        .route(
            "/api/v1/projects/{identifier}/restart",
            post(handler::restart),
        )
        .route("/api/v1/tasks/{identifier}/hints", get(handler::hints))
        .route(
            "/api/v1/tasks/{identifier}/hints/{hint}/unlock",
            post(handler::unlock),
        )
        .route_layer(from_fn_with_state(state.clone(), require_token));

    optional.merge(required)
}

/// The website's surface: unsigned, and gated on a session where it needs to
/// be.
///
/// Merged separately in `api::app`, *outside* the signature layer. Two routers
/// rather than one with holes in it: which door a route is behind should be
/// visible where it is mounted, and a signed router with three exceptions in it
/// is a router whose exceptions grow.
///
/// Caddy needs a rule per browser-facing prefix — see the Caddyfile. That is
/// the point rather than an inconvenience: reaching the api from a browser is a
/// decision written down, not something a new route inherits.
pub(crate) fn page_routes(state: &AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/api/projects", get(handler::page_list))
        .route("/api/projects/{slug}", get(handler::page_show))
        .route("/api/projects/{slug}/tasks/{task}", get(handler::page_task));

    let reader = Router::new()
        .route("/api/projects/{slug}/progress", get(handler::progress))
        .route_layer(from_fn_with_state(state.clone(), require_reader));

    public.merge(reader)
}
