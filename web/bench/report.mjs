#!/usr/bin/env node
// Renders bench/results/*.json into a single self-contained HTML page.
// Separate from the k6 run so the report can be regenerated without spending
// another 40 seconds of load to reword a heading.
import { readFileSync, writeFileSync } from 'node:fs'

const DIR = 'bench/results'

const read = (name, fallback) => {
  try {
    return JSON.parse(readFileSync(`${DIR}/${name}`, 'utf8'))
  }
  catch {
    return fallback
  }
}

const summary = read('summary.json', null)
const meta = read('meta.json', {})

if (!summary) {
  console.error('no bench/results/summary.json — run `make bench` first')
  process.exit(1)
}

const m = summary.metrics ?? {}
const v = key => m[key]?.values ?? {}
const ms = n => (n === undefined ? '—' : `${n.toFixed(1)}ms`)
const esc = s => String(s).replace(/[&<>]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]))

const req = v('http_req_duration')
const failRate = v('http_req_failed').rate ?? 0

// p95 against the thresholds declared in load.js
const verdict = (value, budget) =>
  value === undefined ? 'none' : value <= budget ? 'good' : value <= budget * 1.5 ? 'warn' : 'bad'

const surfaces = [
  ['static', 'prerendered html', v('ttfb_static'), 150],
  ['lesson', 'lesson pages', v('ttfb_lesson'), 300],
  ['api', 'api + markdown render', v('ttfb_api'), 400],
].filter(([, , values]) => values.med !== undefined)

const rows = (meta.firstLoad ?? [])
  .map(r => `<tr><td class="path">${esc(r.path)}</td><td>${r.ttfb.toFixed(1)}</td>`
    + `<td>${r.total.toFixed(1)}</td><td>${(r.bytes / 1024).toFixed(1)} KB</td></tr>`)
  .join('\n')

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>lighthouse-web benchmark</title>
<style>
  :root { color-scheme: light dark; --ink:#1b1b18; --quiet:#5a5a52; --rule:#e8e6df;
          --page:#fff; --panel:#fff; --good:#16a34a; --warn:#d97706; --bad:#dc2626; }
  @media (prefers-color-scheme: dark) {
    :root { --ink:#dbd7ce; --quiet:#979389; --rule:#2c2a26; --page:#121110; --panel:#1b1917; }
  }
  * { box-sizing: border-box; }
  body { margin:0; padding:48px 24px; background:var(--page); color:var(--ink);
         font:15px/1.6 ui-sans-serif, system-ui, sans-serif; }
  main { max-width: 860px; margin: 0 auto; }
  h1 { font-size:28px; margin:0 0 4px; letter-spacing:-0.01em; }
  h2 { font-size:15px; text-transform:uppercase; letter-spacing:.1em; color:var(--quiet);
       margin:40px 0 12px; font-weight:600; }
  .sub { color:var(--quiet); margin:0 0 8px; }
  .cards { display:grid; grid-template-columns:repeat(auto-fit,minmax(150px,1fr)); gap:12px; }
  .card { border:1px solid var(--rule); border-radius:8px; padding:14px 16px; background:var(--panel); }
  .card .k { font-size:12px; color:var(--quiet); text-transform:uppercase; letter-spacing:.08em; }
  .card .val { font-size:24px; font-variant-numeric:tabular-nums; margin-top:4px; }
  table { width:100%; border-collapse:collapse; font-variant-numeric:tabular-nums; }
  th, td { text-align:right; padding:8px 10px; border-bottom:1px solid var(--rule); }
  th:first-child, td:first-child { text-align:left; }
  th { font-size:12px; text-transform:uppercase; letter-spacing:.08em; color:var(--quiet); font-weight:600; }
  .path { font-family:ui-monospace, monospace; font-size:13px; }
  .pill { display:inline-block; padding:1px 8px; border-radius:99px; font-size:12px; font-weight:600; }
  .good { background:color-mix(in oklab, var(--good) 15%, transparent); color:var(--good); }
  .warn { background:color-mix(in oklab, var(--warn) 15%, transparent); color:var(--warn); }
  .bad  { background:color-mix(in oklab, var(--bad) 15%, transparent);  color:var(--bad); }
  .none { background:var(--rule); color:var(--quiet); }
  footer { margin-top:48px; color:var(--quiet); font-size:13px; }
</style>
</head>
<body><main>

<h1>lighthouse-web benchmark</h1>
<p class="sub">${esc(meta.generated ?? '')} · image <code>${esc(meta.image ?? 'lighthouse-web:local')}</code></p>

<h2>Headline</h2>
<div class="cards">
  <div class="card"><div class="k">requests</div><div class="val">${v('http_reqs').count ?? 0}</div></div>
  <div class="card"><div class="k">throughput</div><div class="val">${(v('http_reqs').rate ?? 0).toFixed(1)}<span style="font-size:14px">/s</span></div></div>
  <div class="card"><div class="k">p95 duration</div><div class="val">${ms(req['p(95)'])}</div></div>
  <div class="card"><div class="k">failed</div><div class="val">${(failRate * 100).toFixed(2)}%</div></div>
  <div class="card"><div class="k">cold start</div><div class="val">${meta.bootMs ? `${(meta.bootMs / 1000).toFixed(1)}s` : '—'}</div></div>
</div>

<h2>Latency by surface</h2>
<p class="sub">Averaging these together would hide which one is slow — a prerendered file read and a markdown render are not the same work.</p>
<table>
  <thead><tr><th>surface</th><th>median</th><th>p95</th><th>max</th><th>budget</th><th></th></tr></thead>
  <tbody>
${surfaces.map(([, label, values, budget]) => {
  const cls = verdict(values['p(95)'], budget)
  return `    <tr><td>${esc(label)}</td><td>${ms(values.med)}</td><td>${ms(values['p(95)'])}</td>`
    + `<td>${ms(values.max)}</td><td>${budget}ms</td>`
    + `<td><span class="pill ${cls}">${cls === 'good' ? 'within' : cls === 'warn' ? 'close' : 'over'}</span></td></tr>`
}).join('\n')}
  </tbody>
</table>

<h2>Resources</h2>
<div class="cards">
  <div class="card"><div class="k">cpu avg</div><div class="val">${meta.cpuAvg?.toFixed(1) ?? '—'}%</div></div>
  <div class="card"><div class="k">cpu peak</div><div class="val">${meta.cpuMax?.toFixed(1) ?? '—'}%</div></div>
  <div class="card"><div class="k">memory avg</div><div class="val">${meta.memAvg?.toFixed(0) ?? '—'}<span style="font-size:14px">MiB</span></div></div>
  <div class="card"><div class="k">memory peak</div><div class="val">${meta.memMax?.toFixed(0) ?? '—'}<span style="font-size:14px">MiB</span></div></div>
  <div class="card"><div class="k">image</div><div class="val">${esc(meta.imageSize ?? '—')}</div></div>
</div>

<h2>First load, cold</h2>
<p class="sub">One uncached request each, no concurrency. TTFB is the server; total includes transfer.</p>
<table>
  <thead><tr><th>route</th><th>ttfb (ms)</th><th>total (ms)</th><th>transferred</th></tr></thead>
  <tbody>
${rows || '    <tr><td colspan="4">no data</td></tr>'}
  </tbody>
</table>

<footer>
Generated from bench/results/summary.json. Re-render without re-running the load
with <code>node bench/report.mjs</code>.
</footer>

</main></body></html>
`

writeFileSync(`${DIR}/report.html`, html)
console.log(`    report: ${DIR}/report.html`)
