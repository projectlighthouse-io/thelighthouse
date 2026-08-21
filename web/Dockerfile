# Pinned to a minor, not `26-alpine`. A floating base tag would undo the
# dependency pinning in package.json — the base image is the largest single
# supply-chain surface in the build.
#
# 26 rather than 24: a transitive dep wants >=24.15, and engine-strict in
# .npmrc turns that into a hard failure rather than a runtime surprise. This
# also matches the node used in development.
ARG NODE_VERSION=26.0-alpine

# deps
# Separated from the build so a source-only change reuses the install layer.
# npm ci, not install: exact lockfile, and it fails if the lockfile and
# package.json disagree.
FROM node:${NODE_VERSION} AS deps

WORKDIR /build
COPY package.json package-lock.json .npmrc ./

# --ignore-scripts blocks postinstall hooks, the usual delivery mechanism for a
# compromised package. Nuxt's own `nuxt prepare` is run explicitly in the build
# stage, where it belongs.
RUN npm ci --ignore-scripts

# build
FROM node:${NODE_VERSION} AS build

WORKDIR /build
COPY --from=deps /build/node_modules ./node_modules
COPY . .

ENV NODE_ENV=production
ENV NUXT_TELEMETRY_DISABLED=1

RUN npx nuxt prepare && npx nuxt build

# runtime
# Nitro bundles every runtime dependency into .output, so this stage needs no
# node_modules and no package.json — just node and the output directory. That is
# what keeps the final image small.
FROM node:${NODE_VERSION} AS runtime

# PID 1 gets no default signal handlers. Without an init, a SIGTERM from the
# platform is ignored and the container is SIGKILLed after the grace period,
# dropping in-flight requests on every deploy.
RUN apk add --no-cache tini

WORKDIR /app

ENV NODE_ENV=production
ENV NUXT_TELEMETRY_DISABLED=1
ENV HOST=0.0.0.0
ENV PORT=3000

COPY --from=build --chown=node:node /build/.output ./.output

# `node` exists in the base image; running as root inside a container is one
# less barrier between a code-execution bug and the host.
USER node

EXPOSE 3000

# App Platform runs its own probe, but this makes `docker ps` honest locally and
# gives any orchestrator something to read. wget ships with busybox in alpine.
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget -qO- http://127.0.0.1:3000/ > /dev/null || exit 1

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["node", ".output/server/index.mjs"]
