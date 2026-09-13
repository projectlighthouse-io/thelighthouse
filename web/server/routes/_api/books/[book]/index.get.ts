import type { ApiBookDetail } from '#server/utils/Lighthouse'
import { fromApi, isLocked, priceLabel } from '#server/utils/Lighthouse'

/**
 * One book and its curriculum.
 *
 * `locked` is `isLocked`, the same rule the reader page answers with — the two
 * must agree, and the last time each spelled it out for itself they did not.
 * There is no reader half here: this response is shared and cacheable at the
 * edge, so it cannot depend on who is asking, and the book page asks again from
 * the browser to soften it.
 *
 * Deliberately not gated on the book's price. Price decides what happens at
 * checkout; the markers decide what is withheld, and a book being free today
 * is a pricing decision that can change without the prose moving.
 *
 * The api works `has_paid_part` out while it builds its catalogue, so this
 * reads no markdown — it used to be left `false` precisely because answering
 * it here would have opened every lesson file to render one list.
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
        locked: isLocked(lesson),
      })),
    ),
  }
})
