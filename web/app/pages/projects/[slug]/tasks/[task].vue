<script setup lang="ts">
import type { TaskPage } from '@/types/Content'

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
  <div v-if="task" class="mx-auto max-w-3xl px-2 sm:px-6 lg:px-8">
    <nav class="pt-10 pb-8 font-mono text-sm text-faint">
      <NuxtLink to="/projects" class="hover:text-ink">projects</NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <NuxtLink :to="`/projects/${task.project.slug}`" class="hover:text-ink">
        {{ task.project.slug }}
      </NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <span class="text-quiet">{{ task.slug }}</span>
    </nav>

    <article class="pb-20">
      <div class="flex items-baseline justify-between gap-4">
        <div class="font-mono text-xs tracking-[0.2em] uppercase text-teal">
          task {{ String(task.position).padStart(2, '0') }} of {{ task.total }}
        </div>

        <!-- Only once the poll has answered — see the projects page. -->
        <div v-if="mine" class="font-mono text-xs">
          <span v-if="mine.status === 'challenge_completed'" class="text-teal">
            done · {{ mine.points_earned }} pts
          </span>
          <span v-else-if="mine.attempts > 0" class="text-faint">
            {{ mine.attempts }} {{ mine.attempts === 1 ? 'attempt' : 'attempts' }}
          </span>
        </div>
      </div>

      <h1 class="mt-3 font-serif text-4xl text-ink sm:text-5xl">
        {{ task.title }}
      </h1>

      <div v-if="task.html" class="reader-prose" style="margin-top: 40px">
        <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
        <div class="lesson-content" v-html="task.html" />
      </div>

      <div class="border-pencil-light mt-10 rounded-md bg-panel p-8">
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
</template>
