//! Reading cookies off a request, and building the ones that go back.
//!
//! Hand rolled rather than a cookie crate: this site sets three, all of them
//! minted here, all with values restricted to characters that need no escaping.
//! The dependency would bring percent-decoding, jars and its own signing, none
//! of which is used.

use axum::http::{
    HeaderMap, HeaderValue,
    header::{COOKIE, SET_COOKIE},
};

/// A `Set-Cookie` value being built.
///
/// The defaults are the safe ones — `HttpOnly`, site-wide path, expires with
/// the browser session — so a cookie only becomes reachable from javascript or
/// long-lived by saying so. `secure` is not a default and not a field: it is
/// passed to [`Cookie::to_header`], because it comes from the origin rather
/// than from the cookie, and a builder method could be forgotten.
#[derive(Debug)]
pub(crate) struct Cookie<'a> {
    name: &'a str,
    value: &'a str,
    path: &'a str,
    max_age: i64,
    http_only: bool,
}

impl<'a> Cookie<'a> {
    pub(crate) const fn new(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            path: "/",
            max_age: 0,
            http_only: true,
        }
    }

    /// A cookie that deletes the one already in the browser.
    ///
    /// A browser matches a replacement on name *and* path, and on `HttpOnly`
    /// being consistent, so this has to be given the same `path` and
    /// [`script_readable`](Self::script_readable) the original was set with —
    /// otherwise the old cookie quietly survives.
    pub(crate) const fn expiring(name: &'a str) -> Self {
        Self::new(name, "")
    }

    pub(crate) const fn path(mut self, path: &'a str) -> Self {
        self.path = path;
        self
    }

    /// Seconds. Absent, the cookie dies with the browser session.
    pub(crate) const fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = seconds;
        self
    }

    /// Drops `HttpOnly`, so a script on this origin can read it.
    ///
    /// Only for values that grant nothing. Anything worth stealing keeps the
    /// default.
    pub(crate) const fn script_readable(mut self) -> Self {
        self.http_only = false;
        self
    }

    /// `secure` comes from the origin — see `Config::cookie_secure`. It is off
    /// over plain http because a `Secure` cookie is dropped outright there,
    /// which makes local development look broken rather than misconfigured.
    pub(crate) fn to_header(&self, secure: bool) -> String {
        // `SameSite=Lax`, not `Strict`. Strict would withhold the oauth state
        // cookie on the callback — a top-level navigation *from the provider's
        // origin*, which is exactly what Strict suppresses — and every sign-in
        // would fail the state check.
        let secure = if secure { "; Secure" } else { "" };
        let http_only = if self.http_only { "; HttpOnly" } else { "" };
        let Self {
            name,
            value,
            path,
            max_age,
            ..
        } = self;

        format!(
            "{name}={value}; Path={path}; Max-Age={max_age}\
             {http_only}; SameSite=Lax{secure}"
        )
    }
}

/// One cookie's value out of a request, or `None`.
pub(crate) fn read<'h>(headers: &'h HeaderMap, name: &str) -> Option<&'h str> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        // Exact match on the name: `lh_session_backup` must not answer for
        // `lh_session`.
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

/// Puts built cookies on a response.
pub(crate) fn attach(headers: &mut HeaderMap, cookies: &[String]) {
    for value in cookies {
        // `append`, not `insert`: two Set-Cookie headers set two cookies, one
        // insert would drop the first.
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.append(SET_COOKIE, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_the_safe_ones() {
        let header = Cookie::new("lh_session", "v").to_header(true);

        assert!(header.contains("HttpOnly"), "xss could read the session");
        assert!(header.contains("SameSite=Lax"));
        assert!(header.contains("Secure"));
        assert!(header.contains("Path=/"));
    }

    #[test]
    fn plain_http_omits_secure_so_local_development_can_sign_in() {
        // A Secure cookie is dropped outright over http://localhost, which
        // would make the whole flow look broken rather than misconfigured.
        assert!(
            !Cookie::new("lh_session", "v")
                .to_header(false)
                .contains("Secure")
        );
    }

    #[test]
    fn a_script_readable_cookie_says_so_and_nothing_else_changes() {
        let header = Cookie::new("lh_reader", "1")
            .script_readable()
            .to_header(true);

        assert!(!header.contains("HttpOnly"));
        assert!(header.contains("SameSite=Lax"));
        assert!(header.contains("Secure"));
    }

    #[test]
    fn an_expiring_cookie_keeps_the_path_it_was_set_with() {
        let header = Cookie::expiring("lh_oauth").path("/").to_header(true);

        assert!(header.starts_with("lh_oauth=;"));
        assert!(header.contains("Max-Age=0"));
        assert!(header.contains("Path=/"));
    }

    fn with_cookie(header: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(header).unwrap());
        headers
    }

    #[test]
    fn one_cookie_is_found_among_many() {
        let headers = with_cookie("theme=dark; lh_session=abc.def; locale=bn");

        assert_eq!(read(&headers, "lh_session"), Some("abc.def"));
        assert_eq!(read(&headers, "theme"), Some("dark"));
        assert_eq!(read(&headers, "absent"), None);
    }

    #[test]
    fn a_name_that_merely_starts_the_same_does_not_answer() {
        let headers = with_cookie("lh_session_old=stale; lh_sessionx=no");

        assert_eq!(read(&headers, "lh_session"), None);
    }

    #[test]
    fn a_cookie_split_across_headers_is_still_read() {
        let mut headers = with_cookie("theme=dark");
        headers.append(COOKIE, HeaderValue::from_static("lh_session=abc.def"));

        assert_eq!(read(&headers, "lh_session"), Some("abc.def"));
    }

    #[test]
    fn two_cookies_become_two_headers() {
        let mut headers = HeaderMap::new();
        attach(
            &mut headers,
            &[
                Cookie::new("a", "1").to_header(true),
                Cookie::new("b", "2").to_header(true),
            ],
        );

        assert_eq!(headers.get_all(SET_COOKIE).iter().count(), 2);
    }
}
