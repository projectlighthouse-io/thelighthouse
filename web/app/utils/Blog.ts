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
