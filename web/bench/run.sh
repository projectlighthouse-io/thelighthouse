#!/usr/bin/env bash
# Benchmarks the production container: cold start, first-load timings, then a
# ramped load run with CPU and memory sampled alongside it.
#
# Runs against the container rather than `nuxt dev` on purpose — dev has HMR,
# source maps and no prerendering, so its numbers say nothing about production.
set -euo pipefail

IMAGE=${IMAGE:-lighthouse-web:local}
NAME=lighthouse-bench
PORT=${BENCH_PORT:-3020}
BASE="http://localhost:${PORT}"
OUT=bench/results

mkdir -p "$OUT"
cleanup() { docker rm -f "$NAME" >/dev/null 2>&1 || true; }
trap cleanup EXIT
cleanup

echo "==> starting $IMAGE"
start_ns=$(date +%s%N)
docker run -d --name "$NAME" --platform linux/amd64 -p "${PORT}:3000" "$IMAGE" >/dev/null

# cold start: time until the first request is actually answered, not until the
# process exists — a listening socket that 500s is not "started"
until curl -fsS -o /dev/null "$BASE/" 2>/dev/null; do sleep 0.05; done
boot_ms=$(( ($(date +%s%N) - start_ns) / 1000000 ))
echo "    cold start: ${boot_ms}ms"

echo
echo "==> first load, uncached"
printf "    %-56s %8s %8s %10s\n" route ttfb total bytes
: > "$OUT/first-load.csv"
for path in / /books /books/networking-fundamentals \
            /books/networking-fundamentals/pages/what-is-a-network \
            /syntax/go /blog/why-hashmap-isnt-always-o1 /pricing; do
  read -r ttfb total bytes <<<"$(curl -s -o /dev/null \
    -H 'Accept: text/html' -H 'Accept-Encoding: gzip, br' \
    -w '%{time_starttransfer} %{time_total} %{size_download}' "$BASE$path")"
  t_ms=$(echo "$ttfb * 1000" | bc -l)
  tot_ms=$(echo "$total * 1000" | bc -l)
  printf "    %-56s %8.1f %8.1f %10s\n" "$path" "$t_ms" "$tot_ms" "$bytes"
  printf "%s,%s,%s,%s\n" "$path" "$t_ms" "$tot_ms" "$bytes" >> "$OUT/first-load.csv"
done

echo
echo "==> load, sampling cpu/memory"
: > "$OUT/stats.csv"
(
  while docker inspect -f '{{.State.Running}}' "$NAME" >/dev/null 2>&1; do
    docker stats --no-stream --format '{{.CPUPerc}},{{.MemUsage}}' "$NAME" 2>/dev/null \
      | tr -d '%' >> "$OUT/stats.csv" || true
  done
) &
STATS_PID=$!

BASE="$BASE" k6 run --summary-trend-stats='avg,min,med,p(95),p(99),max' \
  bench/load.js 2>&1 | tee "$OUT/k6.txt" | tail -20

kill "$STATS_PID" 2>/dev/null || true
wait "$STATS_PID" 2>/dev/null || true

echo "==> resources"
read -r CPU_AVG CPU_MAX MEM_AVG MEM_MAX SAMPLES <<<"$(awk -F, '
  { cpu=$1+0; split($2, m, " "); mem=m[1]+0
    if (index(m[1],"GiB")) mem*=1024
    n++; cpus+=cpu; if (cpu>maxc) maxc=cpu
    mems+=mem; if (mem>maxm) maxm=mem }
  END { if (!n) print "0 0 0 0 0"; else printf "%.2f %.2f %.2f %.2f %d", cpus/n, maxc, mems/n, maxm, n }
' "$OUT/stats.csv")"

printf "    cpu    avg %6.1f%%   peak %6.1f%%\n" "$CPU_AVG" "$CPU_MAX"
printf "    memory avg %6.1fMiB peak %6.1fMiB  (%s samples)\n" "$MEM_AVG" "$MEM_MAX" "$SAMPLES"

IMAGE_SIZE=$(docker images "$IMAGE" --format '{{.Size}}' | head -1)

IMAGE="$IMAGE" IMAGE_SIZE="$IMAGE_SIZE" BOOT_MS="$boot_ms" \
CPU_AVG="$CPU_AVG" CPU_MAX="$CPU_MAX" MEM_AVG="$MEM_AVG" MEM_MAX="$MEM_MAX" \
  node bench/meta.mjs

echo
node bench/report.mjs
echo "    raw:    $OUT/summary.json, $OUT/k6.txt, $OUT/stats.csv"
