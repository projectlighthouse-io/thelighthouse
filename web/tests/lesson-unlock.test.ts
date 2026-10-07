/**
 * Why the lesson page asks for the paid half with `cache: 'no-store'`.
 *
 * The api answers one url two ways: the locked lesson to a reader with no
 * session, cacheable with `stale-while-revalidate`, and the whole lesson to an
 * entitled reader, `no-store`. A browser's http cache is keyed on the url, not
 * the cookie — so a browser that read the lesson signed out hands that locked
 * copy back to the signed-in request for the next five minutes, and a reader
 * who paid sees the paywall.
 *
 * The browser half runs a real Chrome against a local server that sends the
 * live api's headers, and is skipped unless `CHROME_PATH` is set, so `npm test`
 * needs no browser:
 *
 *   CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
 *     npx vitest run tests/lesson-unlock.test.ts
 */
import { execFile } from 'node:child_process'
import { mkdtempSync, readFileSync, rmSync } from 'node:fs'
import { createServer, type Server } from 'node:http'
import type { AddressInfo } from 'node:net'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

const CHROME = process.env.CHROME_PATH

/** What `crates/api` sends for an anonymous lesson. */
const ANONYMOUS_CACHE = 'public, max-age=0, s-maxage=60, stale-while-revalidate=300'

/**
 * Reads the lesson signed out, signs in, then asks again both ways. Each
 * answer is `unlocked` from the response the page was handed.
 */
const PAGE = `<!doctype html><pre id="out">pending</pre><script>
(async () => {
  const unlocked = async (init) => (await (await fetch('/api/lesson', init)).json()).unlocked
  const out = {}
  out.signedOut = await unlocked()
  document.cookie = 'session=reader'
  out.plain = await unlocked()
  out.noStore = await unlocked({ cache: 'no-store' })
  document.getElementById('out').textContent = JSON.stringify(out)
})()
</script>`

interface Answers { signedOut: boolean, plain: boolean, noStore: boolean }

function serve(): Server {
  return createServer((req, res) => {
    if (req.url === '/api/lesson') {
      const signedIn = (req.headers.cookie ?? '').includes('session=')
      res.setHeader('Content-Type', 'application/json')
      res.setHeader('Cache-Control', signedIn ? 'no-store' : ANONYMOUS_CACHE)
      res.end(JSON.stringify({ unlocked: signedIn }))
      return
    }

    res.setHeader('Content-Type', 'text/html')
    res.end(PAGE)
  })
}

/** Loads the page in a fresh headless Chrome profile and reads the answers. */
function run(chrome: string, url: string): Promise<Answers> {
  const profile = mkdtempSync(join(tmpdir(), 'lesson-unlock-'))

  return new Promise((resolve, reject) => {
    execFile(
      chrome,
      ['--headless=new', `--user-data-dir=${profile}`, '--virtual-time-budget=5000', '--dump-dom', url],
      { timeout: 30_000 },
      (error, stdout) => {
        rmSync(profile, { recursive: true, force: true })
        if (error) return reject(error)

        const json = /<pre id="out">(.*?)<\/pre>/s.exec(stdout)?.[1]
        if (!json || json === 'pending') return reject(new Error(`page never finished:\n${stdout}`))

        resolve(JSON.parse(json) as Answers)
      },
    )
  })
}

describe.skipIf(!CHROME)('a signed-in reader asking for the paid half, in a real browser', () => {
  let server: Server
  let answers: Answers

  beforeAll(async () => {
    server = serve()
    await new Promise<void>(done => server.listen(0, '127.0.0.1', done))
    const { port } = server.address() as AddressInfo
    answers = await run(CHROME as string, `http://127.0.0.1:${port}/`)
  }, 40_000)

  afterAll(() => {
    server?.close()
  })

  it('is locked while signed out', () => {
    expect(answers.signedOut).toBe(false)
  })

  it('gets the cached locked copy back from a plain fetch', () => {
    expect(answers.plain).toBe(false)
  })

  it('gets the whole lesson with cache: no-store', () => {
    expect(answers.noStore).toBe(true)
  })
})

describe('the lesson page', () => {
  it('asks for the paid half with cache: no-store', () => {
    const page = readFileSync(new URL('../app/pages/books/[slug]/pages/[lesson].vue', import.meta.url), 'utf8')
    const unlock = /async function unlock\(\)[\s\S]*?\n\}/.exec(page)?.[0] ?? ''

    expect(unlock).toContain('/api/books/')
    expect(unlock).toMatch(/cache:\s*'no-store'/)
  })
})
