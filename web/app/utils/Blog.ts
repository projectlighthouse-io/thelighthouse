/**
 * Where a reader's own articles live.
 *
 * Not a page of its own any more: it is the public listing with `?author=`
 * set, and the api hands the author their taken down articles as well when the
 * name is theirs — see the api's `articles::mod`.
 *
 * `/blog` for a reader with no username, which is a row that predates the
 * column rather than a signed-out one.
 */
export function ownWritingUrl(username?: string | null): string {
  return username ? `/blog?author=${encodeURIComponent(username)}` : '/blog'
}

/** Cascading units for [`humanDate`], largest first. */
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 31536000],
  ['month', 2592000],
  ['week', 604800],
  ['day', 86400],
  ['hour', 3600],
  ['minute', 60],
]

const relative = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })

/**
 * A date as "3 days ago".
 *
 * `Intl.RelativeTimeFormat`, no date library: cascade down the units and hand
 * the largest whole one to the formatter. Anything unparseable comes back as
 * it went in, which is a date the reader can still read.
 */
export function humanDate(iso: string): string {
  const seconds = (Date.parse(iso) - Date.now()) / 1000
  if (Number.isNaN(seconds)) return iso

  for (const [unit, size] of UNITS) {
    if (Math.abs(seconds) >= size) return relative.format(Math.round(seconds / size), unit)
  }

  return relative.format(Math.round(seconds), 'second')
}
