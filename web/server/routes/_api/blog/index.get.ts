import type { ApiArticle, ApiPage } from '#server/utils/Lighthouse'

import { fromApi } from '#server/utils/Lighthouse'
import { readMinutesOf } from '#server/utils/SafeMarkdown'

/**
 * The blog listing, from the database rather than a file.
 *
 * Bodies are fetched but never returned: the card needs the subtitle and a
 * read time, and shipping every article's full markdown to render a list of
 * titles is the thing `[slug].get.ts` exists to avoid.
 *
 * `description` is the author's subtitle rather than a summary derived from
 * the body — that is the whole reason the column exists.
 *
 * `?topic=` is passed through to rust, which refuses a topic that is not one
 * of its own — so a bad filter is a 400 here rather than a silent listing of
 * everything.
 *
 * `?author=` is passed through too, and it is the one parameter that makes
 * this answer depend on who is asking: rust hands an author reading their own
 * shelf their taken down articles as well. That is why the session cookie is
 * forwarded for it, and why the answer is then `no-store` — see the api's
 * `articles::mod`. Nothing here decides who may see what; it only carries the
 * cookie that lets rust decide.
 */
/**
 * A query parameter as a number, or `fallback`.
 *
 * Rust clamps paging itself, but it clamps *numbers* — `?page=3'` is a serde
 * failure and a 400 before any of that runs. So the digits are taken here and
 * anything else is the default, which makes a mangled link a first page rather
 * than an error page.
 */
function number(value: unknown, min: number, max: number, fallback: number): string {
  const parsed = Number.parseInt(String(value ?? ''), 10)

  return String(Number.isNaN(parsed) ? fallback : Math.min(max, Math.max(min, parsed)))
}

export default defineEventHandler(async (event) => {
  const { topic, author, page, per_page: perPage } = getQuery(event)
  const byAuthor = typeof author === 'string' && author ? author : null

  if (byAuthor) {
    // The answer may carry one reader's taken down articles. Nothing shared
    // may hold it — not the edge, not the browser.
    setHeader(event, 'cache-control', 'private, no-store')
  }

  const answer = await fromApi<ApiPage<ApiArticle>>(
    '/api/articles',
    {
      ...(typeof topic === 'string' && topic ? { topic } : {}),
      ...(byAuthor ? { author: byAuthor } : {}),
      page: number(page, 1, 100_000, 1),
      // Rust caps this at its own, smaller maximum — this only keeps the value
      // a number it will accept.
      per_page: number(perPage, 1, 50, 15),
    },
    // Only when an author was asked about: every other listing is the same
    // bytes for everybody, and forwarding a cookie would make rust say so.
    byAuthor ? { cookie: getHeader(event, 'cookie') ?? '' } : undefined,
  )

  return {
    ...answer,
    items: answer.items.map(article => ({
      slug: article.slug,
      title: article.title,
      description: article.subtitle,
      publishedAt: (article.created_at ?? '').slice(0, 10),
      tags: article.topics,
      readMinutes: readMinutesOf(article.body),
      author: article.author,
      authorUsername: article.author_username,
      // Only ever set on the author's own listing — rust leaves both columns
      // off every other shape, so `null` here means "live", and `undefined`
      // means nobody asked as the author.
      takenDownAt: article.taken_down_at ?? null,
      takenDownReason: article.taken_down_reason ?? null,
    })),
  }
})
