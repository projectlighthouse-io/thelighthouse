#!/usr/bin/env bash
# Runs the three processes and ties their lifetimes together.
#
# One container with three processes is a deliberate trade: one deploy unit and
# no cross-container networking, at the cost of independent restarts. The rule
# that makes it safe is below — if any one of them dies, the whole container
# dies, so the platform restarts a known-good state rather than leaving a
# half-running site answering requests.
#
# bash, not sh: `wait -n` is what makes "first exit wins" a single line, and
# ash does not have it.
set -euo pipefail

API_PORT=${API_PORT:-9000}
NUXT_PORT=${NUXT_PORT:-3000}
export API_PORT NUXT_PORT

# The api's billing config, from what this STRIPE_SECRET_KEY's account holds.
# Read-only: it creates nothing. A pricing.yaml change that has not been
# applied to this account stops the container here, rather than booting an api
# that sells a price stripe does not hold — run `lighthouse-prices apply` in the
# console, then restart.
echo "==> resolving prices"
/usr/local/bin/lighthouse-prices resolve

# Pending migrations first, before anything serves. App Platform gives no
# console to run them by hand. The cost: a migration that fails stops the
# container here, under `set -e`, and the deploy never comes up — read the
# log, fix the migration, deploy again.
echo "==> migrating"
/usr/local/bin/lighthouse-migrate run

echo "==> api on :${API_PORT}"
/usr/local/bin/lighthouse-api &
api=$!

echo "==> nuxt on :${NUXT_PORT}"
HOST=127.0.0.1 PORT="${NUXT_PORT}" node /app/web/server/index.mjs &
nuxt=$!

echo "==> caddy on :${PORT:-8080}"
caddy run --config /etc/caddy/Caddyfile --adapter caddyfile &
caddy=$!

# Forward the platform's SIGTERM to all three so in-flight requests finish
# instead of being cut off. Without this they are SIGKILLed on every deploy.
terminate() {
	echo "==> shutting down"
	kill -TERM "$api" "$nuxt" "$caddy" 2>/dev/null || true
	wait
	exit 0
}
trap terminate TERM INT

# SIGHUP reaches this script, not the api — `docker kill -s HUP` signals pid 1.
# Trapping it matters twice over: it forwards the reload to the process that can
# act on it, and it stops bash doing its default thing on HUP, which is to die
# and take the container with it.
reload() {
	echo "==> rereading content"
	kill -HUP "$api" 2>/dev/null || true
}
trap reload HUP

# `wait -n` returns when a child exits *or* when a trapped signal arrives, and
# the two must not be confused: a reload would otherwise look like a process
# exiting and stop the container. So wake for any reason, then check whether one
# of the three actually died.
while true; do
	wait -n || true

	for pid in "$api" "$nuxt" "$caddy"; do
		if ! kill -0 "$pid" 2>/dev/null; then
			echo "==> a process exited, stopping the container"
			kill -TERM "$api" "$nuxt" "$caddy" 2>/dev/null || true
			wait
			exit 1
		fi
	done
done
