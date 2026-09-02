import type { ApiLesson } from '#server/utils/Lighthouse'
import { fromApi } from '#server/utils/Lighthouse'

/**
 * A lesson, from ohara by way of the rust api.
 *
 * The url says `pages` and the api says `lessons`. That is not an oversight:
 * `/books/x/pages/y` is the address readers and search engines already have,
 * and the api path is internal and free to be named after the thing.
 *
 * **Only the free half ever arrives here.** The api decides which fragment to
 * send and sends exactly one, so no bug in nitro or in the page can reveal a
 * paid body — there is nothing to reveal. `remainingSections` is a count the
 * api computed from prose this process never saw.
 */
export default defineEventHandler(async (event) => {
  const bookSlug = getRouterParam(event, 'book')
  const lessonSlug = getRouterParam(event, 'lesson')

  const lesson = await fromApi<ApiLesson>(
    `/api/books/${bookSlug}/lessons/${lessonSlug}`,
  )

  return {
    book: {
      slug: lesson.book.slug,
      title: lesson.book.title,
      thumbnailUrl: lesson.book.thumbnail_url ?? '',
    },
    lesson: {
      slug: lesson.slug,
      title: lesson.title,
      description: lesson.description ?? '',
      // "There is more, and this reader cannot see it." Both halves of that
      // are the api's answer now: `has_paid_part` is a fact about the lesson
      // and `unlocked` is a fact about the reader, and the api needs the
      // session cookie to know the second — see the note on forwarding below.
      locked: lesson.has_paid_part && !lesson.unlocked,
    },
    html: lesson.html,
    toc: lesson.toc,
    readMinutes: lesson.read_minutes,
    remainingSections: lesson.remaining_sections,
    position: lesson.position,
    total: lesson.total,
    percent: lesson.percent,
    previous: lesson.previous,
    next: lesson.next,
  }
})
