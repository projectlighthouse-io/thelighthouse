//! The two providers, and everything that differs between them.
//!
//! Socialite models this as a class per provider over an abstract base. Two
//! providers do not need inheritance — an enum with a match arm per endpoint
//! says the same thing, and the compiler catches a third variant that forgets
//! one of them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Github,
    Google,
}

/// The two hosts a provider is talked to on after the browser comes back.
#[derive(Clone, Debug)]
pub(crate) struct Endpoints {
    pub(crate) token: String,
    /// Base for the profile reads. Google serves those from a different host
    /// than its token endpoint, so it cannot be derived from `token`.
    pub(crate) api: String,
}

impl Provider {
    /// The route-parameter form. `None` is a 404 at the edge, which is what the
    /// old `validateProvider` did — an unknown provider is not a bad request,
    /// it is a URL that does not exist.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "github" => Some(Self::Github),
            "google" => Some(Self::Google),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Google => "google",
        }
    }

    pub(crate) fn authorize_endpoint(self) -> &'static str {
        match self {
            Self::Github => "https://github.com/login/oauth/authorize",
            Self::Google => "https://accounts.google.com/o/oauth2/auth",
        }
    }

    /// Where the token exchange and the profile reads go.
    ///
    /// Owned strings rather than constants so a test can point the flow at a
    /// local server. Socialite gets the same seam from test provider stubs that
    /// override `getTokenUrl()`; this is that, without the inheritance.
    pub(crate) fn endpoints(self) -> Endpoints {
        match self {
            Self::Github => Endpoints {
                token: "https://github.com/login/oauth/access_token".to_owned(),
                api: "https://api.github.com".to_owned(),
            },
            Self::Google => Endpoints {
                // Google's current token endpoint. Socialite still points at
                // the older `www.googleapis.com/oauth2/v4/token` alias, which
                // resolves to the same service.
                token: "https://oauth2.googleapis.com/token".to_owned(),
                api: "https://www.googleapis.com".to_owned(),
            },
        }
    }

    /// Pre-joined, because the separator is part of the provider: GitHub splits
    /// scopes on a comma and Google on a space. Socialite models that as a
    /// `$scopeSeparator` field; with two providers a literal is the whole of it.
    pub(crate) fn scopes(self) -> &'static str {
        match self {
            // Without `user:email` the profile comes back with a null email
            // whenever the reader has kept theirs private.
            Self::Github => "user:email",
            Self::Google => "openid profile email",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_provider_does_not_parse() {
        assert_eq!(Provider::parse("github"), Some(Provider::Github));
        assert_eq!(Provider::parse("google"), Some(Provider::Google));
        assert_eq!(Provider::parse("facebook"), None);
        // Case sensitive: the route parameter is lowercase and anything else is
        // a URL that was not linked from this site.
        assert_eq!(Provider::parse("GitHub"), None);
    }

    #[test]
    fn every_endpoint_is_https() {
        for provider in [Provider::Github, Provider::Google] {
            let endpoints = provider.endpoints();

            assert!(provider.authorize_endpoint().starts_with("https://"));
            assert!(endpoints.token.starts_with("https://"));
            assert!(endpoints.api.starts_with("https://"));
        }
    }
}
