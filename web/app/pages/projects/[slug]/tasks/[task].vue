<script setup lang="ts">
import type { TaskPage } from '@/types/Content'
// the reader's column, pager and prose
import '@/assets/css/reader.css'

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))
const taskSlug = computed<string>(() => String(route.params.task))

// The brief itself, from ohara through the rust api. A task that is not in
// that project 404s in the nitro handler.
const { data } = await useAsyncData(
  () => `task:${slug.value}:${taskSlug.value}`,
  () => $fetch<TaskPage>(`/_api/projects/${slug.value}/tasks/${taskSlug.value}`),
  { watch: [slug, taskSlug] },
)

const task = computed<TaskPage | null>(() => data.value ?? null)

if (!task.value) {
  throw createError({ statusCode: 404, statusMessage: 'Task not found', fatal: true })
}

// The reader is working in their terminal; this tab catches up on its own.
const { forTask } = useProjectProgress(slug)

const mine = computed(() => forTask(taskSlug.value))

useSeo(() => ({
  title: `${task.value?.title} — ${task.value?.project.name}`,
  description: task.value?.project.name ?? '',
  noindex: true,
}))
</script>

<template>
  <div v-if="task" class="reader-shell">
    <div class="reader-layout">
      <article class="reader-article">
        <header class="reader-head">
          <p class="lh-eyebrow">
            <NuxtLink :to="`/projects/${task.project.slug}`" class="lh-link">{{ task.project.name }}</NuxtLink>
            · task {{ String(task.position).padStart(2, '0') }} of {{ task.total }}
          </p>
          <h1 class="lh-h1">{{ task.title }}</h1>

          <!-- Only once the poll has answered — see the project page. -->
          <div v-if="mine" class="reader-meta">
            <span v-if="mine.status === 'challenge_completed'" class="lh-num">
              done · {{ mine.points_earned }} points
            </span>
            <span v-else-if="mine.attempts > 0" class="lh-num">
              {{ mine.attempts }} {{ mine.attempts === 1 ? 'attempt' : 'attempts' }}
            </span>
          </div>
        </header>

        <div v-if="task.html" class="reader-body">
          <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
          <div class="lesson-content" v-html="task.html" />
        </div>

        <div class="run">
          <p class="lh-sub">
            Validators and hints run in your terminal, not here. luxctl reads the project's
            blueprint, checks your work on your own machine, and reports the outcome back.
          </p>
          <TerminalPanel tag="luxctl" note="run it on your own machine">
            <pre class="commands"><span class="dim">$ </span>luxctl projects start {{ task.project.slug }}
<span class="dim">$ </span>luxctl tasks submit
  <span class="dim">→ running validator on your machine…</span>
  <span class="ok">✓ checks passed</span></pre>
          </TerminalPanel>
        </div>

        <nav class="reader-pager" aria-label="tasks">
          <UiButton
            v-if="task.previous"
            variant="ghost"
            size="lg"
            :to="`/projects/${task.project.slug}/tasks/${task.previous.slug}`"
          >
            ← {{ task.previous.title }}
          </UiButton>
          <UiButton
            v-if="task.next"
            variant="ghost"
            size="lg"
            :to="`/projects/${task.project.slug}/tasks/${task.next.slug}`"
          >
            {{ task.next.title }} →
          </UiButton>
        </nav>
      </article>
    </div>
  </div>
</template>

<style scoped>
.run {
  display: grid;
  gap: var(--space-6);
  margin-top: var(--space-12);
}

.commands {
  margin: 0;
  padding: var(--space-4) 28px var(--space-6);
  overflow-x: auto;
  font: var(--text-code);
  color: var(--term-bright);
}

.dim { color: var(--term-grey); }
.ok { color: var(--term-green); }
</style>
