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

# What the footer prints — see web/nuxt.config.ts. Passed by `make image`; a
# build without them shows "dev" and no commit rather than failing.
ARG APP_VERSION=dev
ARG GIT_COMMIT=
ENV APP_VERSION=${APP_VERSION}
ENV GIT_COMMIT=${GIT_COMMIT}

RUN npx nuxt prepare && npx nuxt build

# api build
FROM rust:${RUST_VERSION} AS api-build

RUN apk add --no-cache musl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
# `sqlx::migrate!` embeds this directory into lighthouse-migrate at compile time.
COPY migrations ./migrations

# Cache mounts rather than a stub-manifest layer: the workspace has several
# crates and every one needs a real manifest before cargo will resolve it. The
# mounted target dir does not survive into the layer, hence the copy out.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --locked -p lighthouse-api -p lighthouse-migrate -p lighthouse-prices \
    && cp target/release/lighthouse-api target/release/lighthouse-migrate target/release/lighthouse-prices /usr/local/bin/

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

# Content and the price declaration are baked in: App Platform has no volumes,
# and the api refuses to boot without either. `ohara` is a named build context,
# not part of this repo — `make image` passes it with --build-context.
#
# billing.yaml is not baked. It holds stripe's ids, which differ between the
# test and live accounts, so the entrypoint writes it at start with
# `lighthouse-prices resolve`, from pricing.yaml and whichever account
# STRIPE_SECRET_KEY belongs to. `lighthouse-prices status` and `apply` are in
# the image for the console.
ENV CONTENT_PATH=/app/content
ENV PRICING=/app/pricing.yaml
ENV BILLING_PLANS=/app/billing.yaml
COPY --from=ohara books /app/content/books
COPY --from=ohara projects /app/content/projects
COPY pricing.yaml /app/pricing.yaml

COPY Caddyfile /etc/caddy/Caddyfile
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY --from=api-build /usr/local/bin/lighthouse-api /usr/local/bin/lighthouse-api
COPY --from=api-build /usr/local/bin/lighthouse-migrate /usr/local/bin/lighthouse-migrate
COPY --from=api-build /usr/local/bin/lighthouse-prices /usr/local/bin/lighthouse-prices
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
