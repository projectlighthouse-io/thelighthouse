import type { ApiProjectSummary } from '#server/utils/Lighthouse'
import { fromApi } from '#server/utils/Lighthouse'

/**
 * Every published project, from ohara by way of the rust api.
 *
 * The mapping is the whole job, as it is for books: the api speaks snake case
 * and the page wants camel case. Keeping that translation in one handler means
 * neither side has to know the other's conventions.
 *
 * The projects page splits these into two tabs itself — `isChallenge` is the
 * only thing it needs to do that, and which tab a project belongs in is a
 * content decision recorded in `project.yaml` rather than a list kept here.
 */
export default defineEventHandler(async () => {
  const projects = await fromApi<ApiProjectSummary[]>('/api/projects')

  return projects.map(project => ({
    slug: project.slug,
    name: project.name,
    shortDescription: project.short_description ?? '',
    tasksCount: project.task_count,
    isChallenge: project.is_challenge,
  }))
})
