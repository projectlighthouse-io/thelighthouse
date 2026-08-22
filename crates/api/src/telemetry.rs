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

pub(crate) const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

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

    let registry = tracing_subscriber::registry().with(filter);

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

/// Wraps each request in a span carrying its id, method and path.
///
/// Everything logged inside the handler inherits those fields, so a line does
/// not have to remember to include them — which is the difference between
/// correlation that works and correlation that works when someone remembered.
pub(crate) async fn trace_request(mut request: Request, next: Next) -> Response {
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
                tracing::error!(status = status.as_u16(), latency_ms, "request failed");
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
