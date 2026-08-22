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
//!   GET /{provider}/callback     → verify, set lh_session, 302 back in
//!   GET /api/auth/session        → who is this, or null
//!   POST /api/auth/logout        → clear lh_session
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
//! this site started, and the session cookie is signed.
//!
//! Nothing here is cacheable. Every response either sets a cookie or describes
//! one reader, so all of them go out `no-store`.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{COOKIE, LOCATION, SET_COOKIE},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use loginwith::{Callback, Error as LoginError};
use serde::{Deserialize, Serialize};

use crate::{
    api::AppState,
    cache::{self, CachePolicy},
    session::{self, Session},
};

/// The signed session. Site-wide path: every page may ask who the reader is.
const SESSION_COOKIE: &str = "lh_session";

/// The CSRF state, plus where to land afterwards.
///
/// `Path=/` rather than something narrower, because the two routes that use it
/// no longer share a prefix: the login starts at `/api/auth/{provider}` and the
/// provider calls back to `/{provider}/callback`. A cookie scoped to the first
/// would simply not be sent to the second, and every sign-in would fail the
/// state check.
const OAUTH_COOKIE: &str = "lh_oauth";
const OAUTH_PATH: &str = "/";

/// Long enough for a reader to fill in a provider's login form and a 2FA
/// prompt, short enough that an abandoned attempt does not linger.
const OAUTH_MAX_AGE: u64 = 60 * 10;

/// Where a reader lands when they did not ask for anywhere in particular.
const DEFAULT_REDIRECT: &str = "/dashboard";

/// A `redirect` longer than this is not a path anyone linked — it is someone
/// seeing how much they can put in a cookie.
const MAX_REDIRECT_LEN: usize = 512;

/// Deliberately vague, and the same for every failure that is not a cancelled
/// sign-in. What actually happened is in the logs; telling the browser whether
/// a code was rejected or a provider timed out helps nobody but a prober.
const GENERIC_FAILURE: &str = "Sign-in did not complete. Please try again.";
const EXPIRED_FAILURE: &str = "That sign-in link expired. Please try again.";
const CANCELLED: &str = "Sign-in was cancelled.";

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
        .route("/api/auth/session", get(session))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/{provider}", get(start))
        .route("/{provider}/callback", get(callback))
}

#[derive(Debug, Deserialize)]
pub(crate) struct StartQuery {
    /// Where the reader was headed before being asked to sign in. Attacker
    /// controlled — see [`safe_redirect`].
    redirect: Option<String>,
}

/// Mint a state, remember it, and hand the browser to the provider.
async fn start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<StartQuery>,
) -> Response {
    // 404, not 400: an unregistered provider is a URL that does not exist here,
    // and saying otherwise confirms which providers are configured.
    let Some(driver) = state.socials.driver(&provider) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(oauth_state) = loginwith::random_state() else {
        // No fallback. A predictable state is worse than a failed login.
        tracing::error!("cannot read the system random source");
        return to_login(GENERIC_FAILURE, &[]);
    };

    let target = safe_redirect(query.redirect.as_deref());
    let secure = state.config.cookie_secure();

    redirect(
        &driver.authorize_url(&oauth_state),
        &[cookie(
            OAUTH_COOKIE,
            &pack(&oauth_state, &target),
            OAUTH_PATH,
            OAUTH_MAX_AGE,
            secure,
        )],
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    /// Providers send this instead of `code` when the reader declines, or when
    /// the app's registration is wrong.
    error: Option<String>,
}

/// The other half: verify the state, trade the code, sign a session.
async fn callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(driver) = state.socials.driver(&provider) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let secure = state.config.cookie_secure();
    // The attempt is over either way, so the state cookie goes on every path
    // out of here — including the failures, so a stale state cannot be replayed
    // against a second attempt.
    let discard = [cleared(OAUTH_COOKIE, OAUTH_PATH, secure)];

    if let Some(error) = query.error.as_deref() {
        tracing::info!(%provider, %error, "the provider refused the authorization request");
        return to_login(CANCELLED, &discard);
    }

    // A missing cookie is an expired attempt, a session that was never started
    // here, or a browser that dropped it. All three are the same retry.
    let Some(packed) = read_cookie(&headers, OAUTH_COOKIE) else {
        return to_login(EXPIRED_FAILURE, &discard);
    };
    let (expected_state, target) = unpack(packed);

    let Some(code) = query.code.as_deref() else {
        return to_login(GENERIC_FAILURE, &discard);
    };

    let user = match driver
        .user(Callback {
            code,
            // Absent rather than empty is still a state that does not match, and
            // `matches` refuses an empty expectation outright.
            state: query.state.as_deref().unwrap_or_default(),
            expected_state: &expected_state,
        })
        .await
    {
        Ok(user) => user,
        Err(LoginError::InvalidState) => {
            // The one failure that is routinely innocent: a bookmarked callback,
            // a reader who took too long, a second tab. Also what CSRF looks
            // like, which is why it is refused either way.
            tracing::info!(%provider, "the callback state did not match");
            return to_login(EXPIRED_FAILURE, &discard);
        }
        Err(error) => {
            tracing::error!(%provider, error = ?error, "social sign-in failed");
            return to_login(GENERIC_FAILURE, &discard);
        }
    };

    let issued = Session {
        sub: format!("{}:{}", driver.provider().as_str(), user.id),
        provider: driver.provider().as_str().to_owned(),
        // GitHub leaves `name` null far more often than `login`, and a reader
        // with neither gets chrome that says so rather than an empty gap.
        name: user.name.or(user.nickname),
        email: user.email,
        avatar: user.avatar,
        exp: session::now() + session::MAX_AGE,
    };

    let Some(value) = session::encode(&state.config.session_secret, &issued) else {
        tracing::error!(%provider, "cannot encode the session");
        return to_login(GENERIC_FAILURE, &discard);
    };

    tracing::info!(%provider, sub = %issued.sub, "signed in");

    redirect(
        &target,
        &[
            cookie(SESSION_COOKIE, &value, "/", session::MAX_AGE, secure),
            cleared(OAUTH_COOKIE, OAUTH_PATH, secure),
        ],
    )
}

/// What the frontend renders chrome from. Never the source of an entitlement —
/// anything paid is decided server side, against the cookie, not against this.
#[derive(Debug, Serialize)]
struct Reader {
    sub: String,
    provider: String,
    name: Option<String>,
    email: Option<String>,
    avatar: Option<String>,
}

impl From<Session> for Reader {
    fn from(session: Session) -> Self {
        Self {
            sub: session.sub,
            provider: session.provider,
            name: session.name,
            email: session.email,
            avatar: session.avatar,
        }
    }
}

/// Who is this, or `null`.
///
/// 200 with a null body rather than a 401: "nobody" is a correct answer to the
/// question, not a failure to answer it. A 401 would make the anonymous case —
/// the common one — an error in every client that calls this.
async fn session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let reader = read_cookie(&headers, SESSION_COOKIE)
        .and_then(|cookie| session::decode(&state.config.session_secret, cookie))
        .map(Reader::from);

    let mut response = Json(reader).into_response();
    // Describes one reader. No ETag either: revalidation would hand a shared
    // cache a reason to hold it.
    cache::apply(response.headers_mut(), CachePolicy::NoStore, None);
    response
}

/// Clears the cookie. POST, not GET, so a prefetcher or an `<img>` on another
/// site cannot sign a reader out.
///
/// `SameSite=Lax` already stops the cross-site form POST from carrying the
/// cookie, so this needs no CSRF token of its own. The mutations that do —
/// anything that changes stored state — get the double-submit token described
/// in docs/rebuild.md when they exist.
async fn logout(State(state): State<AppState>) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    let headers = response.headers_mut();

    // The path must match the one it was set with, or the browser keeps the
    // original and the reader stays signed in.
    append_cookies(
        headers,
        &[cleared(SESSION_COOKIE, "/", state.config.cookie_secure())],
    );
    cache::apply(headers, CachePolicy::NoStore, None);

    response
}

/// A 302 carrying `Set-Cookie`, which is the shape of every response here
/// except `session`.
fn redirect(location: &str, cookies: &[String]) -> Response {
    let mut response = StatusCode::FOUND.into_response();
    let headers = response.headers_mut();

    // A location that will not fit in a header is a bug upstream of here; the
    // fallback keeps it a redirect rather than a 302 to nowhere, which browsers
    // render as a blank page.
    headers.insert(
        LOCATION,
        HeaderValue::from_str(location).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );

    append_cookies(headers, cookies);
    // A cached redirect carrying Set-Cookie hands one reader's session to the
    // next caller of the same URL.
    cache::apply(headers, CachePolicy::NoStore, None);

    response
}

/// Back to the login page with something to show, and whatever cookies the
/// failed attempt left behind cleared.
fn to_login(message: &str, cookies: &[String]) -> Response {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("error", message)
        .finish();

    redirect(&format!("/login?{query}"), cookies)
}

fn append_cookies(headers: &mut HeaderMap, cookies: &[String]) {
    for value in cookies {
        // `append`, not `insert`: two Set-Cookie headers set two cookies, one
        // insert would drop the first.
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.append(SET_COOKIE, value);
        }
    }
}

fn cookie(name: &str, value: &str, path: &str, max_age: u64, secure: bool) -> String {
    // `SameSite=Lax`, not `Strict`. Strict would withhold the state cookie on
    // the callback — that arrives as a top-level navigation *from the
    // provider's origin*, which is exactly what Strict is for suppressing — and
    // the login would fail every time with a state mismatch.
    let secure = if secure { "; Secure" } else { "" };

    format!("{name}={value}; Path={path}; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}")
}

/// Same name, same path, no value, immediate expiry. A browser matches a
/// replacement on name *and* path, so clearing with the wrong path silently
/// leaves the cookie in place.
fn cleared(name: &str, path: &str, secure: bool) -> String {
    cookie(name, "", path, 0, secure)
}

/// Reads one cookie out of the request.
///
/// Hand rolled rather than a cookie crate: there are two cookies, both minted
/// above, both values restricted to characters that need no escaping. The
/// dependency would bring percent-decoding, jars and its own signing, none of
/// which is used here.
fn read_cookie<'h>(headers: &'h HeaderMap, name: &str) -> Option<&'h str> {
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

/// The state and the landing path, in one cookie.
///
/// One cookie rather than two because they have one lifetime: both are written
/// at redirect and both are dead at callback. The path is base64 so its own
/// `/`, `=` and `&` cannot end the cookie value early.
fn pack(state: &str, redirect_to: &str) -> String {
    format!("{state}.{}", URL_SAFE_NO_PAD.encode(redirect_to))
}

/// Splits [`pack`], and treats anything unreadable as "no state, default
/// destination" — which fails the state check below rather than here.
fn unpack(packed: &str) -> (String, String) {
    let Some((state, encoded)) = packed.split_once('.') else {
        return (String::new(), DEFAULT_REDIRECT.to_owned());
    };

    let target = URL_SAFE_NO_PAD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());

    // Validated again on the way out, not only on the way in. The cookie is
    // HttpOnly, but a value that only gets checked once is a value that stops
    // being checked the day something else starts writing it.
    (state.to_owned(), safe_redirect(target.as_deref()))
}

/// Whether a `redirect` parameter may be used as a `Location`.
///
/// This is the open-redirect check. `?redirect=https://evil.test` on a URL that
/// otherwise looks like the real login page is how a phishing page borrows this
/// site's domain, so only same-site absolute paths survive.
fn safe_redirect(candidate: Option<&str>) -> String {
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
    use axum::body::Body;
    use axum::http::Request;
    use loginwith::{GithubProvider, GoogleProvider};
    use tower::ServiceExt as _;

    use super::*;
    use crate::session::MAX_AGE;

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
    fn the_state_and_the_destination_survive_one_cookie() {
        let packed = pack("the-state", "/books/redis?page=2");

        assert_eq!(
            unpack(&packed),
            ("the-state".to_owned(), "/books/redis?page=2".to_owned())
        );
    }

    #[test]
    fn a_tampered_cookie_yields_a_state_that_cannot_match() {
        // Empty expected state is refused outright by loginwith::state::matches,
        // so a torn cookie is a failed login rather than a waved-through one.
        assert_eq!(unpack("").0, "");
        assert_eq!(unpack("no-separator").0, "");
        assert_eq!(unpack("").1, DEFAULT_REDIRECT);
    }

    #[test]
    fn a_destination_smuggled_into_the_cookie_is_still_checked() {
        let packed = pack("the-state", "https://evil.test");

        assert_eq!(unpack(&packed).1, DEFAULT_REDIRECT);
    }

    fn with_cookie(header: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(header).unwrap());
        headers
    }

    #[test]
    fn one_cookie_is_found_among_many() {
        let headers = with_cookie("theme=dark; lh_session=abc.def; locale=bn");

        assert_eq!(read_cookie(&headers, "lh_session"), Some("abc.def"));
        assert_eq!(read_cookie(&headers, "theme"), Some("dark"));
        assert_eq!(read_cookie(&headers, "absent"), None);
    }

    #[test]
    fn a_name_that_merely_starts_the_same_does_not_answer() {
        let headers = with_cookie("lh_session_old=stale; lh_sessionx=no");

        assert_eq!(read_cookie(&headers, "lh_session"), None);
    }

    #[test]
    fn a_cookie_split_across_headers_is_still_read() {
        let mut headers = with_cookie("theme=dark");
        headers.append(COOKIE, HeaderValue::from_static("lh_session=abc.def"));

        assert_eq!(read_cookie(&headers, "lh_session"), Some("abc.def"));
    }

    #[test]
    fn a_session_cookie_is_not_reachable_from_script_and_not_sent_cross_site() {
        let value = cookie(SESSION_COOKIE, "v", "/", session::MAX_AGE, true);

        assert!(value.contains("HttpOnly"), "xss could read the session");
        assert!(value.contains("SameSite=Lax"));
        assert!(value.contains("Secure"));
        assert!(value.contains("Path=/"));
    }

    #[test]
    fn plain_http_omits_secure_so_local_development_can_sign_in() {
        // A Secure cookie is dropped outright over http://localhost, which would
        // make the whole flow look broken rather than misconfigured.
        assert!(!cookie(SESSION_COOKIE, "v", "/", 60, false).contains("Secure"));
    }

    #[test]
    fn clearing_uses_the_path_the_cookie_was_set_with() {
        let value = cleared(OAUTH_COOKIE, OAUTH_PATH, true);

        assert!(value.contains("Max-Age=0"));
        assert!(value.contains(&format!("Path={OAUTH_PATH}")));
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
        let response = redirect("/dashboard", &[cookie("a", "b", "/", 60, true)]);
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
                cookie(SESSION_COOKIE, "v", "/", 60, true),
                cleared(OAUTH_COOKIE, OAUTH_PATH, true),
            ],
        );

        assert_eq!(response.headers().get_all(SET_COOKIE).iter().count(), 2);
    }

    // ---- the routes, driven through the real router --------------------
    //
    // These exist because the mounting is the part that fails silently: these
    // four routes sit outside the signature layer, and if they ever drift
    // inside it every one of them answers 404 and the site simply has no login.

    fn router() -> axum::Router {
        let config = crate::config::Config::sample();
        let socials = loginwith::providers([
            GithubProvider::with("gh-id", "gh-secret", config.callback_url("github")),
            GoogleProvider::with("goo-id", "goo-secret", config.callback_url("google")),
        ])
        .unwrap();

        // Lazy: nothing on these routes queries, and a pool that only connects
        // on first use keeps the auth tests runnable without a postgres. The
        // day one of them does touch the database, it will fail loudly here
        // rather than quietly passing against a stub.
        let db = sqlx::postgres::PgPool::connect_lazy("postgres://localhost/unused").unwrap();

        crate::api::app(config, socials, db)
    }

    async fn get(uri: &str, cookie_header: Option<&str>) -> Response {
        let mut request = Request::builder().uri(uri);
        if let Some(value) = cookie_header {
            request = request.header(COOKIE, value);
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
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(str::to_owned)
            .collect()
    }

    #[tokio::test]
    async fn starting_a_login_redirects_to_the_provider_carrying_the_state_it_stored() {
        let response = get("/api/auth/github?redirect=/notes", None).await;

        assert_eq!(response.status(), StatusCode::FOUND);

        let location = header(&response, LOCATION);
        assert!(location.starts_with("https://github.com/login/oauth/authorize?"));
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
        let response = get("/github/callback?error=access_denied&state=y", None).await;

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
        assert!(header(&response, axum::http::header::CACHE_CONTROL).contains("no-store"));
    }

    #[tokio::test]
    async fn an_anonymous_reader_is_null_rather_than_an_error() {
        let response = get("/api/auth/session", Some("theme=dark")).await;
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();

        assert_eq!(&body[..], b"null");
    }

    #[tokio::test]
    async fn a_signed_cookie_names_the_reader() {
        let issued = Session {
            sub: "github:1".to_owned(),
            provider: "github".to_owned(),
            name: Some("Octocat".to_owned()),
            email: Some("o@x.test".to_owned()),
            avatar: None,
            exp: session::now() + MAX_AGE,
        };
        let value = session::encode("session", &issued).unwrap();

        let response = get(
            "/api/auth/session",
            Some(&format!("theme=dark; lh_session={value}")),
        )
        .await;
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("github:1"));
        assert!(body.contains("Octocat"));
    }

    #[tokio::test]
    async fn a_cookie_this_process_did_not_sign_is_anonymous() {
        let forged = session::encode(
            "not-the-session-secret",
            &Session {
                sub: "github:999".to_owned(),
                provider: "github".to_owned(),
                name: None,
                email: None,
                avatar: None,
                exp: session::now() + MAX_AGE,
            },
        )
        .unwrap();

        let response = get("/api/auth/session", Some(&format!("lh_session={forged}"))).await;
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();

        assert_eq!(&body[..], b"null");
    }

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
        let cleared = set_cookies(&response);
        assert_eq!(cleared.len(), 1);
        assert!(cleared.first().unwrap().starts_with("lh_session=;"));
        assert!(cleared.first().unwrap().contains("Max-Age=0"));

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
        for uri in ["/api/ping", "/api/books", "/api/me"] {
            assert_eq!(
                get(uri, None).await.status(),
                StatusCode::NOT_FOUND,
                "{uri}"
            );
        }
    }
}
