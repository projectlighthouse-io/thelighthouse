# One image: caddy in front, nuxt and the rust api behind it on loopback.
#
# Base images are pinned to a minor rather than a floating tag. A moving base is
# the largest supply-chain surface in a build, and pinning the dependencies
# inside it while the image itself drifts achieves little.
ARG NODE_VERSION=26.0-alpine
ARG RUST_VERSION=1.98-alpine
ARG CADDY_VERSION=2.10-alpine

# web deps
# Split from the build so a source-only change reuses the install layer.
FROM node:${NODE_VERSION} AS web-deps

WORKDIR /build
COPY web/package.json web/package-lock.json web/.npmrc ./

# --ignore-scripts blocks postinstall hooks, the usual delivery mechanism for a
# compromised package. nuxt prepare runs explicitly in the next stage.
RUN npm ci --ignore-scripts

# web build
FROM node:${NODE_VERSION} AS web-build

WORKDIR /build
COPY --from=web-deps /build/node_modules ./node_modules
COPY web/ .

ENV NODE_ENV=production
ENV NUXT_TELEMETRY_DISABLED=1

RUN npx nuxt prepare && npx nuxt build

# api build
FROM rust:${RUST_VERSION} AS api-build

RUN apk add --no-cache musl-dev

WORKDIR /build

# Manifests first: dependencies only rebuild when they actually change, which
# is the difference between a 20-second and a four-minute rebuild.
COPY Cargo.toml Cargo.lock* ./
COPY crates/api/Cargo.toml ./crates/api/
RUN mkdir -p crates/api/src \
    && echo 'fn main() {}' > crates/api/src/main.rs \
    && cargo build --release --locked 2>/dev/null || cargo build --release

COPY crates ./crates
# cargo skips a rebuild if mtime looks unchanged, and the stub above shares one
RUN touch crates/api/src/main.rs && cargo build --release

# runtime
FROM caddy:${CADDY_VERSION} AS runtime

# bash for the entrypoint's `wait -n`, nodejs to run the nitro output. The rust
# binary is static against musl and needs nothing.
RUN apk add --no-cache bash nodejs

WORKDIR /app

ENV NODE_ENV=production
ENV NUXT_TELEMETRY_DISABLED=1
ENV PORT=8080
ENV API_PORT=9000
ENV NUXT_PORT=3000

COPY Caddyfile /etc/caddy/Caddyfile
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY --from=api-build /build/target/release/lighthouse-api /usr/local/bin/lighthouse-api
COPY --from=web-build /build/.output /app/web

RUN chmod +x /usr/local/bin/entrypoint.sh \
    && addgroup -S lighthouse && adduser -S -G lighthouse lighthouse \
    && chown -R lighthouse:lighthouse /app /etc/caddy

# Caddy binds 8080, not 80, so it does not need CAP_NET_BIND_SERVICE and can
# drop to an unprivileged user.
USER lighthouse

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=20s --retries=3 \
    CMD wget -qO- "http://127.0.0.1:${PORT}/" > /dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
