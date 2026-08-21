import { check, sleep } from 'k6'
import http from 'k6/http'
import { Trend } from 'k6/metrics'

const BASE = __ENV.BASE || 'http://localhost:3000'
const routes = JSON.parse(open('./routes.json'))

// Separate trends per surface. An average across everything is meaningless
// here: prerendered HTML is a file read, an api route renders markdown, and
// mixing them hides which one is slow.
const ttfbStatic = new Trend('ttfb_static', true)
const ttfbLesson = new Trend('ttfb_lesson', true)
const ttfbApi = new Trend('ttfb_api', true)

/**
 * Weighted to look like real reading, not a uniform sweep. Someone lands on a
 * page, reads a lesson, follows a link — lessons dominate, listings less so.
 */
const MIX = [
  { weight: 45, name: 'lesson', pool: routes.lessons, trend: ttfbLesson },
  { weight: 15, name: 'book', pool: routes.books, trend: ttfbStatic },
  { weight: 12, name: 'static', pool: routes.static, trend: ttfbStatic },
  { weight: 10, name: 'project', pool: routes.projects, trend: ttfbStatic },
  { weight: 8, name: 'syntax', pool: routes.syntax, trend: ttfbApi },
  { weight: 6, name: 'blog', pool: routes.blog, trend: ttfbApi },
  { weight: 4, name: 'api', pool: ['/api/blog', '/api/syntax/go'], trend: ttfbApi },
]

const TOTAL = MIX.reduce((n, m) => n + m.weight, 0)

// no object spread: the k6 0.49 babel build rejects it here
function pick() {
  let r = Math.random() * TOTAL
  let hit = MIX[0]
  for (const m of MIX) {
    if ((r -= m.weight) < 0) {
      hit = m
      break
    }
  }
  return {
    name: hit.name,
    trend: hit.trend,
    url: hit.pool[Math.floor(Math.random() * hit.pool.length)],
  }
}

export const options = {
  scenarios: {
    // ramp rather than a flat wall, so the numbers show how it degrades
    browse: {
      executor: 'ramping-vus',
      startVUs: 1,
      stages: [
        { duration: '10s', target: 10 },
        { duration: '20s', target: 30 },
        { duration: '10s', target: 0 },
      ],
      gracefulRampDown: '5s',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<500'],
    ttfb_lesson: ['p(95)<300'],
  },
}

export default function () {
  const { url, name, trend } = pick()

  const res = http.get(`${BASE}${url}`, {
    tags: { surface: name },
    headers: { 'Accept': 'text/html,application/xhtml+xml', 'Accept-Encoding': 'gzip, br' },
  })

  trend.add(res.timings.waiting)

  check(res, {
    'status 200': r => r.status === 200,
    'has body': r => r.body && r.body.length > 500,
  })

  // readers pause; hammering with zero think time measures the load generator
  sleep(Math.random() * 0.4 + 0.1)
}

/**
 * k6 hands the whole summary here at the end of the run. Writing the raw JSON
 * alongside the HTML means the report is regenerable without re-running the
 * load, which matters when a run takes 40s and you only wanted to reword a
 * heading.
 */
export function handleSummary(data) {
  return {
    'bench/results/summary.json': JSON.stringify(data, null, 2),
    stdout: textSummary(data),
  }
}

function ms(v) {
  return v === undefined ? '—' : `${v.toFixed(1)}ms`
}

function textSummary(data) {
  const m = data.metrics
  const req = m.http_req_duration ? m.http_req_duration.values : {}
  const fail = m.http_req_failed ? m.http_req_failed.values.rate : 0
  const lines = [
    '',
    `  requests      ${m.http_reqs ? m.http_reqs.values.count : 0} (${(m.http_reqs ? m.http_reqs.values.rate : 0).toFixed(1)}/s)`,
    `  failed        ${(fail * 100).toFixed(2)}%`,
    `  duration      med ${ms(req.med)}  p95 ${ms(req['p(95)'])}  p99 ${ms(req['p(99)'])}  max ${ms(req.max)}`,
    '',
    '  time to first byte by surface',
  ]
  for (const key of ['ttfb_static', 'ttfb_lesson', 'ttfb_api']) {
    if (!m[key]) continue
    const v = m[key].values
    lines.push(`    ${key.replace('ttfb_', '').padEnd(8)} med ${ms(v.med)}  p95 ${ms(v['p(95)'])}  max ${ms(v.max)}`)
  }
  lines.push('')
  return lines.join('\n')
}
