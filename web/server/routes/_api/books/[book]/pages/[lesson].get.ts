import type { ApiLesson } from '#server/utils/Lighthouse'
import { fromApi, isLocked } from '#server/utils/Lighthouse'

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
  // Passed through, not checked: rust owns which languages a lesson is in, and
  // answers English for anything else.
  const { lang } = getQuery(event)

  const lesson = await fromApi<ApiLesson>(
    `/api/books/${bookSlug}/lessons/${lessonSlug}`,
    typeof lang === 'string' && lang ? { lang } : undefined,
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
      // Both halves are the api's answer here — this url is asked with the
      // session cookie, so `unlocked` is real rather than absent.
      locked: isLocked(lesson),
    },
    html: lesson.html,
    toc: lesson.toc,
    readMinutes: lesson.read_minutes,
    remainingSections: lesson.remaining_sections,
    chapterNumber: lesson.chapter_number,
    moreLessons: lesson.more_lessons,
    moreLessonChapters: lesson.more_lesson_chapters,
    position: lesson.position,
    total: lesson.total,
    percent: lesson.percent,
    previous: lesson.previous,
    next: lesson.next,
    locale: lesson.locale,
    locales: lesson.locales,
  }
})
