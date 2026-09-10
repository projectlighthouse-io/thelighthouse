//! The subscribe form anybody can fill in.
//!
//! ```text
//!   POST /api/newsletter    put an address on the list
//! ```
//!
//! **This is the anonymous half of the newsletter.** `settings::newsletter` is
//! the other one: a signed-in reader toggling `users.newsletter_enabled`, whose
//! address comes from their account rather than from a form. Both end at the
//! same Kit list through the same `settings::kit::subscribe`, and that is the
//! only thing they share — this one has no session, no reader and no column to
//! write, because the visitor filling it in may well not have a row in `users`
//! at all.
//!
//! Nothing is stored on this side. Kit holds the list; asking it who is on
//! there is a question for Kit, not for a table here that would immediately
//! start disagreeing with it.
//!
//! **No CSRF token, deliberately.** Every other write in this api carries one,
//! and every one of those is bound to a session — the attack CSRF prevents is
//! making a victim's *own* browser act with a victim's *own* credentials, and
//! there is no credential here to borrow. A forged cross-site POST to this
//! endpoint subscribes whatever address the forger typed, which is a thing they
//! could equally do by opening the page. What stops that being interesting is
//! the rate limit below.
//!
//! **An address is never confirmed as already subscribed.** Kit answers the
//! same way for a new subscriber and an existing one, and this passes that
//! through unchanged: a form that says "you are already on the list" is a form
//! that answers "is this person a reader of yours" for anybody who asks.

use axum::http::{HeaderValue, header::RETRY_AFTER};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;

use crate::{
    api::AppState, cache::CachePolicy, middleware::rate::caller, response,
    settings::kit,
};

/// What `users.email` holds, and what Kit will take.
const EMAIL_LIMIT: usize = 254;

/// What the form sends.
#[derive(Debug, Deserialize)]
pub(crate) struct Subscribe {
    email: String,
}

pub(crate) fn routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/newsletter", post(subscribe))
        .route_layer(from_fn_with_state(state.clone(), limit_subscribes))
}

/// `POST /api/newsletter` — put an address on the list.
///
/// Answers 202 rather than 200: the address has been handed to Kit, and
/// whether it turns into mail is Kit's to say, not this endpoint's.
async fn subscribe(
    State(state): State<AppState>,
    Json(body): Json<Subscribe>,
) -> Response {
    let email = body.email.trim();

    if !is_email(email) {
        return refuse(
            StatusCode::UNPROCESSABLE_ENTITY,
            "not_an_email",
            "Enter an email address.",
        );
    }

    if !kit::subscribe(&state.config.kit_api_key, email).await {
        return refuse(
            StatusCode::SERVICE_UNAVAILABLE,
            "newsletter_provider_refused",
            "Could not reach the newsletter provider. Try again.",
        );
    }

    // The address is deliberately not logged. It is the whole of what a
    // visitor handed over, and a subscribe line in a log is a mailing list
    // sitting in the log store.
    tracing::info!("newsletter subscription taken");

    response::json(
        StatusCode::ACCEPTED,
        serde_json::json!({ "subscribed": true }),
        CachePolicy::NoStore,
    )
}

/// Whether this is worth handing to Kit.
///
/// Deliberately not a grammar for RFC 5322, which permits quoted local parts,
/// comments and addresses no provider would accept anyway. This refuses what is
/// plainly not an address, and lets Kit be the judge of the rest — it is the
/// one that has to deliver to it.
///
/// ponytail: string checks rather than a validation crate. The rule this
/// enforces is "has a local part, an `@`, and a host with a dot in it", and a
/// dependency that agrees with that is a dependency earning nothing.
fn is_email(value: &str) -> bool {
    if value.is_empty() || value.len() > EMAIL_LIMIT {
        return false;
    }

    // Whitespace anywhere means the form sent two things, or one thing with a
    // typo in it. Either way it is not an address.
    if value.chars().any(char::is_whitespace) {
        return false;
    }

    // `rsplit_once`, so an `@` in the local part leaves the host intact rather
    // than truncating at the first one.
    let Some((local, host)) = value.rsplit_once('@') else {
        return false;
    };

    if local.is_empty() || host.len() < 3 {
        return false;
    }

    // A host with no dot is a local name — `postmaster@localhost` is valid and
    // is never what somebody typed into a newsletter box on a website.
    match host.split_once('.') {
        Some((name, rest)) => !name.is_empty() && !rest.is_empty(),
        None => false,
    }
}

/// How often one address may be handed to Kit.
///
/// Keyed by caller rather than by the email in the body: the body is the part
/// an abuser varies, so counting it would be counting nothing. `state.limits`
/// is the write budget — ten a minute — under its own namespace so a visitor
/// subscribing cannot spend a signed-in reader's note writes or the other way
/// round.
///
/// **This is the only thing standing between the form and Kit's api**, which
/// is why it is stricter than the sixty-a-minute limit every route already
/// inherits: each call through here is an outbound request on our api key, so
/// an unlimited form is an amplifier pointed at our own provider account.
async fn limit_subscribes(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let key = format!("newsletter:{}", caller(&request));

    let Some(retry_after) = state.limits.check(&key) else {
        return next.run(request).await;
    };

    tracing::info!(%key, "newsletter rate limit reached");

    let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();

    // A 429 with no `Retry-After` tells a client to back off but not by how
    // much, so it guesses, and the guess is usually "immediately".
    if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
        response.headers_mut().insert(RETRY_AFTER, value);
    }

    response
}

/// The refusal a caller sees.
///
/// Two cases and no enum: `settings::refusal` earns one by having nine variants
/// shared across four handlers, and this module has one handler. A third case
/// here is the point at which it wants the same treatment.
fn refuse(status: StatusCode, code: &str, message: &str) -> Response {
    response::json(
        status,
        serde_json::json!({ "code": code, "error": message }),
        CachePolicy::NoStore,
    )
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use tower::ServiceExt as _;

    use super::*;
    use crate::config::Config;

    /// The real router, so these prove where the route is mounted rather than
    /// what the handler would do if reached. No database and no Kit: every
    /// case here is refused before either is touched.
    fn router() -> axum::Router {
        let config = Config::sample();
        let socials = loginwith::providers(vec![]).unwrap();
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();
        let catalog = std::sync::Arc::new(
            ohara::catalog::Catalog::load(
                ohara::fixture::content(),
                ohara::Drafts::Hidden,
            )
            .unwrap(),
        );

        crate::api::app(
            config,
            socials,
            db,
            catalog,
            crate::api::Billing::sample(),
        )
    }

    async fn post(body: &str) -> StatusCode {
        router()
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/api/newsletter")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    /// The point of the endpoint: no session, no CSRF token, no signature, and
    /// it still gets past the door. If this ever starts answering 401 or 404
    /// the form is broken for exactly the visitors it exists for.
    ///
    /// Asserted through the refusal rather than a success, so the test never
    /// calls Kit: a malformed address is turned away by `is_email`, which is
    /// upstream of the one line that leaves the process.
    #[tokio::test]
    async fn the_form_needs_no_credential_to_be_refused_on_its_merits() {
        assert_eq!(
            post(r#"{"email":"not-an-address"}"#).await,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    /// A body with no `email` is a client bug, not a bad address, and axum
    /// rejects it before the handler runs.
    #[tokio::test]
    async fn a_body_without_an_email_is_refused() {
        assert_eq!(post(r"{}").await, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn an_ordinary_address_passes() {
        assert!(is_email("reader@example.com"));
        assert!(is_email("a.b+tag@mail.example.co.uk"));
    }

    #[test]
    fn what_is_plainly_not_an_address_is_refused() {
        assert!(!is_email(""));
        assert!(!is_email("reader"));
        assert!(!is_email("@example.com"));
        assert!(!is_email("reader@"));
        assert!(!is_email("reader@localhost"));
        assert!(!is_email("reader@.com"));
        assert!(!is_email("reader@example."));
        assert!(!is_email("two things@example.com"));
    }

    #[test]
    fn an_address_longer_than_the_column_is_refused() {
        let long = format!("{}@example.com", "a".repeat(EMAIL_LIMIT));

        assert!(!is_email(&long));
    }

    /// The local part may hold an `@`, so the host is what follows the *last*
    /// one — splitting at the first would read `b@example.com` as the host.
    #[test]
    fn the_host_is_what_follows_the_last_at() {
        assert!(is_email("a@b@example.com"));
    }
}
