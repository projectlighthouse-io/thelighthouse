import type { ApiArticle } from '#server/utils/Lighthouse'

import { fromApi } from '#server/utils/Lighthouse'
import { readMinutesOf, renderArticle } from '#server/utils/SafeMarkdown'

/**
 * One article, rendered.
 *
 * `renderArticle` is the only thing that turns a reader's markdown into html,
 * and it sanitises — see `SafeMarkdown`. The raw `body` is deliberately *not*
 * in the response: the page has no use for it, and a field that is never sent
 * cannot be rendered unsanitised by a component that reaches for the wrong
 * one.
 *
 * A taken down article is a 404 from rust, and `fromApi` turns that into
 * nitro's — so it is indistinguishable from a slug that never existed.
 */
export default defineEventHandler(async (event) => {
  const slug = getRouterParam(event, 'slug')

  if (!slug) {
    throw createError({ statusCode: 404, statusMessage: 'Post not found' })
  }

  const article = await fromApi<ApiArticle>(`/api/articles/${encodeURIComponent(slug)}`)

  return {
    slug: article.slug,
    title: article.title,
    description: article.subtitle,
    publishedAt: (article.created_at ?? '').slice(0, 10),
    updatedAt: (article.updated_at ?? '').slice(0, 10),
    tags: article.topics,
    readMinutes: readMinutesOf(article.body),
    author: article.author,
    authorUsername: article.author_username,
    html: renderArticle(article.body),
  }
})
