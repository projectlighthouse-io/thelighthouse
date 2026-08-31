import type { ProjectProgress, TaskProgress } from '@/types/Content'

/**
 * What a reader has done on a project, refreshed while they work.
 *
 * The reader is in their terminal, not on this page. They run `lux submit`,
 * and the tab they left open should catch up without being reloaded — that is
 * the whole feature.
 *
 * **A poll, deliberately.** Not SSE and not a websocket. A reader working
 * through a project produces an event every few minutes at most, so a
 * connection per open tab would sit idle almost all of the time, and it would
 * still have to be re-established after every deploy. One indexed query every
 * five seconds is cheaper to run and far cheaper to reason about.
 *
 * **Stopped while the tab is hidden.** A background tab left open overnight
 * would otherwise make seventeen thousand requests to watch nothing happen.
 * The Page Visibility API is what says whether anyone is looking, and coming
 * back fetches immediately rather than waiting out the interval — the reader
 * switched to this tab *because* they want to see what changed.
 *
 * Client only. There is nothing for SSR to render here: the first paint is the
 * project's tasks, and progress arrives a moment later.
 */

/** How often to ask, while somebody is watching. */
const EVERY = 5_000

export function useProjectProgress(slug: MaybeRefOrGetter<string>) {
  const progress = ref<ProjectProgress | null>(null)

  // Distinct from `progress === null`, which cannot tell "not signed in" from
  // "not asked yet" — and the difference is a progress bar flashing empty at
  // somebody who has finished the project.
  const loaded = ref<boolean>(false)

  let timer: ReturnType<typeof setInterval> | null = null

  /** One task's row, or `null` for a reader with nothing recorded. */
  const forTask = (task: string): TaskProgress | null =>
    progress.value?.tasks.find(row => row.slug === task) ?? null

  async function poll(): Promise<void> {
    const project = toValue(slug)

    if (!project) return

    try {
      progress.value = await $fetch<ProjectProgress>(
        `/api/projects/${encodeURIComponent(project)}/progress`,
      )
    }
    catch {
      // Including a 401: a signed-out reader has no progress, and the page is
      // perfectly readable without it. Left as `null` rather than zeroed, so
      // the template can tell the two apart.
      progress.value = null
    }
    finally {
      loaded.value = true
    }
  }

  function stop(): void {
    if (timer !== null) {
      clearInterval(timer)
      timer = null
    }
  }

  function start(): void {
    stop()
    void poll()
    timer = setInterval(() => void poll(), EVERY)
  }

  /**
   * Runs on every visibility change, and is what makes a hidden tab free.
   *
   * `stop()` rather than a flag checked inside the interval: an interval that
   * fires and returns early is still a wake-up, and a laptop closing its lid
   * should mean nothing is scheduled at all.
   */
  function onVisibility(): void {
    if (document.visibilityState === 'visible') {
      start()
    }
    else {
      stop()
    }
  }

  onMounted(() => {
    document.addEventListener('visibilitychange', onVisibility)
    onVisibility()
  })

  onBeforeUnmount(() => {
    document.removeEventListener('visibilitychange', onVisibility)
    stop()
  })

  // A reader navigating between two projects keeps the same component, so the
  // slug changing has to restart the poll — otherwise the second project's
  // page shows the first project's progress until the next tick.
  watch(
    () => toValue(slug),
    () => {
      progress.value = null
      loaded.value = false

      if (document.visibilityState === 'visible') {
        start()
      }
    },
  )

  return { progress, loaded, forTask, refresh: poll }
}
