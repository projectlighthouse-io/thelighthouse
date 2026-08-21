import { books } from '@/data/Books'
import { curriculum } from '@/data/Curriculum'
import { lessonFixture } from '#server/data/LessonFixture'
import { renderLesson } from '#server/utils/LessonBody'

/**
 * Chooses the body and renders it, server side.
 *
 * This is deliberately the shape the rust endpoint will have — the entitlement
 * check picks the fragment and only the chosen one crosses the wire, so a
 * frontend bug cannot reveal a paid body. Swapping this handler for a fetch to
 * rust in phase 4 leaves the page untouched.
 */
export default defineEventHandler((event) => {
  const bookSlug = getRouterParam(event, 'book')
  const lessonSlug = getRouterParam(event, 'lesson')

  const book = books.find(b => b.slug === bookSlug)
  const lessons = curriculum[bookSlug ?? '']?.lessons ?? []
  const index = lessons.findIndex(l => l.slug === lessonSlug)
  const lesson = lessons[index]

  if (!book || !lesson) {
    throw createError({ statusCode: 404, statusMessage: 'Lesson not found' })
  }

  const rendered = renderLesson(lessonFixture, lesson.locked ? 1 : Number.MAX_SAFE_INTEGER)
  const previous = lessons[index - 1]
  const next = lessons[index + 1]

  return {
    book: { slug: book.slug, title: book.title, thumbnailUrl: book.thumbnailUrl },
    lesson: {
      slug: lesson.slug,
      title: lesson.title,
      description: lesson.description,
      locked: lesson.locked,
    },
    ...rendered,
    position: index + 1,
    total: lessons.length,
    percent: Math.round(((index + 1) / Math.max(1, lessons.length)) * 100),
    previous: previous ? { slug: previous.slug, title: previous.title } : null,
    next: next ? { slug: next.slug, title: next.title } : null,
  }
})
