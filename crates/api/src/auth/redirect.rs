//! Sending a browser somewhere, and refusing to send it off-site.

use axum::{
    http::{HeaderValue, StatusCode, header::LOCATION},
    response::{IntoResponse, Response},
};

use crate::{
    cache::{self, CachePolicy},
    cookie,
};

/// Where a reader lands when they did not ask for anywhere in particular.
pub(crate) const DEFAULT_REDIRECT: &str = "/dashboard";

/// A `redirect` longer than this is not a path anyone linked — it is someone
/// seeing how much they can put in a cookie.
pub(crate) const MAX_REDIRECT_LEN: usize = 512;

/// A 302 carrying `Set-Cookie`, which is the shape of every response here
/// except `session`.
pub(crate) fn redirect(location: &str, cookies: &[String]) -> Response {
    let mut response = StatusCode::FOUND.into_response();
    let headers = response.headers_mut();

    // A location that will not fit in a header is a bug upstream of here; the
    // fallback keeps it a redirect rather than a 302 to nowhere, which browsers
    // render as a blank page.
    headers.insert(
        LOCATION,
        HeaderValue::from_str(location)
            .unwrap_or_else(|_| HeaderValue::from_static("/")),
    );

    cookie::attach(headers, cookies);
    // A cached redirect carrying Set-Cookie hands one reader's session to the
    // next caller of the same URL.
    cache::apply(headers, CachePolicy::NoStore, None);

    response
}

/// Back to the login page with something to show, and whatever cookies the
/// failed attempt left behind cleared.
pub(crate) fn to_login(message: &str, cookies: &[String]) -> Response {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("error", message)
        .finish();

    redirect(&format!("/login?{query}"), cookies)
}

/// Whether a `redirect` parameter may be used as a `Location`.
///
/// This is the open-redirect check. `?redirect=https://evil.test` on a URL that
/// otherwise looks like the real login page is how a phishing page borrows this
/// site's domain, so only same-site absolute paths survive.
pub(crate) fn safe_redirect(candidate: Option<&str>) -> String {
    let allowed = candidate.is_some_and(|path| {
        path.starts_with('/')
            // `//evil.test` is a protocol-relative URL: it starts with a slash
            // and still leaves the site.
            && !path.starts_with("//")
            && path.len() <= MAX_REDIRECT_LEN
            // A backslash is treated as a slash by some browsers, so `/\evil.test`
            // is `//evil.test` to them. A control character is header injection.
            && path.chars().all(|c| c != '\\' && !c.is_control())
    });

    match candidate {
        Some(path) if allowed => path.to_owned(),
        _ => DEFAULT_REDIRECT.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{SESSION_COOKIE, signin::expired_oauth};
    use crate::cookie::Cookie;

    #[test]
    fn a_same_site_path_is_kept() {
        assert_eq!(safe_redirect(Some("/dashboard")), "/dashboard");
        assert_eq!(
            safe_redirect(Some("/books/redis/lessons/1")),
            "/books/redis/lessons/1"
        );
        assert_eq!(safe_redirect(Some("/notes?q=fork")), "/notes?q=fork");
    }

    #[test]
    fn anything_that_leaves_the_site_falls_back() {
        for hostile in [
            "https://evil.test",
            "http://evil.test",
            "//evil.test",
            "/\\evil.test",
            "\\\\evil.test",
            "javascript:alert(1)",
            "dashboard",
            "",
        ] {
            assert_eq!(
                safe_redirect(Some(hostile)),
                DEFAULT_REDIRECT,
                "{hostile:?} was accepted as a redirect target"
            );
        }

        assert_eq!(safe_redirect(None), DEFAULT_REDIRECT);
    }

    #[test]
    fn a_redirect_cannot_inject_a_header_or_fill_a_cookie() {
        assert_eq!(
            safe_redirect(Some("/a\r\nSet-Cookie: x=1")),
            DEFAULT_REDIRECT
        );
        assert_eq!(safe_redirect(Some("/a\nb")), DEFAULT_REDIRECT);

        let long = format!("/{}", "a".repeat(MAX_REDIRECT_LEN));
        assert_eq!(safe_redirect(Some(&long)), DEFAULT_REDIRECT);
    }

    #[test]
    fn a_failure_message_is_escaped_into_the_login_url() {
        let response = to_login("Sign-in was cancelled.", &[]);
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap();

        assert!(location.starts_with("/login?error="));
        assert!(!location.contains(' '));
    }

    #[test]
    fn a_redirect_that_sets_a_cookie_is_never_cached() {
        let response =
            redirect("/dashboard", &[Cookie::new("a", "b").to_header(true)]);
        let cache_control = response
            .headers()
            .get(axum::http::header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .unwrap();

        assert!(cache_control.contains("no-store"));
    }

    #[test]
    fn two_cookies_become_two_headers() {
        let response = redirect(
            "/dashboard",
            &[
                Cookie::new(SESSION_COOKIE, "v").max_age(60).to_header(true),
                expired_oauth(true),
            ],
        );

        assert_eq!(
            response
                .headers()
                .get_all(axum::http::header::SET_COOKIE)
                .iter()
                .count(),
            2
        );
    }
}
