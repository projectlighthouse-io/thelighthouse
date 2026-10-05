/**
 * The rules every page's head follows, kept pure so they are tested
 * without rendering a page — `useSeo` applies them.
 */

/** Past this a results page truncates a description itself, mid-word. */
const DESCRIPTION_MAX = 160

/** Past this a results page truncates a title. */
const TITLE_MAX = 70

/**
 * The site's name as search results show it. Only in titles and cards:
 * everywhere else on the site — the wordmark, the copy — it stays lowercase.
 */
const TITLE_NAME = 'Project Lighthouse'

/** A site name a page already wrote at the end of its title, in any case,
 *  with or without the space, after an em dash, a hyphen or a middle dot. */
const WRITTEN_NAME = /\s*(?:—|-|·)?\s*project\s?lighthouse\s*$/i

/**
 * The site's name on the end of every title, once, spelled for results.
 * Pages write titles their own way; whatever they ended with is respelled
 * here rather than doubled.
 */
export function pageTitle(title: string): string {
  const bare = title.replace(WRITTEN_NAME, '').trim()

  return bare ? `${bare} · ${TITLE_NAME}` : TITLE_NAME
}

/**
 * A lesson's title: the book's name too while the whole thing, site name
 * included, still shows in a result; past that the lesson's own name is the
 * part worth keeping.
 */
export function lessonTitle(lesson: string, book: string): string {
  const full = `${lesson} — ${book}`

  return pageTitle(full).length <= TITLE_MAX ? full : lesson
}

/**
 * A description that fits a results page: whitespace collapsed, and anything
 * long cut at a word boundary with an ellipsis, rather than left for the search
 * engine to cut wherever it likes. Content descriptions — a book's blurb, a
 * post's subtitle — are written for the page, not for this limit.
 */
export function metaDescription(text: string): string {
  const flat = text.replace(/\s+/g, ' ').trim()
  if (flat.length <= DESCRIPTION_MAX) return flat

  const room = flat.slice(0, DESCRIPTION_MAX - 1)
  const lastSpace = room.lastIndexOf(' ')
  const cut = lastSpace > 0 ? room.slice(0, lastSpace) : room

  return `${cut.replace(/[\s,;:—–-]+$/, '')}…`
}
