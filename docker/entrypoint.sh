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

# Blocks until the first process exits, whichever it is.
wait -n
echo "==> a process exited, stopping the container"
kill -TERM "$api" "$nuxt" "$caddy" 2>/dev/null || true
wait
exit 1
