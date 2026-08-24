import type { ApiBookDetail } from '#server/utils/Lighthouse'
import { fromApi, priceLabel } from '#server/utils/Lighthouse'

/**
 * One book and its curriculum.
 *
 * `locked` is the page's word for "there is more behind a paywall", and it is
 * deliberately *not* answered here. Knowing it means reading every lesson's
 * markdown, which would make a table of contents open seventeen files. The
 * lesson endpoint answers it for the lesson actually being read.
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
        locked: false,
      })),
    ),
  }
})
