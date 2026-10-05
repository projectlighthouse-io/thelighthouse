/**
 * Every page's search metadata, read from the html a crawler actually gets.
 *
 * Against a running server rather than mounted components: what matters is the
 * server-rendered `<head>`, after layouts, `useSeo`, `useJsonLd` and route
 * middleware have all had their say — the one thing a component test cannot
 * see. Skipped unless `SEO_BASE_URL` is set, so `npm test` needs no server:
 *
 *   SEO_BASE_URL=http://localhost:3000 npm run test:seo
 *
 * Dynamic routes are tested on one real row each, fetched from the same `/_api`
 * the pages render from, so a book or project that exists is the one checked.
 */
import { beforeAll, describe, expect, it } from 'vitest'

const BASE = process.env.SEO_BASE_URL
const SITE = 'https://projectlighthouse.io'

/** Pages a search engine should list. */
const PUBLIC_STATIC = [
  '/',
  '/books',
  '/projects',
  '/pricing',
  '/blog',
  '/syntax',
  '/changelog',
  '/connecting-the-dots',
  '/newsletter',
  '/support',
  '/terms',
  '/privacy',
]

/** Pages that render for anybody but must stay out of the index. */
const PRIVATE_RENDERED = ['/login', '/register']

/** Behind a session: they redirect a crawler away, or render noindex. */
const PRIVATE_GUARDED = [
  '/dashboard',
  '/notes',
  '/profile',
  '/settings/profile',
  '/settings/public-profile',
  '/settings/tokens',
  '/settings/billing',
  '/blog/write-something-amazing',
  '/admin',
  '/billing/thanks',
]

interface Head {
  status: number
  html: string
  titles: string[]
  meta: (key: string) => string | undefined
  canonical?: string
  lang?: string
  h1s: number
  jsonLd: string[]
  /** `X-Robots-Tag` — how a page rendered only in the browser says noindex,
   *  since the html a crawler gets has no head of its own. */
  robotsHeader: string | null
}

/** The attributes of every `<tag ...>` in `html`, as maps. */
function tags(html: string, tag: string): Record<string, string>[] {
  const found: Record<string, string>[] = []

  for (const [, attrs] of html.matchAll(new RegExp(`<${tag}\\b([^>]*)>`, 'gi'))) {
    const map: Record<string, string> = {}
    for (const [, name, value] of (attrs ?? '').matchAll(/([\w:-]+)="([^"]*)"/g)) {
      if (name) map[name.toLowerCase()] = decode(value ?? '')
    }
    found.push(map)
  }

  return found
}

function decode(text: string): string {
  return text
    .replace(/&amp;/g, '&')
    .replace(/&quot;/g, '"')
    .replace(/&#39;|&#x27;/g, '\'')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
}

async function head(path: string): Promise<Head> {
  const response = await fetch(`${BASE}${path}`, { redirect: 'manual' })
  const html = await response.text()
  const metas = tags(html, 'meta')

  return {
    status: response.status,
    html,
    titles: [...html.matchAll(/<title[^>]*>([^<]*)<\/title>/gi)].map(m => decode(m[1] ?? '')),
    meta: key => metas.find(m => m.name === key || m.property === key)?.content,
    canonical: tags(html, 'link').find(l => l.rel === 'canonical')?.href,
    lang: tags(html, 'html')[0]?.lang,
    h1s: [...html.matchAll(/<h1\b/gi)].length,
    jsonLd: [...html.matchAll(/<script[^>]*type="application\/ld\+json"[^>]*>([\s\S]*?)<\/script>/gi)]
      .map(m => m[1] ?? ''),
    robotsHeader: response.headers.get('x-robots-tag'),
  }
}

async function json<T>(path: string): Promise<T> {
  const response = await fetch(`${BASE}${path}`)

  return response.json() as Promise<T>
}

/** One real path for each dynamic route. */
async function dynamicPaths(): Promise<string[]> {
  const books = await json<{ slug: string }[]>('/_api/books')
  const book = books[0]?.slug
  const detail = book ? await json<{ lessons: { slug: string }[] }>(`/_api/books/${book}`) : null
  const lesson = detail?.lessons[0]?.slug

  const projects = await json<{ slug: string }[]>('/_api/projects')
  const project = projects[0]?.slug
  const tasks = project ? await json<{ tasks: { slug: string }[] }>(`/_api/projects/${project}`) : null
  const task = tasks?.tasks[0]?.slug

  const blog = await json<{ items: { slug: string, authorUsername: string | null }[] }>('/_api/blog')
  const post = blog.items[0]

  return [
    book && `/books/${book}`,
    book && lesson && `/books/${book}/pages/${lesson}`,
    project && `/projects/${project}`,
    project && task && `/projects/${project}/tasks/${task}`,
    post && `/blog/${post.slug}`,
    post?.authorUsername && `/users/@${post.authorUsername}`,
    '/syntax/go',
  ].filter((path): path is string => typeof path === 'string')
}

/** Whether a page tells crawlers to stay out, by meta tag or by header. */
function saysNoindex(found: Head): boolean {
  return found.meta('robots') === 'noindex, follow'
    || (found.robotsHeader ?? '').includes('noindex')
}

describe.runIf(BASE)('seo', () => {
  let publicPaths: string[] = []
  const heads = new Map<string, Head>()

  beforeAll(async () => {
    publicPaths = [...PUBLIC_STATIC, ...await dynamicPaths()]

    for (const path of [...publicPaths, ...PRIVATE_RENDERED, ...PRIVATE_GUARDED]) {
      heads.set(path, await head(path))
    }
  }, 120_000)

  const page = (path: string): Head => {
    const found = heads.get(path)
    if (!found) throw new Error(`not fetched: ${path}`)

    return found
  }

  it('covers a real row of every dynamic route', () => {
    for (const prefix of ['/books/', '/projects/', '/blog/', '/users/@', '/syntax/']) {
      expect(publicPaths.some(p => p.startsWith(prefix)), prefix).toBe(true)
    }
    expect(publicPaths.some(p => p.includes('/pages/')), 'a lesson').toBe(true)
    expect(publicPaths.some(p => p.includes('/tasks/')), 'a task').toBe(true)
  })

  describe('every public page', () => {
    it('renders on the server with a 200', () => {
      for (const path of publicPaths) expect.soft(page(path).status, path).toBe(200)
    })

    it('has exactly one title, named for the site and short enough to show whole', () => {
      for (const path of publicPaths) {
        const { titles } = page(path)

        expect.soft(titles, path).toHaveLength(1)
        expect.soft(titles[0], path).toMatch(/projectlighthouse/)
        expect.soft(titles[0]?.length, `${path}: "${titles[0]}"`).toBeLessThanOrEqual(70)
      }
    })

    it('has a description a results page will show without cutting it to nothing', () => {
      for (const path of publicPaths) {
        const description = page(path).meta('description') ?? ''

        expect.soft(description.length, `${path}: "${description}"`).toBeGreaterThanOrEqual(50)
        expect.soft(description.length, `${path}: "${description}"`).toBeLessThanOrEqual(170)
      }
    })

    it('names itself as the canonical url', () => {
      for (const path of publicPaths) {
        expect.soft(page(path).canonical, path).toBe(`${SITE}${path === '/' ? '' : path}`)
      }
    })

    it('says it may be indexed', () => {
      for (const path of publicPaths) expect.soft(page(path).meta('robots'), path).toBe('index, follow')
    })

    it('carries open graph and twitter cards that point back at it', () => {
      for (const path of publicPaths) {
        const { meta, canonical } = page(path)

        expect.soft(meta('og:title'), path).toBeTruthy()
        expect.soft(meta('og:description'), path).toBeTruthy()
        expect.soft(meta('og:type'), path).toMatch(/^(website|article)$/)
        expect.soft(meta('og:url'), path).toBe(canonical)
        expect.soft(meta('og:image'), path).toMatch(/^https:\/\//)
        expect.soft(meta('twitter:card'), path).toBe('summary_large_image')
      }
    })

    it('declares its language', () => {
      for (const path of publicPaths) expect.soft(page(path).lang, path).toBeTruthy()
    })

    it('has exactly one h1', () => {
      for (const path of publicPaths) expect.soft(page(path).h1s, path).toBe(1)
    })

    it('has structured data that parses', () => {
      for (const path of publicPaths) {
        for (const block of page(path).jsonLd) {
          expect.soft(() => JSON.parse(block), path).not.toThrow()
          expect.soft(JSON.parse(block)['@context'], path).toBe('https://schema.org')
        }
      }
    })
  })

  it('credits a blog post to the person who wrote it', async () => {
    const blog = await json<{ items: { slug: string, author: string }[] }>('/_api/blog')
    const post = blog.items[0]
    if (!post) return

    const blocks = page(`/blog/${post.slug}`).jsonLd.map(b => JSON.parse(b) as Record<string, unknown>)
    const posting = blocks.find(b => b['@type'] === 'BlogPosting') as { author?: { name?: string } } | undefined

    expect(posting?.author?.name).toBe(post.author)
  })

  it('gives no two public pages the same title', () => {
    const seen = new Map<string, string>()

    for (const path of publicPaths) {
      const title = page(path).titles[0] ?? ''
      expect.soft(seen.get(title), `${path} repeats the title of ${seen.get(title)}`).toBeUndefined()
      seen.set(title, path)
    }
  })

  it('gives no two public pages the same description', () => {
    const seen = new Map<string, string>()

    for (const path of publicPaths) {
      const description = page(path).meta('description') ?? ''
      expect.soft(seen.get(description), `${path} repeats the description of ${seen.get(description)}`).toBeUndefined()
      seen.set(description, path)
    }
  })

  it('keeps sign-in out of the index', () => {
    for (const path of PRIVATE_RENDERED) {
      expect.soft(page(path).meta('robots'), path).toBe('noindex, follow')
    }
  })

  it('never offers a private page to the index', () => {
    for (const path of PRIVATE_GUARDED) {
      const { status } = page(path)

      // A redirect has nothing to index; anything that renders says noindex.
      if (status >= 300 && status < 400) continue
      expect.soft(saysNoindex(page(path)), path).toBe(true)
    }
  })

  it('lists every public page in the sitemap', async () => {
    const sitemap = await (await fetch(`${BASE}/sitemap.xml`)).text()

    for (const path of publicPaths) {
      expect.soft(sitemap, path).toContain(`<loc>${SITE}${path === '/' ? '' : path}</loc>`)
    }
  })

  it('lists no private page in the sitemap', async () => {
    const sitemap = await (await fetch(`${BASE}/sitemap.xml`)).text()

    for (const path of [...PRIVATE_RENDERED, ...PRIVATE_GUARDED]) {
      expect.soft(sitemap, path).not.toContain(`${SITE}${path}<`)
    }
  })

  it('points robots.txt at the sitemap and away from private pages', async () => {
    const robots = await (await fetch(`${BASE}/robots.txt`)).text()

    expect.soft(robots).toContain(`Sitemap: ${SITE}/sitemap.xml`)
    for (const path of [
      '/settings/', '/dashboard', '/notes', '/profile', '/login', '/register', '/admin',
      '/billing/', '/blog/write-something-amazing', '/blog/edit/',
    ]) {
      expect.soft(robots, path).toContain(`Disallow: ${path}`)
    }
  })
})
