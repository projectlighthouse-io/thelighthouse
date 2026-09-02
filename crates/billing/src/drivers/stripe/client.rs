//! The http half of the Stripe driver: one request, retried to a plan.

use std::fmt;

use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;

use super::wire;
use crate::{error::Error, request::RequestStrategy};

/// The api version every request pins.
///
/// Pinned rather than left to the account default, because the shape of a
/// response is what this driver's parsing is written against and an account
/// setting somebody changes in a dashboard must not silently change it.
///
/// Note this does **not** pin webhook payloads: those are serialised with the
/// account's version, or the version set when the endpoint was created. That
/// is why [`wire::Sub`] reads the period end from both places it has lived.
pub(crate) const API_VERSION: &str = "2026-08-26.dahlia";

const BASE: &str = "https://api.stripe.com";

/// Identifies this crate in Stripe's request logs, which is what turns "some
/// integration created this subscription" into a name.
const USER_AGENT: &str = concat!("billing/", env!("CARGO_PKG_VERSION"));

/// Long enough for a slow checkout session, short enough that a wedged
/// connection does not hold a request handler open indefinitely.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub(crate) struct Client {
    http: reqwest::Client,
    secret: String,
    base: String,
    strategy: RequestStrategy,
}

/// Hand written, so a state dump cannot print the api key.
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base", &self.base)
            .field("secret", &"<redacted>")
            .field("strategy", &self.strategy)
            // The http client itself has no state worth printing, and the
            // secret above is the reason this impl is written by hand.
            .finish_non_exhaustive()
    }
}

impl Client {
    pub(crate) fn new(
        secret: impl Into<String>,
        strategy: RequestStrategy,
    ) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .build()
            .map_err(|source| Error::Unreachable {
                cause: source.to_string(),
            })?;

        Ok(Self {
            http,
            secret: secret.into(),
            base: BASE.to_owned(),
            strategy,
        })
    }

    /// Point the driver somewhere else. For tests, against a local server.
    #[cfg(test)]
    pub(crate) fn with_base(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }

    /// Send one request, retrying it as far as the strategy allows.
    ///
    /// The idempotency key is minted once, before the first attempt, and every
    /// attempt carries the same one — which is the only thing that makes
    /// repeating a `POST` that creates a subscription safe.
    pub(crate) async fn send<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        form: &[(String, String)],
    ) -> Result<T, Error> {
        let attempts = self.strategy.begin()?;
        let url = format!("{}{path}", self.base);

        for attempt in 0..attempts.count() {
            let mut request = self
                .http
                .request(method.clone(), &url)
                .basic_auth(&self.secret, None::<&str>)
                .header("Stripe-Version", API_VERSION);

            if let Some(key) = attempts.key() {
                request = request.header("Idempotency-Key", key.as_str());
            }

            if !form.is_empty() {
                request = request.form(form);
            }

            let outcome = match request.send().await {
                Ok(response) => self.read(response).await,
                Err(source) => Err(Error::Unreachable {
                    cause: source.to_string(),
                }),
            };

            let failure = match outcome {
                Ok(value) => return Ok(value),
                Err(failure) => failure,
            };

            // The last attempt, or an answer rather than a hiccup. A 402 is a
            // declined card: sending it again declines it again.
            if !failure.retryable() || attempt + 1 >= attempts.count() {
                return Err(failure);
            }

            tokio::time::sleep(attempts.backoff(attempt)).await;
        }

        // Unreachable: `count` is at least one and every path through the loop
        // either returns or continues. Written as an error rather than a
        // panic because this crate does not panic in a request path.
        Err(Error::Unreachable {
            cause: "the request was never attempted".to_owned(),
        })
    }

    async fn read<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, Error> {
        let status = response.status();
        let body =
            response
                .bytes()
                .await
                .map_err(|source| Error::Unreachable {
                    cause: source.to_string(),
                })?;

        if !status.is_success() {
            return Err(refusal(status, &body));
        }

        serde_json::from_slice(&body).map_err(|source| Error::Malformed {
            what: "a stripe response",
            cause: source.to_string(),
        })
    }
}

/// Turn a non-2xx into a refusal, keeping whatever Stripe said about it.
///
/// A body that is not Stripe's error envelope still becomes a refusal rather
/// than a parse error — something answered with a failing status, and that is
/// the fact worth reporting.
fn refusal(status: StatusCode, body: &[u8]) -> Error {
    let detail = serde_json::from_slice::<wire::Failure>(body)
        .ok()
        .map(|failure| failure.error);

    let (code, message) = detail.map_or_else(
        || {
            (
                "unreadable".to_owned(),
                String::from_utf8_lossy(body).chars().take(200).collect(),
            )
        },
        |detail| {
            (
                detail.code.unwrap_or(detail.kind),
                detail.message.unwrap_or_default(),
            )
        },
    );

    Error::Refused {
        status: status.as_u16(),
        code,
        message,
    }
}

/// One form field, as Stripe's bracketed nesting spells it.
pub(crate) fn field(
    name: impl Into<String>,
    value: impl Into<String>,
) -> (String, String) {
    (name.into(), value.into())
}
