//! Everything the process reads from its environment, read once at startup.
//!
//! One place, one time, all of it required. A key that is missing fails the
//! boot and names itself, rather than surfacing as a 404 on the one route that
//! needed it. There are no defaults on purpose: a value that quietly falls back
//! is a value nobody notices is wrong until it is wrong in production.
//!
//! `RUST_LOG` and `LOG_FORMAT` are deliberately not here — `telemetry::init`
//! runs before this does, because a config failure has to be loggable.

use secrecy::SecretString;

use crate::limit::RateLimit;

/// Every credential is a [`SecretString`], so `Debug` is derived rather than
/// hand-written: they print as `[REDACTED]` and their memory is zeroed on
/// drop. Redaction is a property of the field's type, which means a key added
/// later is covered without anybody remembering to add a line to an impl.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    /// Shared with luxctl. Required, so a deployment cannot come up with the
    /// signed routes reachable but unverified.
    pub(crate) luxctl_secret: SecretString,
    /// Loopback port the api binds.
    pub(crate) api_port: u16,
    /// Postgres connection string. The rebuild runs against the schema the
    /// laravel app already owns, so during the crossover this points at the
    /// same database that stack is using.
    pub(crate) database_url: SecretString,
    /// The site's own origin, e.g. `https://projectlighthouse.io`. Two things
    /// derive from it rather than being configured separately and drifting: the
    /// OAuth callback URLs, and whether cookies are marked `Secure`.
    pub(crate) app_url: String,
    /// Where the content repo is checked out. The prose is private and lives
    /// outside this repo entirely — see `content`.
    pub(crate) content_path: String,
    /// Whether unfinished lessons load at all. Hidden unless `SHOW_DRAFTS` is
    /// exactly `true` — see [`ohara::Drafts`] for why this one key is allowed
    /// to be absent when every other is required.
    pub(crate) drafts: ohara::Drafts,
    /// Stripe's api key. Required: a deployment that came up with the billing
    /// routes mounted but unusable would only find out from a reader who tried
    /// to pay.
    pub(crate) stripe_secret_key: SecretString,
    /// The signing secret for *this deployment's* webhook endpoint — the
    /// `whsec_…` shown when the endpoint is created. Not the api key, and
    /// different per endpoint, so staging and production do not share one.
    pub(crate) stripe_webhook_secret: SecretString,
    /// Where the plans on sale are described. Gitignored, and injected at
    /// deploy time: which plans exist and what they map to at the provider is
    /// a deployment's business rather than this repository's.
    pub(crate) billing_plans: String,
    /// Requests a minute, per caller, across every route.
    ///
    /// Defaulted rather than required, unlike every secret here: a limit that
    /// is absent has an obvious safe answer, and a deployment that forgot one
    /// should still come up limited rather than not come up at all.
    pub(crate) request_limit: u32,
    /// Note and bookmark writes a minute, per reader.
    pub(crate) note_write_limit: u32,
    /// Content reloads a minute, for the whole process.
    pub(crate) content_reload_limit: u32,
    pub(crate) github_id: SecretString,
    pub(crate) github_secret: SecretString,
    pub(crate) google_id: SecretString,
    pub(crate) google_secret: SecretString,
}

/// A limit, or the default when it is not set.
///
/// A value that is present but not a number is an error rather than a silent
/// fall back to the default: somebody meant to set it, and a typo that reads
/// as "unset" is the kind that is only noticed under load.
///
/// Zero is refused. It reads as "no limit" and means the opposite — every
/// request refused — which is a way to take a site down with a config edit.
fn count(value: Option<&str>, fallback: u32) -> Result<u32, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty())
    else {
        return Ok(fallback);
    };

    match value.parse::<u32>() {
        Ok(0) | Err(_) => Err(format!(
            "a rate limit must be a positive number of requests a minute, \
             and this one is {value:?}"
        )),
        Ok(limit) => Ok(limit),
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
            get(key).ok_or_else(|| {
                format!("{key} is not set in .env or the environment")
            })
        };

        // Empty is a value: `LUXCTL_SECRET=` is a deliberate choice and is kept.
        // A port is the exception — there is no empty port.
        let api_port = required("API_PORT")?;
        let api_port = api_port.parse().map_err(|_| {
            format!("API_PORT is not a port number: {api_port:?}")
        })?;

        Ok(Self {
            luxctl_secret: required("LUXCTL_SECRET")?.into(),
            api_port,
            database_url: required("DATABASE_URL")?.into(),
            // Trailing slash trimmed once, here, so every caller can join a path
            // onto it without producing `//github/callback` — which a provider
            // compares byte for byte against its registered callback and refuses.
            app_url: required("APP_URL")?.trim_end_matches('/').to_owned(),
            content_path: required("CONTENT_PATH")?,
            // Not `required`: unset means hidden, which is the safe answer.
            drafts: ohara::Drafts::from_env(get("SHOW_DRAFTS").as_deref()),
            request_limit: count(
                get("REQUEST_LIMIT").as_deref(),
                RateLimit::REQUESTS,
            )?,
            note_write_limit: count(
                get("NOTE_WRITE_LIMIT").as_deref(),
                RateLimit::NOTE_WRITES,
            )?,
            content_reload_limit: count(
                get("CONTENT_RELOAD_LIMIT").as_deref(),
                RateLimit::CONTENT_RELOADS,
            )?,
            stripe_secret_key: required("STRIPE_SECRET_KEY")?.into(),
            stripe_webhook_secret: required("STRIPE_WEBHOOK_SECRET")?.into(),
            billing_plans: required("BILLING_PLANS")?,
            github_id: required("GITHUB_CLIENT_ID")?.into(),
            github_secret: required("GITHUB_CLIENT_SECRET")?.into(),
            google_id: required("GOOGLE_CLIENT_ID")?.into(),
            google_secret: required("GOOGLE_CLIENT_SECRET")?.into(),
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
            luxctl_secret: "luxctl".into(),
            api_port: 9000,
            database_url: "postgres://localhost/lighthouse".into(),
            app_url: "https://lighthouse.test".to_owned(),
            content_path: "../ohara".to_owned(),
            drafts: ohara::Drafts::Hidden,
            request_limit: RateLimit::REQUESTS,
            note_write_limit: RateLimit::NOTE_WRITES,
            content_reload_limit: RateLimit::CONTENT_RELOADS,
            stripe_secret_key: "sk_test".into(),
            stripe_webhook_secret: "whsec_test".into(),
            billing_plans: "billing.sample.yaml".to_owned(),
            github_id: "gh-id".into(),
            github_secret: "gh-secret".into(),
            google_id: "goo-id".into(),
            google_secret: "goo-secret".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret as _;

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
        ("CONTENT_PATH", "../ohara"),
        ("STRIPE_SECRET_KEY", "sk_test"),
        ("STRIPE_WEBHOOK_SECRET", "whsec_test"),
        ("BILLING_PLANS", "billing.sample.yaml"),
        ("GITHUB_CLIENT_ID", "gh-id"),
        ("GITHUB_CLIENT_SECRET", "gh-secret"),
        ("GOOGLE_CLIENT_ID", "goo-id"),
        ("GOOGLE_CLIENT_SECRET", "goo-secret"),
    ];

    fn vars(
        pairs: Vec<(&'static str, String)>,
    ) -> impl Fn(&str) -> Option<String> {
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

    /// `COMPLETE` with one key overridden, or added when it is not required.
    ///
    /// The optional keys — the rate limits — are absent from `COMPLETE` on
    /// purpose, so setting one has to add it rather than silently do nothing
    /// and leave the test asserting against the default.
    fn with(key: &'static str, value: &str) -> Vec<(&'static str, String)> {
        let mut pairs = complete();

        match pairs.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 = value.to_owned(),
            None => pairs.push((key, value.to_owned())),
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
        let config =
            Config::from_vars(vars(with("LUXCTL_SECRET", ""))).unwrap();

        assert_eq!(config.luxctl_secret.expose_secret(), "");
        assert_eq!(config.api_port, 9000);
    }

    #[test]
    fn a_rate_limit_falls_back_rather_than_failing_the_boot() {
        // Unlike every secret here. A limit that is merely absent has an
        // obvious safe answer, and a deployment that forgot one should come up
        // limited rather than not come up.
        let config = Config::from_vars(vars(complete())).unwrap();

        assert_eq!(config.request_limit, RateLimit::REQUESTS);
        assert_eq!(config.note_write_limit, RateLimit::NOTE_WRITES);
        assert_eq!(config.content_reload_limit, RateLimit::CONTENT_RELOADS);
    }

    #[test]
    fn a_rate_limit_that_is_set_is_used() {
        let config =
            Config::from_vars(vars(with("REQUEST_LIMIT", "100000"))).unwrap();

        assert_eq!(config.request_limit, 100_000);
    }

    #[test]
    fn a_rate_limit_that_is_not_a_number_is_refused() {
        // A typo that read as "unset" would fall back to sixty and only be
        // noticed under load, which is the worst time to find out.
        for bad in ["nonsense", "-1", "12.5"] {
            assert!(
                Config::from_vars(vars(with("REQUEST_LIMIT", bad))).is_err(),
                "{bad} should not parse as a limit"
            );
        }
    }

    #[test]
    fn a_rate_limit_of_zero_is_refused() {
        // It reads as "no limit" and means the opposite: every request
        // refused. A way to take the site down with a config edit.
        assert!(Config::from_vars(vars(with("REQUEST_LIMIT", "0"))).is_err());
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
        let config = Config::from_vars(vars(with(
            "APP_URL",
            "https://lighthouse.test/",
        )))
        .unwrap();

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

    /// Every key whose value must never reach a log line.
    ///
    /// Not just the ones with `SECRET` in the name: a connection string
    /// carries a password, and a client id is an identifier of a private
    /// application even though OAuth puts it in a url. Each is a
    /// `SecretString` on `Config`, and this is what proves it — adding a
    /// credential as a plain `String` fails here.
    const CREDENTIALS: &[&str] = &[
        "LUXCTL_SECRET",
        "DATABASE_URL",
        "STRIPE_SECRET_KEY",
        "STRIPE_WEBHOOK_SECRET",
        "GITHUB_CLIENT_ID",
        "GITHUB_CLIENT_SECRET",
        "GOOGLE_CLIENT_ID",
        "GOOGLE_CLIENT_SECRET",
    ];

    #[test]
    fn debug_prints_no_credential_at_all() {
        let mut pairs = complete();
        for (key, value) in &mut pairs {
            if CREDENTIALS.contains(key) {
                *value = format!("hunter2-{key}");
            }
        }

        let dumped = format!("{:?}", Config::from_vars(vars(pairs)).unwrap());

        assert!(
            !dumped.contains("hunter2"),
            "a credential reached a log line: {dumped}"
        );
    }

    #[test]
    fn every_credential_is_covered_by_that_test() {
        // The list above is hand-maintained, and a key added to `COMPLETE`
        // without being classified would silently escape the check. Anything
        // that is not a credential has to be named here on purpose.
        const PUBLIC: &[&str] =
            &["API_PORT", "APP_URL", "CONTENT_PATH", "BILLING_PLANS"];

        for (key, _) in COMPLETE {
            assert!(
                CREDENTIALS.contains(key) || PUBLIC.contains(key),
                "{key} is neither a credential nor declared public"
            );
        }
    }

    #[test]
    fn debug_still_says_something_useful() {
        // Redaction that swallowed the whole struct would be a config dump
        // nobody can debug with.
        let dumped =
            format!("{:?}", Config::from_vars(vars(complete())).unwrap());

        assert!(dumped.contains("https://lighthouse.test"), "{dumped}");
        assert!(dumped.contains("9000"), "{dumped}");
    }
}
