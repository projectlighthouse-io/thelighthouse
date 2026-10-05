/** Locales the Laravel app served under a URL prefix. */
export const LEGACY_LOCALES = ['en', 'bn'] as const

export type LegacyLocale = (typeof LEGACY_LOCALES)[number]

const isLegacyLocale = (segment: string): segment is LegacyLocale =>
  (LEGACY_LOCALES as readonly string[]).includes(segment)

export interface StripResult {
  /** the path with every leading locale segment removed */
  path: string
  /** the first locale seen, so the reader's language choice survives */
  locale: LegacyLocale | null
}

/**
 * Removes leading locale segments from a pathname.
 *
 * Strips all of them in one pass rather than one per request — `/en/bn/books`
 * becomes `/books` directly instead of a redirect chain, which search engines
 * treat worse than a single hop.
 *
 * Returns locale: null when there was nothing to strip, which is the caller's
 * signal to leave the request alone.
 */
export function stripLegacyLocale(pathname: string): StripResult {
  const segments = pathname.split('/').filter(Boolean)

  let i = 0
  while (i < segments.length && isLegacyLocale(segments[i] as string)) i++

  if (i === 0) return { path: pathname, locale: null }

  const rest = segments.slice(i)
  // a trailing slash on the original is not worth preserving; '/en/' -> '/'
  return {
    path: rest.length > 0 ? `/${rest.join('/')}` : '/',
    locale: segments[0] as LegacyLocale,
  }
}
