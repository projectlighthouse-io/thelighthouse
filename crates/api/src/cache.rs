//! Cache policy for API responses.
//!
//! The rule this exists to enforce: **a response that depended on who was
//! asking must never reach a shared cache.** Get that wrong once and a CDN
//! serves one reader's paid lesson to everyone.
//!
//! So the default is `CachePolicy::Private`. A handler has to opt *in* to
//! being cached, and the only way to do that is to say out loud that the
//! response is the same for everybody. Forgetting fails closed.

// This module is the api's cache *vocabulary*, and all of it belongs here
// whether or not an endpoint currently reaches for every word. `Private` and
// the three revalidation helpers lost their only callers when the `/api/me`
// and `/api/ping` placeholders were replaced by `projects`: a CLI has no
// browser cache to revalidate against, so none of luxctl's ten endpoints wants
// them, and the browser endpoints that will are later slices of
// `docs/rebuild.md`. Deleting them would mean rederiving `must-revalidate` and
// weak ETags from scratch later, and getting one of those subtly wrong is how a
// shared cache ends up holding one reader's own page.
#![allow(dead_code)]

use axum::http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH, VARY},
};
use sha2::{Digest, Sha256};

/// How a response may be stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CachePolicy {
    /// Identical for every caller. Safe for a CDN to hold and reuse.
    Shared {
        max_age: u32,
        stale_while_revalidate: u32,
    },
    /// Depends on the session. The browser may keep it; nothing shared may.
    ///
    /// The middle case: safe for the reader's own browser to keep and
    /// revalidate, never for anything shared. Collapsing it into `NoStore`
    /// would make every signed-in page refetch from scratch.
    Private,
    /// Never stored anywhere. Anything entitlement-dependent belongs here —
    /// `private` alone still permits a browser cache, and a shared device is
    /// enough for that to matter.
    NoStore,
}

impl CachePolicy {
    /// A free lesson, a book listing — the same bytes for anyone.
    /// Content whose shape depends on entitlement, for a reader who has none.
    ///
    /// The same bytes for every unentitled reader, so it is still shareable —
    /// but a reader who buys mid-window keeps seeing the locked copy until it
    /// expires, so the window is short. The *entitled* answer is never this:
    /// it carries paid prose and is [`CachePolicy::NoStore`].
    pub(crate) const fn withheld_content() -> Self {
        Self::Shared {
            max_age: 60,
            stale_while_revalidate: 300,
        }
    }

    pub(crate) const fn public_content() -> Self {
        Self::Shared {
            max_age: 300,
            stale_while_revalidate: 86_400,
        }
    }

    fn header_value(self) -> HeaderValue {
        match self {
            Self::Shared { max_age, stale_while_revalidate } => HeaderValue::from_str(&format!(
                "public, max-age=0, s-maxage={max_age}, stale-while-revalidate={stale_while_revalidate}"
            ))
            .unwrap_or_else(|_| HeaderValue::from_static("private, no-store")),
            Self::Private => HeaderValue::from_static("private, max-age=0, must-revalidate"),
            Self::NoStore => HeaderValue::from_static("private, no-store"),
        }
    }
}

/// Weak `ETag` over the body.
///
/// Weak because the payload is JSON we generated — two byte-identical bodies
/// are the same resource, and we make no promise about byte-for-byte stability
/// across versions.
pub(crate) fn etag_for(body: &[u8]) -> Option<HeaderValue> {
    let digest = Sha256::digest(body);
    let hex = digest.iter().take(16).fold(String::new(), |mut out, b| {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
        out
    });
    HeaderValue::from_str(&format!("W/\"{hex}\"")).ok()
}

/// Whether the caller already has this exact body.
pub(crate) fn matches_if_none_match(
    headers: &HeaderMap,
    etag: &HeaderValue,
) -> bool {
    headers
        .get_all(IF_NONE_MATCH)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(str::trim)
        .any(|candidate| {
            candidate == etag.to_str().unwrap_or_default() || candidate == "*"
        })
}

/// Applies the policy, and `Vary` alongside it.
///
/// `Vary` is not optional. A shared cache keys on the URL; without being told
/// that the response depends on the cookie or the signature, it will happily
/// reuse one caller's response for the next. That is the same leak as a missing
/// `no-store`, arriving by a different route.
pub(crate) fn apply(
    headers: &mut HeaderMap,
    cache_policy: CachePolicy,
    etag: Option<HeaderValue>,
) {
    headers.insert(CACHE_CONTROL, cache_policy.header_value());

    let vary = match cache_policy {
        // Even for shared content: the response body is the same, but the
        // encoding is not, and an entitled caller may get a different one later.
        CachePolicy::Shared { .. } => "Accept-Encoding",
        CachePolicy::Private | CachePolicy::NoStore => {
            "Accept-Encoding, Cookie, Authorization"
        }
    };
    headers.insert(VARY, HeaderValue::from_static(vary));

    if let Some(etag) = etag {
        headers.insert(ETAG, etag);
    }
}

/// 304 when the caller's copy is current.
pub(crate) const NOT_MODIFIED: StatusCode = StatusCode::NOT_MODIFIED;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_is_the_default_shape() {
        // The one that matters: nothing session-dependent may say "public".
        let v = CachePolicy::Private.header_value();
        assert!(v.to_str().unwrap().starts_with("private"));
        let v = CachePolicy::NoStore.header_value();
        assert!(v.to_str().unwrap().contains("no-store"));
    }

    #[test]
    fn shared_content_lets_a_cdn_hold_it_but_not_the_browser() {
        let v = CachePolicy::public_content().header_value();
        let v = v.to_str().unwrap();
        // max-age=0 so the browser revalidates and picks up a deploy
        assert!(v.contains("max-age=0"));
        assert!(v.contains("s-maxage=300"));
        assert!(v.contains("stale-while-revalidate"));
    }

    #[test]
    fn session_dependent_responses_vary_on_cookie() {
        let mut headers = HeaderMap::new();
        apply(&mut headers, CachePolicy::NoStore, None);
        let vary = headers.get(VARY).unwrap().to_str().unwrap();
        assert!(
            vary.contains("Cookie"),
            "a shared cache would reuse this across users"
        );
    }

    #[test]
    fn etag_changes_with_the_body() {
        let a = etag_for(b"one").unwrap();
        let b = etag_for(b"two").unwrap();
        assert_ne!(a, b);
        assert_eq!(a, etag_for(b"one").unwrap());
    }

    #[test]
    fn if_none_match_accepts_a_list_and_a_wildcard() {
        let etag = etag_for(b"body").unwrap();
        let mut headers = HeaderMap::new();

        headers.insert(IF_NONE_MATCH, etag.clone());
        assert!(matches_if_none_match(&headers, &etag));

        headers.insert(
            IF_NONE_MATCH,
            HeaderValue::from_str(&format!(
                "W/\"other\", {}",
                etag.to_str().unwrap()
            ))
            .unwrap(),
        );
        assert!(matches_if_none_match(&headers, &etag));

        headers.insert(IF_NONE_MATCH, HeaderValue::from_static("*"));
        assert!(matches_if_none_match(&headers, &etag));

        headers.insert(IF_NONE_MATCH, HeaderValue::from_static("W/\"nope\""));
        assert!(!matches_if_none_match(&headers, &etag));
    }
}
