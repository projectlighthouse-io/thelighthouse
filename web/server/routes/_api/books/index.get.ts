import type { ApiBookSummary } from '#server/utils/Lighthouse'
import { fromApi, priceLabel } from '#server/utils/Lighthouse'

/**
 * Every published book, from ohara by way of the rust api.
 *
 * The mapping here is the whole job: the api speaks snake case and minor units,
 * the page wants camel case and a price tag. Keeping that translation in one
 * handler means neither side has to know about the other's conventions.
 */
export default defineEventHandler(async () => {
  const books = await fromApi<ApiBookSummary[]>('/api/books')

  return books.map(book => ({
    slug: book.slug,
    title: book.title,
    description: book.description ?? '',
    thumbnailUrl: book.thumbnail_url ?? '',
    pages: book.lesson_count,
    price: priceLabel(book.price),
    firstLesson: book.first_lesson,
    // Nothing in ohara says a book is unfinished yet. Better a flat `false`
    // than a guess dressed up as data — see the note in the books page.
  }))
})
