import type { ApiProjectPage } from '#server/utils/Lighthouse'
import { fromApi } from '#server/utils/Lighthouse'

/**
 * One project's page.
 *
 * Carries its tasks by name, which the static data never could — the page used
 * to render "Task 1", "Task 2" from a count, because a count was all it had.
 *
 * No blueprint: the api's page endpoint does not serve one, a browser cannot
 * run one, and luxctl fetches its own from the signed surface.
 */
export default defineEventHandler(async (event) => {
  const slug = getRouterParam(event, 'slug')

  if (!slug) {
    throw createError({ statusCode: 400, statusMessage: 'No project named' })
  }

  const project = await fromApi<ApiProjectPage>(
    `/api/projects/${encodeURIComponent(slug)}`,
  )

  return {
    slug: project.slug,
    name: project.name,
    headline: project.headline ?? '',
    shortDescription: project.short_description ?? '',
    longDescription: project.long_description ?? '',
    difficulty: project.difficulty ?? '',
    tasksCount: project.task_count,
    isChallenge: project.is_challenge,
    unlockMode: project.unlock_mode,
    features: project.features.map(feature => ({
      title: feature.title,
      description: feature.description ?? '',
      icon: feature.icon ?? '',
    })),
    tasks: project.tasks.map(task => ({
      slug: task.slug,
      title: task.title,
      sortOrder: task.sort_order,
      points: task.points,
      isFree: task.is_free,
    })),
  }
})
