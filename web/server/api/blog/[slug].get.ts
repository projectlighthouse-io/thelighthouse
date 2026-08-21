import { posts } from '@/data/Blog'
import { blogBodies } from '#server/data/BlogBodies'
import { renderLesson } from '#server/utils/LessonBody'

export default defineEventHandler((event) => {
  const slug = getRouterParam(event, 'slug')
  const post = posts.find(p => p.slug === slug)

  if (!post) {
    throw createError({ statusCode: 404, statusMessage: 'Post not found' })
  }

  const { html } = renderLesson(blogBodies[post.slug] ?? '', Number.MAX_SAFE_INTEGER)

  return { ...post, html }
})
