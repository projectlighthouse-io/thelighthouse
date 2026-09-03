//! `Kit`, and the one call this site makes to it.
//!
//! Kit — `ConvertKit` until they renamed — is where the newsletter list lives.
//! The list is theirs, not ours: `users.newsletter_enabled` records that a
//! reader asked, and Kit records who actually gets sent to. Both have to be
//! written for a subscription to mean anything, which is why the handler
//! refuses when this call fails rather than saving the flag and reporting a
//! success the reader would never see an email from.
//!
//! **Subscribing is the whole surface.** Turning the toggle off writes the
//! column and stops there. Kit's v4 api is not asked to unsubscribe anybody,
//! because the laravel app never asked it either — its
//! `UnsubscribeFromNewsletter` flips the same column and syncs to `MailerLite`,
//! a different provider. Sending reads the column, so a reader who turns it
//! off stops receiving mail; the row at Kit going stale is a known cost, and
//! reconciling it is a job for whatever ends up doing the sending.
//!
//! The endpoint and header are the ones the laravel integration uses, which is
//! the only description of this api this repository can check against.

use secrecy::{ExposeSecret, SecretString};

/// Kit v4. Creating a subscriber that already exists answers with the existing
/// one rather than an error, so this is safe to call for a reader who has
/// subscribed, unsubscribed and subscribed again.
const SUBSCRIBERS: &str = "https://api.kit.com/v4/subscribers";

/// Tells Kit this address wants the newsletter.
///
/// Answers whether Kit took it. The failure is logged here, with the status and
/// body, because that is the only place the provider's own words are available
/// — the caller gets a bool and a refusal to send on.
///
/// ponytail: a `reqwest::Client` per call, rather than one in `AppState`. This
/// runs when somebody flips a checkbox on a settings page, so the connection
/// pool it throws away had one connection in it. Move the client into the
/// state if anything else here starts talking to Kit.
pub(crate) async fn subscribe(api_key: &SecretString, email: &str) -> bool {
    let request = reqwest::Client::new()
        .post(SUBSCRIBERS)
        .header("X-Kit-Api-Key", api_key.expose_secret())
        .json(&serde_json::json!({ "email_address": email }))
        .send()
        .await;

    let response = match request {
        Ok(response) => response,
        Err(cause) => {
            tracing::warn!(%cause, "failed to reach kit");
            return false;
        }
    };

    let status = response.status();

    if status.is_success() {
        return true;
    }

    // Read after the status, and only on the failing path: the body is what
    // says *why* Kit refused, and on the happy path nothing reads it.
    let body = response.text().await.unwrap_or_default();
    tracing::warn!(%status, %body, "kit refused a subscriber");

    false
}
