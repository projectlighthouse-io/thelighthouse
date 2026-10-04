<script setup lang="ts">
import type { Project } from '@/types/Content'

/**
 * A project or a challenge in the build-and-drill grid. What a project is
 * counted in — stages — and what a challenge is counted in — tasks — is the
 * only difference between the two.
 */
const props = defineProps<{ project: Project }>()

const kind = computed<string>(() => (props.project.isChallenge ? 'challenge' : 'project'))

const meta = computed<string>(() => {
  const n = props.project.tasksCount
  const unit = props.project.isChallenge ? 'task' : 'stage'

  return `${n} ${unit}${n === 1 ? '' : 's'}`
})
</script>

<template>
  <NuxtLink :to="`/projects/${project.slug}`" class="card">
    <span class="top">
      <span class="name">{{ project.name }}</span>
      <span class="kind" :class="`is-${kind}`">{{ kind }}</span>
    </span>
    <span class="learn">{{ project.shortDescription }}</span>
    <span class="bottom">
      <span class="lh-num">{{ meta }}</span>
      <span aria-hidden="true">→</span>
    </span>
  </NuxtLink>
</template>

<style scoped>
.card {
  display: grid;
  gap: var(--space-5);
  align-content: start;
  padding: 28px var(--space-8);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  color: var(--ink);
  text-decoration: none;
  transition: background-color var(--duration) var(--ease-out);
}

.card:hover { background: var(--surface-sunken); color: var(--ink); }

.top {
  display: flex;
  align-items: baseline;
  gap: var(--space-4);
}

.name {
  font: var(--text-h3);
  letter-spacing: var(--tracking-title);
  min-width: 0;
}

.kind {
  margin-left: auto;
  font: var(--text-label-mono);
  white-space: nowrap;
}

.is-project { color: var(--accent-strong); }
.is-challenge { color: var(--ink-faint); }

.learn {
  font: var(--text-body-sm);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

.bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: var(--space-1);
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

@media (max-width: 560px) {
  .card { padding: var(--space-6); }
}
</style>
