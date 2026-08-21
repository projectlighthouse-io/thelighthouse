<script setup lang="ts">
import { challenges, projects } from '@/data/Projects'

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))
const taskNumber = computed<number>(() => Number(route.params.task))

const all = [...projects, ...challenges]
const project = computed(() => all.find(p => p.slug === slug.value))

const valid = computed<boolean>(
  () =>
    !!project.value
    && Number.isInteger(taskNumber.value)
    && taskNumber.value >= 1
    && taskNumber.value <= project.value.tasksCount,
)

if (!valid.value) {
  throw createError({ statusCode: 404, statusMessage: 'Task not found', fatal: true })
}

const previous = computed<number | null>(() => (taskNumber.value > 1 ? taskNumber.value - 1 : null))
const next = computed<number | null>(() =>
  project.value && taskNumber.value < project.value.tasksCount ? taskNumber.value + 1 : null,
)

useSeo({
  title: `Task ${taskNumber.value} — ${project.value?.name}`,
  description: project.value?.shortDescription ?? '',
  noindex: true,
})
</script>

<template>
  <div v-if="project" class="mx-auto max-w-5xl px-4 sm:px-6 lg:px-8">
    <nav class="pt-10 pb-8 font-mono text-sm text-faint">
      <NuxtLink to="/projects" class="hover:text-ink">projects</NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <NuxtLink :to="`/projects/${project.slug}`" class="hover:text-ink">{{ project.slug }}</NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <span class="text-quiet">task {{ taskNumber }}</span>
    </nav>

    <article class="pb-20">
      <div class="font-mono text-xs tracking-[0.2em] uppercase text-teal">
        task {{ String(taskNumber).padStart(2, '0') }} of {{ project.tasksCount }}
      </div>

      <h1 class="mt-3 font-serif text-4xl text-ink sm:text-5xl">
        {{ project.name }}
      </h1>

      <div class="border-pencil-light mt-10 rounded-md bg-panel p-8">
        <p class="font-serif text-lg leading-relaxed text-ink">
          {{ project.shortDescription }}
        </p>

        <p class="mt-6 text-mono-body">
          Task briefs, hints and validators come from the API — see docs/rebuild.md phase 7. This
          page is the shell they land in.
        </p>

        <pre class="mt-8 overflow-x-auto rounded-md bg-term-bg p-5 font-mono text-xs leading-relaxed text-term-text sm:text-sm"><span class="text-term-dim">$</span> luxctl projects start {{ project.slug }}
<span class="text-term-dim">$</span> luxctl tasks submit
  <span class="text-term-dim">→ running validator on your machine…</span>
  <span class="text-term-green">✓ checks passed</span></pre>
      </div>

      <div class="mt-12 flex items-center justify-between gap-4">
        <NuxtLink
          v-if="previous"
          :to="`/projects/${project.slug}/tasks/${previous}`"
          class="btn-chalk text-sm text-ink"
        >
          ← task {{ previous }}
        </NuxtLink>
        <span v-else />
        <NuxtLink
          v-if="next"
          :to="`/projects/${project.slug}/tasks/${next}`"
          class="btn-chalk text-sm text-ink"
        >
          task {{ next }} →
        </NuxtLink>
      </div>
    </article>
  </div>
</template>
