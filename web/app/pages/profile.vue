<script setup lang="ts">
import type { Project, ProjectProgress } from '@/types/Content'

definePageMeta({ middleware: 'auth' })

useSeo({
  title: 'Progress — projectlighthouse',
  description: 'Where you are in each project.',
  noindex: true,
})

// The route guard already resolved the session to let this page render, so
// this reads the state rather than asking again.
const { reader } = useReader()

interface Row {
  project: Project
  progress: ProjectProgress
}

/**
 * Every project the reader has started, with how far along each is.
 *
 * Browser only, like the rest of this page — one progress request per project,
 * asked once rather than polled. A project with nothing run and nothing done
 * is not started and is left off.
 */
const { data: rows, pending } = await useAsyncData('progress-rows', async () => {
  const projects = await $fetch<Project[]>('/_api/projects').catch(() => [] as Project[])

  const answers = await Promise.all(projects.map(project =>
    $fetch<ProjectProgress>(`/api/projects/${encodeURIComponent(project.slug)}/progress`)
      .then(progress => ({ project, progress }))
      .catch(() => null)))

  return answers.filter((row): row is Row =>
    row !== null && (row.progress.completed > 0 || row.progress.run > 0))
}, { server: false })

const ticks = (progress: ProjectProgress): boolean[] =>
  Array.from({ length: progress.total }, (_, i) => i < progress.completed)
</script>

<template>
  <AccountShell
    title="Progress"
    :sub="reader ? `${reader.name} · signed in with ${reader.provider}` : undefined"
  >
    <p v-if="pending" class="lh-sub">Loading…</p>

    <ul v-else-if="rows?.length" class="rows">
      <li v-for="row in rows" :key="row.project.slug">
        <NuxtLink :to="`/projects/${row.project.slug}`" class="row">
          <span class="top">
            <span class="lh-h3">{{ row.project.name }}</span>
            <span class="lh-mono lh-muted">{{ row.project.isChallenge ? 'challenge' : 'project' }}</span>
          </span>
          <span class="lh-ticks" aria-hidden="true">
            <span v-for="(done, i) in ticks(row.progress)" :key="i" :class="{ 'is-done': done }" />
          </span>
          <span class="lh-mono lh-muted lh-num">
            {{ row.progress.completed }} of {{ row.progress.total }} done · {{ row.progress.points_earned }} points
          </span>
        </NuxtLink>
      </li>
    </ul>

    <EmptyState
      v-else
      class="lh-card"
      eyebrow="nothing started"
      heading="Which one will you build?"
      detail="Start a project or a challenge and its progress shows up here."
      :action="{ label: 'See the projects', to: '/projects' }"
    />
  </AccountShell>
</template>

<style scoped>
.rows {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: var(--space-2);
}

.row {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-5) var(--space-6);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  color: var(--ink);
  text-decoration: none;
  transition: background-color var(--duration) var(--ease-out);
}

.row:hover { background: var(--surface-sunken); color: var(--ink); }

.top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-4);
}

</style>
