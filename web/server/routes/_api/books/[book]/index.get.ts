import type { ApiBookDetail } from '#server/utils/Lighthouse'
import { fromApi, priceLabel } from '#server/utils/Lighthouse'

/**
 * One book and its curriculum.
 *
 * `locked` is the page's word for "there is more behind a paywall here". Two
 * things have to be true for it: the lesson withholds something, and the book
 * costs money. A book priced at zero is given away in full, so a lock on one
 * would be a promise of a purchase that does not exist.
 *
 * The api works `has_paid_part` out while it builds its catalogue, so this
 * reads no markdown — it used to be left `false` precisely because answering
 * it here would have opened every lesson file to render one list.
 *
 * It is deliberately not per reader. This response is shared and cacheable at
 * the edge, so it cannot depend on who is asking; an entitled reader still sees
 * the lock here and the full prose when they open the lesson.
 */
export default defineEventHandler(async (event) => {
  const slug = getRouterParam(event, 'book')
  const book = await fromApi<ApiBookDetail>(`/api/books/${slug}`)

  return {
    book: {
      slug: book.slug,
      title: book.title,
      description: book.description ?? '',
      thumbnailUrl: book.thumbnail_url ?? '',
      pages: book.lesson_count,
      price: priceLabel(book.price),
      firstLesson: book.first_lesson,
      inProgress: false,
    },
    chapters: book.chapters.map(chapter => ({
      id: chapter.id,
      title: chapter.title,
    })),
    lessons: book.chapters.flatMap(chapter =>
      chapter.lessons.map(lesson => ({
        slug: lesson.slug,
        title: lesson.title,
        description: lesson.description ?? '',
        chapterId: chapter.id,
        locked: lesson.has_paid_part && book.price.amount > 0,
      })),
    ),
  }
})
