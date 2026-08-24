//! Social sign-in: the two routes the browser walks, and the cookies that
//! survive them.
//!
//! `loginwith` owns the OAuth mechanics — the authorize URL, the state check,
//! the token exchange, the profile mapping. What is here is the half that crate
//! deliberately refuses to guess at: where the state lives between the redirect
//! and the callback, what a signed-in reader *is*, and where a browser is sent
//! afterwards.
//!
//! ```text
//!   GET /api/auth/{provider}     → set lh_oauth, 302 to the provider
//!   GET /{provider}/callback     → verify, resolve a user, open a session
//!   GET /api/auth/session        → who is this, or null
//!   POST /api/auth/logout        → delete the session row, clear lh_session
//! ```
//!
//! The callback is at the root while everything else is under `/api/auth`, and
//! that is not an oversight: its URL is registered with Google and GitHub, so
//! it keeps the path the laravel app already serves rather than making the
//! cutover depend on editing two OAuth consoles first.
//!
//! None of these carry a luxctl signature — an OAuth callback is a browser
//! navigation and cannot — so Caddy routes them to the api unsigned and they
//! authenticate themselves: the state cookie is what ties a callback to a login
//! this site started, and the session cookie names a row that only this process
//! could have written.
//!
//! Nothing here is cacheable. Every response either sets a cookie or describes
//! one reader, so all of them go out `no-store`.

mod reader;
mod redirect;
mod signin;

use axum::{
    Router,
    routing::{get, post},
};

use crate::api::AppState;

/// The session id. Site-wide path: every page may ask who the reader is.
///
/// Opaque — 32 random bytes naming a row. Nothing about the reader travels in
/// it, so there is nothing in it to read or to edit. See `session`.
pub(crate) const SESSION_COOKIE: &str = "lh_session";

/// A companion to the session that javascript is allowed to read.
///
/// It carries no identity — the value is always `1` — and grants nothing. Its
/// only job is to let the frontend know, *before it has asked anyone*, which
/// shape the header should be.
///
/// Without it the frontend cannot tell a signed-in reader from an anonymous one
/// until `/api/auth/session` answers, because the real session cookie is
/// `HttpOnly` and deliberately unreadable. So every page load drew a join button
/// at signed-in readers and swapped it a round trip later. Server-rendering the
/// answer instead would make every page per-reader and uncacheable, which is a
/// far worse trade than one extra cookie.
///
/// Set and cleared in lockstep with the session. If they ever disagree the
/// frontend corrects itself the moment the session endpoint replies — this is a
/// hint, and nothing is trusted because of it.
pub(crate) const READER_COOKIE: &str = "lh_reader";

/// The CSRF state, plus where to land afterwards.
///
/// `Path=/` rather than something narrower, because the two routes that use it
/// no longer share a prefix: the login starts at `/api/auth/{provider}` and the
/// provider calls back to `/{provider}/callback`. A cookie scoped to the first
/// would simply not be sent to the second, and every sign-in would fail the
/// state check.
pub(crate) const OAUTH_COOKIE: &str = "lh_oauth";

pub(crate) const OAUTH_PATH: &str = "/";

/// Mounted on the root router with absolute paths rather than nested, so the
/// paths in this file are the paths in the Caddyfile — no prefix to hold in
/// your head while reading either one.
///
/// **The callback sits at the root, not under `/api/auth`, and that asymmetry
/// is deliberate.** Every other path here is ours to choose; the callback is
/// registered with Google and GitHub and appears in their consent screens, so
/// it is the one URL the rebuild cannot rename — see docs/rebuild.md. It keeps
/// the path the laravel app serves (`config/services.php`), which is what makes
/// the cutover a DNS change rather than a round trip through two OAuth consoles.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/session", get(reader::session))
        .route("/api/auth/logout", post(reader::logout))
        .route("/api/auth/{provider}", get(signin::start))
        .route("/{provider}/callback", get(signin::callback))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header::LOCATION};
    use axum::response::Response;
    use loginwith::{GithubProvider, GoogleProvider};
    use tower::ServiceExt as _;

    use super::*;
    use crate::auth::signin::unpack;

    // These exist because the mounting is the part that fails silently: these
    // four routes sit outside the signature layer, and if they ever drift
    // inside it every one of them answers 404 and the site simply has no login.

    fn router() -> axum::Router {
        let config = crate::config::Config::sample();
        let socials = loginwith::providers([
            GithubProvider::with(
                "gh-id",
                "gh-secret",
                config.callback_url("github"),
            ),
            GoogleProvider::with(
                "goo-id",
                "goo-secret",
                config.callback_url("google"),
            ),
        ])
        .unwrap();

        // Lazy: nothing on these routes queries, and a pool that only connects
        // on first use keeps the auth tests runnable without a postgres. The
        // day one of them does touch the database, it will fail loudly here
        // rather than quietly passing against a stub.
        let db =
            sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused")
                .unwrap();

        // The fixture repo, not `config.content_path`: these tests are about
        // routing and cookies, and reading the real ohara would fail them on
        // whatever happens to be mid-edit there.
        let catalog = std::sync::Arc::new(
            crate::ohara::catalog::Catalog::load(
                crate::ohara::fixture::content(),
            )
            .unwrap(),
        );

        crate::api::app(config, socials, db, catalog)
    }

    async fn get(uri: &str, cookie_header: Option<&str>) -> Response {
        let mut request = Request::builder().uri(uri);
        if let Some(value) = cookie_header {
            request = request.header(axum::http::header::COOKIE, value);
        }

        router()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    fn header(response: &Response, name: axum::http::HeaderName) -> String {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned()
    }

    fn set_cookies(response: &Response) -> Vec<String> {
        response
            .headers()
            .get_all(axum::http::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(str::to_owned)
            .collect()
    }

    #[tokio::test]
    async fn starting_a_login_redirects_to_the_provider_carrying_the_state_it_stored()
     {
        let response = get("/api/auth/github?redirect=/notes", None).await;

        assert_eq!(response.status(), StatusCode::FOUND);

        let location = header(&response, LOCATION);
        assert!(
            location.starts_with("https://github.com/login/oauth/authorize?")
        );
        assert!(location.contains("client_id=gh-id"));

        let stored = set_cookies(&response);
        assert_eq!(stored.len(), 1);
        let stored = stored.first().unwrap();
        assert!(stored.starts_with("lh_oauth="));
        assert!(stored.contains("HttpOnly"));

        // The state in the URL and the state in the cookie have to be the same
        // value, or every callback fails the CSRF check.
        let (state, _) = unpack(read_back(stored));
        assert!(!state.is_empty());
        assert!(location.contains(&format!("state={state}")));

        // ...and the destination survived alongside it.
        assert_eq!(unpack(read_back(stored)).1, "/notes");
    }

    /// The cookie value out of a `Set-Cookie` line, as a browser would send it
    /// back.
    fn read_back(set_cookie: &str) -> &str {
        set_cookie
            .split(';')
            .next()
            .and_then(|pair| pair.split_once('='))
            .map(|(_, value)| value)
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn two_logins_do_not_share_a_state() {
        let first = set_cookies(&get("/api/auth/github", None).await);
        let second = set_cookies(&get("/api/auth/github", None).await);

        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn an_unregistered_provider_is_a_404_not_a_hint() {
        for uri in [
            "/api/auth/facebook",
            "/facebook/callback",
            "/api/auth/GitHub",
        ] {
            assert_eq!(
                get(uri, None).await.status(),
                StatusCode::NOT_FOUND,
                "{uri}"
            );
        }
    }

    #[tokio::test]
    async fn a_callback_without_a_state_cookie_goes_back_to_login() {
        let response = get("/github/callback?code=x&state=y", None).await;

        assert_eq!(response.status(), StatusCode::FOUND);
        assert!(header(&response, LOCATION).starts_with("/login?error="));
        // The attempt is over, so the state cookie is cleared even on the way out.
        assert!(
            set_cookies(&response)
                .iter()
                .any(|c| c.contains("Max-Age=0"))
        );
    }

    #[tokio::test]
    async fn a_declined_login_says_so_and_never_reaches_the_provider() {
        let response =
            get("/github/callback?error=access_denied&state=y", None).await;

        assert_eq!(response.status(), StatusCode::FOUND);
        assert!(header(&response, LOCATION).contains("cancelled"));
    }

    #[tokio::test]
    async fn the_session_route_is_reachable_without_a_signature() {
        // The regression this guards: /api/* is 404 unless it carries a luxctl
        // HMAC, and the browser has none. If this ever returns 404, the whole
        // frontend renders every reader as signed out.
        let response = get("/api/auth/session", None).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            header(&response, axum::http::header::CACHE_CONTROL)
                .contains("no-store")
        );
    }

    #[tokio::test]
    async fn an_anonymous_reader_is_null_rather_than_an_error() {
        let response = get("/api/auth/session", Some("theme=dark")).await;
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();

        assert_eq!(&body[..], b"null");
    }

    // There is no test here for "a cookie naming a real session names the
    // reader": a session is a row now, so proving that needs a database, and
    // these tests deliberately run without one. What that costs is stated in
    // CLAUDE.local.md — the end-to-end check in the plan covers it instead.

    #[tokio::test]
    async fn signing_out_clears_the_cookie_and_refuses_a_get() {
        let response = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Both of them. The readable companion left behind would keep the
        // frontend drawing a signed-in header for somebody who is not.
        let cleared = set_cookies(&response);
        assert_eq!(cleared.len(), 2);
        for name in [SESSION_COOKIE, READER_COOKIE] {
            let value = cleared
                .iter()
                .find(|c| c.starts_with(&format!("{name}=;")))
                .unwrap_or_else(|| panic!("{name} was not cleared"));
            assert!(value.contains("Max-Age=0"));
        }

        // GET would let a prefetch or an <img> sign a reader out.
        assert_eq!(
            get("/api/auth/logout", None).await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn the_signed_routes_are_still_signed() {
        // The other half of the mounting: adding the auth routes must not have
        // opened up everything else on /api/*.
        //
        // `/api/books` is *not* here. It used to be a signed placeholder and is
        // now the real, public book listing — the same bytes for everyone, and
        // the only kind of response the edge may hold.
        for uri in ["/api/ping", "/api/me"] {
            assert_eq!(
                get(uri, None).await.status(),
                StatusCode::NOT_FOUND,
                "{uri}"
            );
        }
    }

    #[tokio::test]
    async fn a_write_without_a_session_is_refused_before_anything_else() {
        // Ordering, not just the outcome: the session check runs first, so a
        // write with no cookie is 401 and never reaches the csrf comparison —
        // which would otherwise have no session token to compare against. It
        // also never reaches the json extractor, which is why an empty body is
        // enough here.
        for (method, uri) in [
            ("POST", "/api/notes"),
            ("PATCH", "/api/notes/1"),
            ("DELETE", "/api/notes/1"),
        ] {
            let response = router()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{method} {uri}"
            );
        }
    }

    #[tokio::test]
    async fn a_reader_route_refuses_anyone_without_a_session_cookie() {
        // The third mounting: /api/notes is outside the signature layer — the
        // browser has no HMAC — and inside the session layer. If it ever drifts
        // out of both, one reader's notes are readable by anybody who asks.
        //
        // No cookie means no database is touched, which is why this one can run
        // here. A cookie naming an unknown session needs postgres.
        for cookies in [None, Some("theme=dark"), Some("lh_session_old=stale")]
        {
            assert_eq!(
                get("/api/notes", cookies).await.status(),
                StatusCode::UNAUTHORIZED,
                "{cookies:?}"
            );
        }
    }
}
