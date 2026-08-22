//! Registering the providers once at boot, and looking one up per request.
//!
//! This is Socialite's manager. Credentials are read from config in one place
//! during startup, and a handler asks for a driver by the name in its route
//! parameter — it never sees a client id, and a missing provider is decided
//! once here rather than in every endpoint.
//!
//! ```ignore
//! // boot, after config and before the router
//! let socials = loginwith::providers([
//!     GithubProvider::with(&config.github_id, &config.github_secret, &config.github_redirect),
//!     GoogleProvider::with(&config.google_id, &config.google_secret, &config.google_redirect),
//! ])?;
//!
//! // GET /auth/login/{provider}
//! let driver = socials.driver(provider).ok_or_else(not_found)?;
//! ```

use std::fmt;

use crate::{client::Client, error::Error, provider::Provider};

/// One provider's credentials, waiting to be built.
///
/// Returned by [`GithubProvider::with`] and [`GoogleProvider::with`], and only
/// useful as an argument to [`providers`].
#[derive(Clone)]
pub struct Registration {
    provider: Provider,
    id: String,
    secret: String,
    redirect_url: String,
}

/// Hand written, so a config dump cannot print the secret.
impl fmt::Debug for Registration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registration")
            .field("provider", &self.provider)
            .field("id", &self.id)
            .field("secret", &"<redacted>")
            .field("redirect_url", &self.redirect_url)
            .finish()
    }
}

impl Registration {
    fn new(
        provider: Provider,
        id: impl Into<String>,
        secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            id: id.into(),
            secret: secret.into(),
            redirect_url: redirect_url.into(),
        }
    }
}

/// GitHub's half of the registration list.
#[derive(Clone, Copy, Debug)]
pub struct GithubProvider;

impl GithubProvider {
    /// `redirect_url` must be byte-identical to the callback registered on the
    /// GitHub OAuth app — both send it again at the token exchange and compare.
    #[must_use]
    pub fn with(
        id: impl Into<String>,
        secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Registration {
        Registration::new(Provider::Github, id, secret, redirect_url)
    }
}

/// Google's half of the registration list.
#[derive(Clone, Copy, Debug)]
pub struct GoogleProvider;

impl GoogleProvider {
    /// `redirect_url` must match an authorised redirect URI on the Google
    /// OAuth client exactly.
    #[must_use]
    pub fn with(
        id: impl Into<String>,
        secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Registration {
        Registration::new(Provider::Google, id, secret, redirect_url)
    }
}

/// Every provider this process can sign a reader in with.
///
/// Cheap to clone — the clients share one connection pool — so it can go
/// straight into request state.
#[derive(Clone, Debug)]
pub struct Providers {
    drivers: Vec<Client>,
}

/// Build the registry. Call once, at boot.
///
/// Doing it here rather than per request means a bad credential is a startup
/// failure with a name attached, not a 500 the first time somebody tries to log
/// in with that provider.
///
/// # Errors
///
/// [`Error::Duplicate`] if a provider is registered twice — the second
/// registration would otherwise be silently ignored and read, much later, as
/// wrong credentials. [`Error::Http`] if a client cannot be built.
pub fn providers(list: impl IntoIterator<Item = Registration>) -> Result<Providers, Error> {
    let mut drivers: Vec<Client> = Vec::new();

    for registration in list {
        if drivers
            .iter()
            .any(|driver| driver.provider() == registration.provider)
        {
            return Err(Error::Duplicate {
                provider: registration.provider.as_str(),
            });
        }

        drivers.push(Client::new(
            registration.provider,
            registration.id,
            registration.secret,
            registration.redirect_url,
        )?);
    }

    Ok(Providers { drivers })
}

impl Providers {
    /// Socialite's `Socialite::driver($provider)`.
    ///
    /// `None` covers both "no such provider" and "that provider was never
    /// registered", because a handler owes the same answer to each: a 404. A
    /// 403 or a 500 would confirm which of the two it was.
    #[must_use]
    pub fn driver(&self, name: &str) -> Option<&Client> {
        self.get(Provider::parse(name)?)
    }

    /// The same lookup once the provider is already known, so a caller holding
    /// a [`Provider`] does not have to round-trip through its name.
    #[must_use]
    pub fn get(&self, provider: Provider) -> Option<&Client> {
        self.drivers
            .iter()
            .find(|driver| driver.provider() == provider)
    }

    /// Which providers are registered, for a login page that should only offer
    /// the buttons that will work.
    pub fn registered(&self) -> impl Iterator<Item = Provider> + '_ {
        self.drivers.iter().map(Client::provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn both() -> Providers {
        providers([
            GithubProvider::with("gh-id", "gh-secret", "https://l.test/auth/github/callback"),
            GoogleProvider::with(
                "goo-id",
                "goo-secret",
                "https://l.test/auth/google/callback",
            ),
        ])
        .unwrap()
    }

    #[test]
    fn a_driver_is_found_by_its_route_parameter() {
        let socials = both();

        assert_eq!(
            socials.driver("github").map(Client::provider),
            Some(Provider::Github)
        );
        assert_eq!(
            socials.driver("google").map(Client::provider),
            Some(Provider::Google)
        );
    }

    #[test]
    fn each_driver_carries_its_own_credentials() {
        let socials = both();
        let url = socials.driver("github").unwrap().authorize_url("s");

        assert!(url.contains("client_id=gh-id"));
        // The other provider's id must not leak across.
        assert!(!url.contains("goo-id"));
    }

    #[test]
    fn an_unknown_name_has_no_driver() {
        assert!(both().driver("facebook").is_none());
        assert!(both().driver("").is_none());
    }

    #[test]
    fn a_provider_that_was_never_registered_has_no_driver() {
        // The name is valid, the credentials were never supplied. Same 404 as
        // an unknown provider, rather than a panic on a missing key.
        let only_github =
            providers([GithubProvider::with("id", "secret", "https://l.test/cb")]).unwrap();

        assert!(only_github.driver("github").is_some());
        assert!(only_github.driver("google").is_none());
    }

    #[test]
    fn registering_a_provider_twice_is_a_boot_failure() {
        let error = providers([
            GithubProvider::with("first", "secret", "https://l.test/cb"),
            GithubProvider::with("second", "secret", "https://l.test/cb"),
        ])
        .unwrap_err();

        assert!(matches!(error, Error::Duplicate { provider: "github" }));
    }

    #[test]
    fn an_empty_registry_is_allowed_and_answers_nothing() {
        let none = providers([]).unwrap();

        assert!(none.driver("github").is_none());
        assert_eq!(none.registered().count(), 0);
    }

    #[test]
    fn registered_lists_what_was_supplied() {
        assert_eq!(
            both().registered().collect::<Vec<_>>(),
            vec![Provider::Github, Provider::Google]
        );
    }

    #[test]
    fn debug_does_not_print_a_secret() {
        let registration = GithubProvider::with("id", "hunter2", "https://l.test/cb");

        assert!(!format!("{registration:?}").contains("hunter2"));
        assert!(!format!("{:?}", both()).contains("gh-secret"));
    }
}
