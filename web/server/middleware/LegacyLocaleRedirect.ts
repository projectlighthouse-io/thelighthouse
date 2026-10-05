import { stripLegacyLocale } from '#server/utils/LegacyLocale'

/**
 * The Laravel app served every page under /{locale}/. The rebuild dropped the
 * prefix, so those URLs are now dead — and they are indexed, linked from
 * newsletters, and sitting in people's bookmarks.
 *
 * 301, not 302. A permanent redirect passes ranking to the new URL and gets the
 * old one dropped from the index; a temporary one leaves both live and splits
 * the ranking between them. This runs as server middleware rather than a route
 * guard so crawlers get the redirect without executing any JS.
 */
export default defineEventHandler((event) => {
  const url = getRequestURL(event)
  const { path, locale } = stripLegacyLocale(url.pathname)

  if (!locale) return

  // The prefix was an explicit language choice, so honour it rather than
  // dropping every Bengali reader into English on their next visit.
  setCookie(event, 'locale', locale, {
    path: '/',
    maxAge: 60 * 60 * 24 * 365,
    sameSite: 'lax',
  })

  return sendRedirect(event, path + url.search, 301)
})
