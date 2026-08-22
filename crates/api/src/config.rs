//! Everything the process reads from its environment, read once at startup.
//!
//! One place, one time, all of it required. A key that is missing fails the
//! boot and names itself, rather than surfacing as a 404 on the one route that
//! needed it. There are no defaults on purpose: a value that quietly falls back
//! is a value nobody notices is wrong until it is wrong in production.
//!
//! `RUST_LOG` and `LOG_FORMAT` are deliberately not here — `telemetry::init`
//! runs before this does, because a config failure has to be loggable.

use std::fmt;

#[derive(Clone)]
pub(crate) struct Config {
    /// Shared with luxctl. Required, so a deployment cannot come up with the
    /// signed routes reachable but unverified.
    pub(crate) luxctl_secret: String,
    /// Loopback port the api binds.
    pub(crate) api_port: u16,
    /// Postgres connection string. The rebuild runs against the schema the
    /// laravel app already owns, so during the crossover this points at the
    /// same database that stack is using.
    pub(crate) database_url: String,
    /// The site's own origin, e.g. `https://projectlighthouse.io`. Two things
    /// derive from it rather than being configured separately and drifting: the
    /// OAuth callback URLs, and whether cookies are marked `Secure`.
    pub(crate) app_url: String,
    /// Signs the session cookie. Rotating it signs everyone out, which is the
    /// only revocation a stateless session has — see `session`.
    pub(crate) session_secret: String,
    pub(crate) github_id: String,
    pub(crate) github_secret: String,
    pub(crate) google_id: String,
    pub(crate) google_secret: String,
}

/// Hand written. The derived one would put every secret here into any log line
/// that formats the config with `{:?}`.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("luxctl_secret", &"<redacted>")
            .field("api_port", &self.api_port)
            // Redacted: a connection string carries the password.
            .field("database_url", &"<redacted>")
            .field("app_url", &self.app_url)
            .field("session_secret", &"<redacted>")
            .field("github_id", &self.github_id)
            .field("github_secret", &"<redacted>")
            .field("google_id", &self.google_id)
            .field("google_secret", &"<redacted>")
            .finish()
    }
}

impl Config {
    /// Loads `.env` if there is one, then reads the process environment.
    ///
    /// **No `.env` is not an error.** The runtime image ships without one and
    /// every value arrives as a real environment variable from the platform;
    /// treating the file as required means the container cannot boot at all.
    /// A file that exists but cannot be read or parsed *is* fatal — that is a
    /// deployment saying one thing and doing another.
    ///
    /// `dotenvy` does not overwrite variables that are already set, so a real
    /// environment variable still wins over the file. Either way the keys below
    /// are still required: the check that a value is present has not moved,
    /// only the insistence on where it came from.
    pub(crate) fn load() -> Result<Self, String> {
        if let Err(error) = dotenvy::dotenv()
            && !error.not_found()
        {
            return Err(format!("cannot read .env: {error}"));
        }

        Self::from_vars(|key| std::env::var(key).ok())
    }

    /// Split from `load` so the rules below are testable without touching the
    /// process environment, which `unsafe_code = "forbid"` puts out of reach
    /// anyway on edition 2024.
    fn from_vars(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        // Names the key and both places it could have come from: in production
        // there is no .env, and "not set in .env" sends whoever is reading a
        // failed deploy looking for a file that was never there.
        let required = |key: &str| {
            get(key).ok_or_else(|| format!("{key} is not set in .env or the environment"))
        };

        // Empty is a value: `LUXCTL_SECRET=` is a deliberate choice and is kept.
        // A port is the exception — there is no empty port.
        let api_port = required("API_PORT")?;
        let api_port = api_port
            .parse()
            .map_err(|_| format!("API_PORT is not a port number: {api_port:?}"))?;

        Ok(Self {
            luxctl_secret: required("LUXCTL_SECRET")?,
            api_port,
            database_url: required("DATABASE_URL")?,
            // Trailing slash trimmed once, here, so every caller can join a path
            // onto it without producing `//github/callback` — which a provider
            // compares byte for byte against its registered callback and refuses.
            app_url: required("APP_URL")?.trim_end_matches('/').to_owned(),
            session_secret: required("SESSION_SECRET")?,
            github_id: required("GITHUB_CLIENT_ID")?,
            github_secret: required("GITHUB_CLIENT_SECRET")?,
            google_id: required("GOOGLE_CLIENT_ID")?,
            google_secret: required("GOOGLE_CLIENT_SECRET")?,
        })
    }

    /// Whether cookies may be marked `Secure`.
    ///
    /// Derived rather than configured: a boolean someone can set wrong is a
    /// production site handing out cookies that any network can read. The origin
    /// already says which it is.
    pub(crate) fn cookie_secure(&self) -> bool {
        self.app_url.starts_with("https://")
    }

    /// Where a provider sends the browser back. Must match the callback
    /// registered with that provider byte for byte.
    ///
    /// The path is the laravel app's (`config/services.php`), kept deliberately:
    /// it is already registered with Google and GitHub, so the rebuild inherits
    /// those registrations instead of needing a second set. Changing it means
    /// editing every OAuth console before this can ship.
    pub(crate) fn callback_url(&self, provider: &str) -> String {
        format!("{}/{provider}/callback", self.app_url)
    }

    /// A complete config, for tests that need one to build a router with.
    #[cfg(test)]
    pub(crate) fn sample() -> Self {
        Self {
            luxctl_secret: "luxctl".to_owned(),
            api_port: 9000,
            database_url: "postgres://localhost/lighthouse".to_owned(),
            app_url: "https://lighthouse.test".to_owned(),
            session_secret: "session".to_owned(),
            github_id: "gh-id".to_owned(),
            github_secret: "gh-secret".to_owned(),
            google_id: "goo-id".to_owned(),
            google_secret: "goo-secret".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key set to something valid. Tests override or drop one at a time,
    /// so a new required key does not mean editing every case below.
    const COMPLETE: &[(&str, &str)] = &[
        ("LUXCTL_SECRET", "luxctl"),
        ("API_PORT", "9000"),
        (
            "DATABASE_URL",
            "postgres://postgres:password@localhost/projectlighthouse",
        ),
        ("APP_URL", "https://lighthouse.test"),
        ("SESSION_SECRET", "session"),
        ("GITHUB_CLIENT_ID", "gh-id"),
        ("GITHUB_CLIENT_SECRET", "gh-secret"),
        ("GOOGLE_CLIENT_ID", "goo-id"),
        ("GOOGLE_CLIENT_SECRET", "goo-secret"),
    ];

    fn vars(pairs: Vec<(&'static str, String)>) -> impl Fn(&str) -> Option<String> {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        }
    }

    fn complete() -> Vec<(&'static str, String)> {
        COMPLETE
            .iter()
            .map(|(k, v)| (*k, (*v).to_owned()))
            .collect()
    }

    fn with(key: &str, value: &str) -> Vec<(&'static str, String)> {
        let mut pairs = complete();
        if let Some(entry) = pairs.iter_mut().find(|(k, _)| *k == key) {
            entry.1 = value.to_owned();
        }
        pairs
    }

    fn without(key: &str) -> Vec<(&'static str, String)> {
        let mut pairs = complete();
        pairs.retain(|(k, _)| *k != key);
        pairs
    }

    #[test]
    fn every_key_is_required() {
        assert!(Config::from_vars(vars(vec![])).is_err());

        for (key, _) in COMPLETE {
            assert!(
                Config::from_vars(vars(without(key))).is_err(),
                "{key} was not required"
            );
        }

        assert!(Config::from_vars(vars(complete())).is_ok());
    }

    #[test]
    fn an_empty_value_is_still_a_value() {
        let config = Config::from_vars(vars(with("LUXCTL_SECRET", ""))).unwrap();

        assert_eq!(config.luxctl_secret, "");
        assert_eq!(config.api_port, 9000);
    }

    #[test]
    fn a_port_that_is_not_a_port_is_an_error() {
        for bad in ["", "http", "70000"] {
            assert!(Config::from_vars(vars(with("API_PORT", bad))).is_err());
        }
    }

    #[test]
    fn a_callback_url_has_exactly_one_slash_before_the_path() {
        // A trailing slash in APP_URL would otherwise produce `//api/auth/...`,
        // which no longer matches what is registered with the provider.
        let config = Config::from_vars(vars(with("APP_URL", "https://lighthouse.test/"))).unwrap();

        assert_eq!(
            config.callback_url("github"),
            "https://lighthouse.test/github/callback"
        );
    }

    #[test]
    fn cookies_are_secure_only_over_https() {
        assert!(
            Config::from_vars(vars(with("APP_URL", "https://lighthouse.test")))
                .unwrap()
                .cookie_secure()
        );
        assert!(
            !Config::from_vars(vars(with("APP_URL", "http://localhost:8080")))
                .unwrap()
                .cookie_secure()
        );
    }

    #[test]
    fn debug_prints_no_secret_at_all() {
        let mut pairs = complete();
        for (key, value) in &mut pairs {
            if key.contains("SECRET") {
                *value = format!("hunter2-{key}");
            }
        }

        let dumped = format!("{:?}", Config::from_vars(vars(pairs)).unwrap());

        assert!(!dumped.contains("hunter2"), "a secret reached a log line");
    }
}
