//! Scaffolding shared by the flow tests.
//!
//! The flow tests run the real code against a real local HTTP server and assert
//! what it sent. Socialite does the equivalent with a mocked Guzzle client and
//! an exact `form_params` expectation; a live server is the closest thing that
//! also proves the headers and the URL.

use wiremock::MockServer;

use crate::{
    client::Client,
    provider::{Endpoints, Provider},
};

pub(crate) const ID: &str = "client-id";
pub(crate) const SECRET: &str = "shh";
pub(crate) const REDIRECT: &str = "https://lighthouse.test/api/auth/callback";

/// Whatever the caller stashed in the session before redirecting.
pub(crate) const STATE: &str = "the-state";
pub(crate) const CODE: &str = "the-code";
pub(crate) const TOKEN: &str = "access-token";

/// A client whose token exchange and profile reads both point at `server`.
pub(crate) fn client_for(provider: Provider, server: &MockServer) -> Client {
    Client::with_endpoints(
        provider,
        ID.into(),
        SECRET.into(),
        REDIRECT,
        Endpoints {
            token: format!("{}/token", server.uri()),
            api: server.uri(),
        },
    )
    .unwrap()
}

/// The exact body an authorization-code exchange must carry.
///
/// Written out rather than rebuilt from the same code under test, so a change
/// to a field name or the grant type fails here instead of failing silently
/// against the provider.
pub(crate) fn expected_token_form() -> String {
    format!(
        "grant_type=authorization_code\
         &client_id={ID}\
         &client_secret={SECRET}\
         &code={CODE}\
         &redirect_uri=https%3A%2F%2Flighthouse.test%2Fapi%2Fauth%2Fcallback"
    )
}
