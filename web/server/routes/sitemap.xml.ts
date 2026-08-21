import { books } from '@/data/Books'
import { posts } from '@/data/Blog'
import { curriculum } from '@/data/Curriculum'
import { challenges, projects } from '@/data/Projects'
import { languages } from '@/data/Syntax'

const SITE = 'https://projectlighthouse.io'

interface Entry {
  path: string
  priority: number
  changefreq: 'daily' | 'weekly' | 'monthly' | 'yearly'
  lastmod?: string
}

/**
 * Built from the same data the pages render from, so a book that exists on the
 * site cannot be missing here. Private routes are absent by construction —
 * nothing in these lists is behind auth.
 */
function entries(): Entry[] {
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

  for (const book of books) {
    out.push({ path: `/books/${book.slug}`, priority: 0.9, changefreq: 'weekly' })
    for (const lesson of curriculum[book.slug]?.lessons ?? []) {
      // locked lessons still get indexed — the free portion is real content and
      // the paywall is declared in the page's structured data
      out.push({ path: `/books/${book.slug}/pages/${lesson.slug}`, priority: 0.7, changefreq: 'monthly' })
    }
  }

  for (const project of [...projects, ...challenges]) {
    out.push({ path: `/projects/${project.slug}`, priority: 0.8, changefreq: 'monthly' })
  }

  for (const post of posts) {
    out.push({ path: `/blog/${post.slug}`, priority: 0.7, changefreq: 'monthly', lastmod: post.publishedAt })
  }

  for (const lang of languages) {
    out.push({ path: `/syntax/${lang.slug}`, priority: 0.6, changefreq: 'monthly' })
  }

  return out
}

export default defineEventHandler((event) => {
  setHeader(event, 'content-type', 'application/xml; charset=utf-8')

  const urls = entries()
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
