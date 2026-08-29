import type { ApiBookDetail } from '#server/utils/Lighthouse'
import { fromApi, priceLabel } from '#server/utils/Lighthouse'

/**
 * One book and its curriculum.
 *
 * `locked` is the page's word for "there is more behind a paywall here", and
 * it is the lesson's own `has_paid_part` — the same rule the reader page uses.
 * The two must agree: a contents list calling a lesson free while opening it
 * shows a paywall is worse than either answer on its own.
 *
 * Deliberately not gated on the book's price. Price decides what happens at
 * checkout; the markers decide what is withheld, and a book being free today
 * is a pricing decision that can change without the prose moving.
 *
 * The api works `has_paid_part` out while it builds its catalogue, so this
 * reads no markdown — it used to be left `false` precisely because answering
 * it here would have opened every lesson file to render one list.
 *
 * It is deliberately not per reader either. This response is shared and
 * cacheable at the edge, so it cannot depend on who is asking.
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
      // In order, and the page shows them in order. Empty for a book with
      // none, which is a page with no slideshow rather than an empty frame.
      images: book.images ?? [],
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
        locked: lesson.has_paid_part,
      })),
    ),
  }
})
