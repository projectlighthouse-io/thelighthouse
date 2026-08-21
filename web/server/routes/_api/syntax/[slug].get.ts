import { languages } from '@/data/Syntax'
import { syntaxBodies } from '#server/data/SyntaxBodies'
import { renderLesson } from '#server/utils/LessonBody'

/**
 * The markdown for all three languages is ~100kb. Reading it here instead of
 * importing it into the page keeps it on the server: the browser receives the
 * rendered HTML for the one language it asked for, and nothing else.
 */
export default defineEventHandler((event) => {
  const slug = getRouterParam(event, 'slug')
  const lang = languages.find(l => l.slug === slug)

  if (!lang) {
    throw createError({ statusCode: 404, statusMessage: 'Language not found' })
  }

  const { html, toc, readMinutes } = renderLesson(syntaxBodies[lang.slug] ?? '', Number.MAX_SAFE_INTEGER)

  return { slug: lang.slug, name: lang.name, description: lang.description, html, toc, readMinutes }
})
