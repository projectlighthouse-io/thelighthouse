<script setup lang="ts">
import type { TaskPage } from '@/types/Content'

// The brief is authored markdown and renders through `.lesson-content`, which
// only exists inside `.reader-shell`. Imported here rather than globally for
// the reason `main.css` gives: it is ~20kb no other route needs.
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
    <div class="reader-subbar">
      <!-- The layout's own grid, borrowed: this page's gutter is empty, so the
           breadcrumb has nothing to line up with unless it sits in the same
           column the article does. See `reader.css`. -->
      <div class="reader-layout reader-layout--no-aside reader-subbar__grid">
        <div class="reader-subbar__gutter" />

        <div class="reader-subbar__row">
          <NuxtLink class="reader-subbar__book" :to="`/projects/${task.project.slug}`">
            {{ task.project.name }}
          </NuxtLink>
          <span class="reader-subbar__sep">›</span>
          <span class="reader-subbar__cur">{{ task.title }}</span>
          <span class="reader-subbar__spacer" />

          <!-- Only once the poll has answered — see the projects page. -->
          <span v-if="mine?.status === 'challenge_completed'" class="reader-subbar__rt">
            done · {{ mine.points_earned }} pts
          </span>
          <span v-else-if="mine && mine.attempts > 0" class="reader-subbar__rt">
            {{ mine.attempts }} {{ mine.attempts === 1 ? 'attempt' : 'attempts' }}
          </span>
        </div>
      </div>
    </div>

    <div class="reader-layout reader-layout--no-aside">
      <!-- A task carries no toc, so the gutter is empty. It still has to be
           here: the article sits in the middle track of a three-track grid. -->
      <aside class="reader-toc" />

      <article class="reader-article">
        <div class="reader-eyebrow">
          task {{ String(task.position).padStart(2, '0') }} of {{ task.total }}
        </div>
        <h1>{{ task.title }}</h1>

        <div v-if="task.html" class="reader-prose" style="margin-top: 44px">
          <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
          <div class="lesson-content" v-html="task.html" />
        </div>

        <div class="border-pencil-light mt-12 rounded-md bg-panel p-8">
          <p class="text-mono-body">
            Validators and hints run in your terminal, not here. luxctl reads the project's
            blueprint, checks your work on your own machine, and reports the outcome back.
          </p>

          <pre class="mt-8 overflow-x-auto rounded-md bg-term-bg p-5 font-mono text-xs leading-relaxed text-term-text sm:text-sm"><span class="text-term-dim">$</span> luxctl projects start {{ task.project.slug }}
<span class="text-term-dim">$</span> luxctl tasks submit
  <span class="text-term-dim">→ running validator on your machine…</span>
  <span class="text-term-green">✓ checks passed</span></pre>
        </div>

        <div class="mt-12 flex items-center justify-between gap-4">
          <NuxtLink
            v-if="task.previous"
            :to="`/projects/${task.project.slug}/tasks/${task.previous.slug}`"
            class="btn-chalk text-sm text-ink"
          >
            ← {{ task.previous.title }}
          </NuxtLink>
          <span v-else />
          <NuxtLink
            v-if="task.next"
            :to="`/projects/${task.project.slug}/tasks/${task.next.slug}`"
            class="btn-chalk text-sm text-ink"
          >
            {{ task.next.title }} →
          </NuxtLink>
        </div>
      </article>
    </div>
  </div>
</template>
