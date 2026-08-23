//! The flow itself: build the authorize URL, then handle the callback.
//!
//! Everything provider-specific is a call into [`crate::github`] or
//! [`crate::google`]. What is left here is the half of OAuth that is the same
//! either way — the state check and the token exchange.

use std::fmt;

use serde::Deserialize;

use crate::{
    USER_AGENT,
    error::Error,
    github, google,
    provider::{Endpoints, Provider},
    state,
    user::SocialUser,
};

/// What the callback hands back, plus what the caller put in the session before
/// the redirect.
///
/// Both states are fields rather than something [`Client::user`] could skip, so
/// the CSRF check cannot be forgotten at a call site. Socialite pulls the
/// expected value out of the session itself; this crate has no session to pull
/// from, so it asks.
#[derive(Clone, Copy, Debug)]
pub struct Callback<'a> {
    /// `code` from the provider's query string.
    pub code: &'a str,
    /// `state` from the provider's query string.
    pub state: &'a str,
    /// The state issued at redirect time, pulled from the session.
    pub expected_state: &'a str,
}

/// Cloning shares the underlying connection pool, so a registry of these is
/// cheap to hand to every request.
#[derive(Clone)]
pub struct Client {
    provider: Provider,
    /// The provider's `client_id`, named for what it is here rather than for
    /// the wire field it becomes.
    id: String,
    secret: String,
    redirect_url: String,
    endpoints: Endpoints,
    http: reqwest::Client,
}

/// Hand written. The derived one would put the client secret into any log line
/// that formats the client with `{:?}`.
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("provider", &self.provider)
            .field("id", &self.id)
            .field("secret", &"<redacted>")
            .field("redirect_url", &self.redirect_url)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// `redirect_url` must be byte-identical to the one registered with the
    /// provider — both send it again at token exchange and compare.
    ///
    /// # Errors
    ///
    /// [`Error::Http`] if the TLS backend cannot be initialised, which is a
    /// build problem rather than a runtime one.
    pub fn new(
        provider: Provider,
        id: impl Into<String>,
        secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Result<Self, Error> {
        Self::with_endpoints(
            provider,
            id,
            secret,
            redirect_url,
            provider.endpoints(),
        )
    }

    /// Same as [`Client::new`] but with the hosts overridden.
    ///
    /// `pub(crate)` and used only by the tests, which point the flow at a local
    /// server so the requests it makes can be asserted rather than guessed at.
    /// Socialite does this with a subclass that overrides `getTokenUrl()`.
    pub(crate) fn with_endpoints(
        provider: Provider,
        id: impl Into<String>,
        secret: impl Into<String>,
        redirect_url: impl Into<String>,
        endpoints: Endpoints,
    ) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|source| Error::Http {
                provider: provider.as_str(),
                source,
            })?;

        Ok(Self {
            provider,
            id: id.into(),
            secret: secret.into(),
            redirect_url: redirect_url.into(),
            endpoints,
            http,
        })
    }

    #[must_use]
    pub fn provider(&self) -> Provider {
        self.provider
    }

    /// Base URL for the provider's profile reads, for [`crate::github`] and
    /// [`crate::google`] to build their paths on.
    pub(crate) fn api(&self) -> &str {
        &self.endpoints.api
    }

    /// The URL to redirect the browser to.
    ///
    /// `state` is the caller's to generate — [`crate::random_state`] — and to
    /// keep in the session until the callback.
    #[must_use]
    pub fn authorize_url(&self, state: &str) -> String {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.id)
            .append_pair("redirect_uri", &self.redirect_url)
            .append_pair("scope", self.provider.scopes())
            .append_pair("response_type", "code")
            .append_pair("state", state)
            .finish();

        format!("{}?{}", self.provider.authorize_endpoint(), query)
    }

    /// The callback half: verify, exchange, fetch, map.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidState`] when the two states disagree, before anything is
    /// sent anywhere. [`Error::Rejected`] when the provider refuses the code,
    /// and [`Error::Http`] when a request or its body fails.
    pub async fn user(
        &self,
        callback: Callback<'_>,
    ) -> Result<SocialUser, Error> {
        if !state::matches(callback.expected_state, callback.state) {
            return Err(Error::InvalidState);
        }

        let token = self.access_token(callback.code).await?;

        match self.provider {
            Provider::Github => github::user(self, &token).await,
            Provider::Google => google::user(self, &token).await,
        }
    }

    async fn access_token(&self, code: &str) -> Result<String, Error> {
        let response: TokenResponse = self
            .http
            .post(&self.endpoints.token)
            // GitHub answers form-encoded unless asked otherwise, and a
            // form-encoded body is not what the deserialiser below expects.
            .header(reqwest::header::ACCEPT, "application/json")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", self.id.as_str()),
                ("client_secret", self.secret.as_str()),
                ("code", code),
                ("redirect_uri", self.redirect_url.as_str()),
            ])
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|source| self.http_error(source))?
            .json()
            .await
            .map_err(|source| self.http_error(source))?;

        // GitHub reports a bad code as 200 with an `error` body, so a status
        // check alone is not enough to know the exchange worked.
        response.access_token.ok_or_else(|| Error::Rejected {
            provider: self.provider.as_str(),
            message: response
                .error_description
                .or(response.error)
                .unwrap_or_else(|| {
                    "no access token in the response".to_owned()
                }),
        })
    }

    /// An authenticated GET returning JSON. `scheme` is the `Authorization`
    /// prefix, which the two providers disagree about.
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        scheme: &str,
    ) -> Result<T, Error> {
        self.http
            .get(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::AUTHORIZATION, format!("{scheme} {token}"))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|source| self.http_error(source))?
            .json()
            .await
            .map_err(|source| self.http_error(source))
    }

    fn http_error(&self, source: reqwest::Error) -> Error {
        Error::Http {
            provider: self.provider.as_str(),
            source,
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string, header, method, path},
    };

    use super::*;
    use crate::testing::{CODE, STATE, TOKEN, client_for, expected_token_form};

    fn client(provider: Provider) -> Client {
        Client::new(
            provider,
            "client-id",
            "shh",
            "https://lighthouse.test/api/auth/callback",
        )
        .unwrap()
    }

    #[test]
    fn the_authorize_url_carries_every_field_the_provider_needs() {
        let url = client(Provider::Github).authorize_url("state-value");

        assert!(url.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(url.contains("client_id=client-id"));
        assert!(url.contains(
            "redirect_uri=https%3A%2F%2Flighthouse.test%2Fapi%2Fauth%2Fcallback"
        ));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=state-value"));
        // The secret is not part of the redirect, only of the exchange.
        assert!(!url.contains("shh"));
    }

    #[test]
    fn scopes_use_each_providers_separator() {
        assert!(
            client(Provider::Github)
                .authorize_url("s")
                .contains("scope=user%3Aemail")
        );
        // '+' is an encoded space, which is what the space separator becomes.
        assert!(
            client(Provider::Google)
                .authorize_url("s")
                .contains("scope=openid+profile+email")
        );
    }

    #[test]
    fn debug_does_not_print_the_secret() {
        assert!(!format!("{:?}", client(Provider::Github)).contains("shh"));
    }

    #[test]
    fn a_token_response_without_a_token_carries_the_reason() {
        let response: TokenResponse = serde_json::from_str(
            r#"{"error":"bad_verification_code","error_description":"The code passed is incorrect."}"#,
        )
        .unwrap();

        assert!(response.access_token.is_none());
        assert_eq!(
            response.error_description.as_deref(),
            Some("The code passed is incorrect.")
        );
    }

    /// Socialite: `testExceptionIsThrownIfStateIsInvalid`.
    #[tokio::test]
    async fn a_state_that_does_not_match_is_rejected_before_anything_is_sent() {
        let server = MockServer::start().await;
        let client = client_for(Provider::Github, &server);

        let error = client
            .user(Callback {
                code: CODE,
                state: "some-other-state",
                expected_state: STATE,
            })
            .await
            .unwrap_err();

        assert!(matches!(error, Error::InvalidState));
        // The point of checking first: an attacker's code is never redeemed.
        // No mocks are mounted, so any request at all would also have failed —
        // this asserts the stronger thing, that none was made.
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    /// Socialite: `testExceptionIsThrownIfStateIsNotSet` — the session had no
    /// state to pull, which must not be read as "no state required".
    #[tokio::test]
    async fn a_state_the_session_never_issued_is_rejected() {
        let server = MockServer::start().await;
        let client = client_for(Provider::Github, &server);

        let error = client
            .user(Callback {
                code: CODE,
                state: "",
                expected_state: "",
            })
            .await
            .unwrap_err();

        assert!(matches!(error, Error::InvalidState));
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    /// Socialite asserts the same `form_params` against a mocked Guzzle client.
    #[tokio::test]
    async fn the_exchange_posts_the_authorization_code_grant() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .and(header("accept", "application/json"))
            .and(body_string(expected_token_form()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": TOKEN,
                "token_type": "bearer",
                "scope": "user:email",
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1, "login": "octocat", "email": "o@x.test", "avatar_url": null, "name": null,
            })))
            .mount(&server)
            .await;

        let user = client_for(Provider::Github, &server)
            .user(Callback {
                code: CODE,
                state: STATE,
                expected_state: STATE,
            })
            .await
            .unwrap();

        assert_eq!(user.id, "1");
    }

    /// GitHub answers a bad code with `200` and an error object rather than a
    /// 4xx, so a status check alone would read this as a successful login.
    #[tokio::test]
    async fn an_error_body_behind_a_200_is_still_a_rejection() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "error": "bad_verification_code",
                "error_description": "The code passed is incorrect.",
            })))
            .mount(&server)
            .await;

        let error = client_for(Provider::Github, &server)
            .user(Callback {
                code: CODE,
                state: STATE,
                expected_state: STATE,
            })
            .await
            .unwrap_err();

        match error {
            Error::Rejected { provider, message } => {
                assert_eq!(provider, "github");
                assert_eq!(message, "The code passed is incorrect.");
            }
            other => panic!("expected a rejection, got {other:?}"),
        }
    }
}
