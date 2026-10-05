//! Structured logging and request correlation.
//!
//! A request crosses Caddy, then either Nuxt or straight into this process.
//! Without a shared id those are three unrelated log lines and there is no way
//! to follow one request through. Caddy mints the id and forwards it; this
//! module reads it, puts it on every line the request produces, and echoes it
//! back so a reader can quote it in a bug report.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::{Instrument, info_span};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub(crate) const REQUEST_ID: HeaderName =
    HeaderName::from_static("x-request-id");

/// Only used when a request arrives without an id — a loopback call from Nuxt,
/// or anything that bypassed Caddy. Not a UUID: this needs to be unique within
/// a process lifetime, not globally, and a counter plus the start timestamp
/// gets there without a dependency.
fn generated_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    static START: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

    let start = *START.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    });
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{start:x}-{n:x}")
}

/// JSON to stdout. App Platform captures stdout, and JSON keeps the fields
/// queryable if they are ever shipped somewhere that can index them.
///
/// `RUST_LOG` controls the level. Defaulting to `info` rather than `warn`
/// because a server that logs nothing on a normal request is a server you
/// cannot debug from production.
pub(crate) fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug"));

    let json = std::env::var("LOG_FORMAT").as_deref() != Ok("pretty");

    // Errors become Sentry events, info and up become breadcrumbs, spans become
    // spans. A no-op when `sentry()` found no DSN.
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(sentry::integrations::tracing::layer());

    if json {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init();
    } else {
        // Local development. Unreadable JSON in a terminal is a real cost when
        // the thing you are doing is reading logs.
        registry.with(fmt::layer().pretty()).init();
    }
}

/// Error reporting. Off unless `SENTRY_DSN` is set, so a clone with no account
/// reports nothing anywhere.
///
/// Read here rather than in `Config` for the same reason as `RUST_LOG`: it has
/// to be running before anything it should catch, a bad `Config` included. The
/// guard flushes pending events on drop, so `main` holds it to the end.
pub(crate) fn sentry() -> Result<Option<sentry::ClientInitGuard>, String> {
    Ok(sentry_options(|key| std::env::var(key).ok())?.map(sentry::init))
}

/// `SENTRY_SEND_DEFAULT_PII` defaults to `false`, `SENTRY_TRACES_SAMPLE_RATE` to
/// `1.0`. Either set to something unreadable is a boot failure, not a default.
fn sentry_options(
    get: impl Fn(&str) -> Option<String>,
) -> Result<Option<sentry::ClientOptions>, String> {
    let set = |key: &str| {
        get(key)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    };

    let Some(dsn) = set("SENTRY_DSN") else {
        return Ok(None);
    };

    let dsn = dsn
        .parse::<sentry::types::Dsn>()
        .map_err(|error| format!("SENTRY_DSN is not a valid DSN: {error}"))?;

    let send_default_pii = match set("SENTRY_SEND_DEFAULT_PII").as_deref() {
        Some("true") => true,
        // Off unless asked for: on, the request headers go too, session cookie
        // included, and reading the Sentry project becomes taking a session.
        None | Some("false") => false,
        Some(other) => {
            return Err(format!(
                "SENTRY_SEND_DEFAULT_PII must be true or false, and it is {other:?}"
            ));
        }
    };

    let traces_sample_rate = match set("SENTRY_TRACES_SAMPLE_RATE") {
        None => 1.0,
        Some(rate) => rate
            .parse::<f32>()
            .ok()
            .filter(|rate| (0.0..=1.0).contains(rate))
            .ok_or_else(|| {
                format!(
                    "SENTRY_TRACES_SAMPLE_RATE must be between 0 and 1, and it is {rate:?}"
                )
            })?,
    };

    // The builder's `dsn` and `traces_sample_rate` panic on a bad value; both
    // are checked above, and the dsn goes in as the parsed field regardless.
    let mut options = sentry::ClientOptions::new()
        .maybe_release(sentry::release_name!())
        .send_default_pii(send_default_pii)
        .traces_sample_rate(traces_sample_rate);
    options.dsn = Some(dsn);

    Ok(Some(options))
}

/// Wraps each request in a span carrying its id, method and path.
///
/// Everything logged inside the handler inherits those fields, so a line does
/// not have to remember to include them — which is the difference between
/// correlation that works and correlation that works when someone remembered.
pub(crate) async fn trace_request(
    mut request: Request,
    next: Next,
) -> Response {
    let id = request
        .headers()
        .get(&REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .map_or_else(generated_id, str::to_owned);

    let method = request.method().clone();
    let path = request.uri().path().to_owned();

    // Put it back on the request so anything downstream sees the same id,
    // including a generated one.
    if let Ok(value) = HeaderValue::from_str(&id) {
        request.headers_mut().insert(REQUEST_ID, value.clone());

        let span = info_span!("request", request_id = %id, method = %method, path = %path);

        let started = Instant::now();
        let mut response = next.run(request).instrument(span.clone()).await;
        let status = response.status();

        // The span alone emits nothing — it only decorates events logged inside
        // it. Without this line a request produces no log at all, which is how
        // correlation ends up wired but useless.
        span.in_scope(|| {
            let latency_ms = started.elapsed().as_millis();
            if status.is_server_error() {
                tracing::error!(
                    status = status.as_u16(),
                    latency_ms,
                    "request failed"
                );
            } else {
                tracing::info!(status = status.as_u16(), latency_ms, "request");
            }
        });

        // Echoed back so a reader can quote it and it can be found in the logs.
        response.headers_mut().insert(REQUEST_ID, value);
        return response;
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::sentry_options;

    const DSN: &str = "https://public@sentry.example.com/1";

    fn from(
        pairs: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Option<String> {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn no_dsn_means_no_sentry() {
        assert!(sentry_options(from(&[])).unwrap().is_none());
        assert!(sentry_options(from(&[("SENTRY_DSN", " ")])).unwrap().is_none());
    }

    #[test]
    fn a_dsn_alone_takes_the_defaults() {
        let options = sentry_options(from(&[("SENTRY_DSN", DSN)]))
            .unwrap()
            .unwrap();

        assert!(!options.send_default_pii);
    }

    #[test]
    fn the_defaults_can_be_overridden() {
        let options = sentry_options(from(&[
            ("SENTRY_DSN", DSN),
            ("SENTRY_SEND_DEFAULT_PII", "true"),
            ("SENTRY_TRACES_SAMPLE_RATE", "0.25"),
        ]))
        .unwrap()
        .unwrap();

        assert!(options.send_default_pii);
    }

    #[test]
    fn unreadable_values_refuse_to_boot() {
        for pairs in [
            &[("SENTRY_DSN", "not a dsn")][..],
            &[("SENTRY_DSN", DSN), ("SENTRY_SEND_DEFAULT_PII", "yes")][..],
            &[("SENTRY_DSN", DSN), ("SENTRY_TRACES_SAMPLE_RATE", "1.5")][..],
            &[("SENTRY_DSN", DSN), ("SENTRY_TRACES_SAMPLE_RATE", "NaN")][..],
        ] {
            assert!(sentry_options(from(pairs)).is_err(), "{pairs:?}");
        }
    }
}
