import type {
  ApiBookDetail,
  ApiBookSummary,
  ApiProjectPage,
  ApiProjectSummary,
} from '#server/utils/Lighthouse'
import { posts } from '@/data/Blog'
import { languages } from '@/data/Syntax'
import { fromApi } from '#server/utils/Lighthouse'

const SITE = 'https://projectlighthouse.io'

interface Entry {
  path: string
  priority: number
  changefreq: 'daily' | 'weekly' | 'monthly' | 'yearly'
  lastmod?: string
}

/**
 * Built from the same source the pages render from, so a book or a project
 * that exists on the site cannot be missing here. Private routes are absent by construction —
 * nothing in these lists is behind auth.
 *
 * Books come from the api rather than a static list, which is also why this
 * route is not prerendered: content changes on SIGHUP without a build, and a
 * sitemap baked at build time would advertise the previous set of lessons.
 */
async function entries(): Promise<Entry[]> {
  const out: Entry[] = [
    { path: '/', priority: 1.0, changefreq: 'weekly' },
    { path: '/books', priority: 0.9, changefreq: 'weekly' },
    { path: '/projects', priority: 0.9, changefreq: 'weekly' },
    { path: '/pricing', priority: 0.8, changefreq: 'monthly' },
    { path: '/blog', priority: 0.8, changefreq: 'weekly' },
    { path: '/syntax', priority: 0.7, changefreq: 'monthly' },
    { path: '/roadmap', priority: 0.5, changefreq: 'weekly' },
    { path: '/changelog', priority: 0.5, changefreq: 'weekly' },
    { path: '/connecting-the-dots', priority: 0.5, changefreq: 'monthly' },
    { path: '/support', priority: 0.3, changefreq: 'yearly' },
    { path: '/terms', priority: 0.2, changefreq: 'yearly' },
    { path: '/privacy', priority: 0.2, changefreq: 'yearly' },
  ]

  const books = await fromApi<ApiBookSummary[]>('/api/books')

  for (const summary of books) {
    out.push({ path: `/books/${summary.slug}`, priority: 0.9, changefreq: 'weekly' })

    const book = await fromApi<ApiBookDetail>(`/api/books/${summary.slug}`)

    for (const chapter of book.chapters) {
      for (const lesson of chapter.lessons) {
        // Paywalled lessons still get indexed — the free portion is real
        // content, and the page declares the gap in its structured data.
        out.push({ path: `/books/${summary.slug}/pages/${lesson.slug}`, priority: 0.7, changefreq: 'monthly' })
      }
    }
  }

  // Projects come from the api for the same reason books do: they change on
  // SIGHUP without a build, and a list typed in here goes stale the first time
  // one is published without somebody remembering this file.
  const projects = await fromApi<ApiProjectSummary[]>('/api/projects')

  for (const summary of projects) {
    out.push({ path: `/projects/${summary.slug}`, priority: 0.8, changefreq: 'monthly' })

    const project = await fromApi<ApiProjectPage>(`/api/projects/${summary.slug}`)

    for (const task of project.tasks) {
      // The brief is real content and worth indexing. What is sold is the
      // validation and the hints, and neither is on the page.
      out.push({
        path: `/projects/${summary.slug}/tasks/${task.slug}`,
        priority: 0.6,
        changefreq: 'monthly',
      })
    }
  }

  for (const post of posts) {
    out.push({ path: `/blog/${post.slug}`, priority: 0.7, changefreq: 'monthly', lastmod: post.publishedAt })
  }

  for (const lang of languages) {
    out.push({ path: `/syntax/${lang.slug}`, priority: 0.6, changefreq: 'monthly' })
  }

  return out
}

export default defineEventHandler(async (event) => {
  setHeader(event, 'content-type', 'application/xml; charset=utf-8')

  const urls = (await entries())
    .map(e => [
      '  <url>',
      `    <loc>${SITE}${e.path}</loc>`,
      e.lastmod ? `    <lastmod>${e.lastmod}</lastmod>` : '',
      `    <changefreq>${e.changefreq}</changefreq>`,
      `    <priority>${e.priority.toFixed(1)}</priority>`,
      '  </url>',
    ].filter(Boolean).join('\n'))
    .join('\n')

  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls}
</urlset>
`
})
