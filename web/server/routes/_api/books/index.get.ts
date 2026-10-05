import type { ApiBookSummary } from '#server/utils/Lighthouse'
import { fromApi, priceLabel } from '#server/utils/Lighthouse'

/**
 * Every published book, from ohara by way of the rust api.
 *
 * The mapping here is the whole job: the api speaks snake case and minor units,
 * the page wants camel case and a price tag. Keeping that translation in one
 * handler means neither side has to know about the other's conventions.
 *
 * **`?track=` is forwarded, not applied here.** The api owns which books are on
 * a track and in what order, because that is what `book.yaml` states; filtering
 * a second time in this handler would be a second place for the answer to be
 * wrong. An unknown track is the api's `[]`, and reaches the caller as one.
 */
export default defineEventHandler(async (event) => {
  const { track } = getQuery(event)

  const books = await fromApi<ApiBookSummary[]>(
    '/api/books',
    typeof track === 'string' && track ? { track } : undefined,
  )

  return books.map(book => ({
    slug: book.slug,
    title: book.title,
    description: book.description ?? '',
    thumbnailUrl: book.thumbnail_url ?? '',
    // Defaulted rather than assumed: a book.yaml written before `tracks:`
    // existed still parses, and arrives here without the field.
    tracks: book.tracks ?? {},
    pages: book.lesson_count,
    price: priceLabel(book.price),
    firstLesson: book.first_lesson,
    // Nothing in ohara says a book is unfinished yet. Better a flat `false`
    // than a guess dressed up as data — see the note in the books page.
  }))
})
