//! OAuth 2.0 sign-in for GitHub and Google.
//!
//! This is Socialite's job in Rust, cut down to the two providers the platform
//! actually uses.
//!
//! Credentials are registered once at boot, and a handler asks for a driver by
//! the name in its route parameter — the same shape as
//! `Socialite::driver($provider)`:
//!
//! ```ignore
//! // boot: after config, before the router
//! let socials = loginwith::providers([
//!     GithubProvider::with(&config.github_id, &config.github_secret, &config.github_redirect),
//!     GoogleProvider::with(&config.google_id, &config.google_secret, &config.google_redirect),
//! ])?;
//!
//! // GET /auth/login/{provider}
//! let driver = socials.driver(provider).ok_or_else(not_found)?;
//! let state = random_state()?;
//! session.put("oauth_state", &state);
//! redirect(driver.authorize_url(&state));
//!
//! // GET /auth/callback/{provider}
//! let user = socials.driver(provider).ok_or_else(not_found)?
//!     .user(Callback { code, state, expected_state })
//!     .await?;
//! ```
//!
//! The flow underneath is the one `Laravel\Socialite\Two` runs:
//!
//! 1. [`Client::authorize_url`] builds the provider's consent URL. The caller
//!    stores the `state` in the session and redirects the browser to it.
//! 2. The provider calls back with `code` and `state`.
//! 3. [`Client::user`] checks the state, trades the code for an access token,
//!    reads the profile, and returns a [`SocialUser`].
//!
//! What is deliberately not here, because nothing needs it yet: PKCE, refresh
//! tokens, Google ID-token (JWT) verification, and any provider beyond these
//! two. Socialite carries all of it; each one is dead weight until a caller
//! asks.
//!
//! The crate owns no session and no database. It takes a code and gives back a
//! profile — deciding who that profile *is* belongs to the caller.
//!
//! # Layout
//!
//! Every module is private and its public items are re-exported here, so the
//! paths callers write stay flat — `loginwith::Client`, not
//! `loginwith::client::Client` — and moving something between modules is not a
//! breaking change.
//!
//! - [`registry`] — registering the providers at boot and looking one up.
//! - [`provider`] — which two providers exist, and their endpoints and scopes.
//! - [`client`] — the flow: authorize URL, state check, token exchange.
//! - [`github`], [`google`] — one module per provider's wire format and the
//!   mapping from it onto [`SocialUser`].
//! - [`state`] — minting the CSRF state and comparing it.
//! - [`user`], [`error`] — the two types that cross the crate boundary.

mod client;
mod error;
mod github;
mod google;
mod provider;
mod registry;
mod state;
mod user;

#[cfg(test)]
mod testing;

pub use client::{Callback, Client};
pub use error::Error;
pub use provider::Provider;
pub use registry::{
    GithubProvider, GoogleProvider, Providers, Registration, providers,
};
pub use state::random_state;
pub use user::SocialUser;

/// GitHub refuses any API request without a `User-Agent` and there is no
/// default in reqwest, unlike Guzzle. Without this the profile fetch is a 403
/// and the cause is not obvious from the response.
const USER_AGENT: &str = concat!("loginwith/", env!("CARGO_PKG_VERSION"));
