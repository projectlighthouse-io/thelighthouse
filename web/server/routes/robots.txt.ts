/**
 * Served from Nitro rather than public/ so the disallow list and the sitemap
 * URL live next to the routes they describe.
 */
const DISALLOW = [
  // the Rust api, and this app's own endpoints in front of it — data, not pages
  '/api/',
  '/_api/',
  '/settings/',
  '/dashboard',
  '/notes',
  '/profile',
  '/login',
  '/register',
  '/admin',
  '/billing/',
  '/blog/write-something-amazing',
  '/blog/edit/',
  '/checkout',
]

export default defineEventHandler((event) => {
  setHeader(event, 'content-type', 'text/plain; charset=utf-8')

  return `${[
    'User-agent: *',
    ...DISALLOW.map(path => `Disallow: ${path}`),
    '',
    'Sitemap: https://projectlighthouse.io/sitemap.xml',
  ].join('\n')}\n`
})
