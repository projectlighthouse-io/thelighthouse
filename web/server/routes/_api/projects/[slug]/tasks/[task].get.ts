import type { ApiTaskPage } from '#server/utils/Lighthouse'
import { fromApi } from '#server/utils/Lighthouse'

/**
 * One task's brief.
 *
 * Addressed within its project, unlike luxctl's `/api/v1/tasks/{identifier}`:
 * a url a reader can see should say which project they are in, and a task slug
 * is unique inside a project rather than across the repo.
 */
export default defineEventHandler(async (event) => {
  const slug = getRouterParam(event, 'slug')
  const task = getRouterParam(event, 'task')

  if (!slug || !task) {
    throw createError({ statusCode: 400, statusMessage: 'No task named' })
  }

  const page = await fromApi<ApiTaskPage>(
    `/api/projects/${encodeURIComponent(slug)}/tasks/${encodeURIComponent(task)}`,
  )

  return {
    slug: page.slug,
    title: page.title,
    sortOrder: page.sort_order,
    points: page.points,
    isFree: page.is_free,
    html: page.html,
    position: page.position,
    total: page.total,
    project: page.project,
    previous: page.previous,
    next: page.next,
  }
})
