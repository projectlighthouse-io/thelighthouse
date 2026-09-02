<script setup lang="ts">
import type { ProjectPage } from '@/types/Content'

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))

// From ohara, through the rust api. A project that is not there 404s in the
// nitro handler, which is where the api's own 404 is translated — see
// `server/utils/Lighthouse.ts`.
const { data } = await useAsyncData(
  () => `project:${slug.value}`,
  () => $fetch<ProjectPage>(`/_api/projects/${slug.value}`),
  { watch: [slug] },
)

const project = computed<ProjectPage | null>(() => data.value ?? null)

if (!project.value) {
  throw createError({ statusCode: 404, statusMessage: 'Project not found', fatal: true })
}

// What the reader has done, refreshed while they work in their terminal. Null
// for a signed-out reader, which is why every use below is guarded rather than
// defaulted — "no progress" and "no attempts" are different things to draw.
const { forTask, progress } = useProjectProgress(slug)

const label = computed<string>(() => (project.value?.isChallenge ? 'challenge' : 'project'))

/** Where "start" goes: the first task not already done, or the first of all. */
const resume = computed<string>(() => {
  const tasks = project.value?.tasks ?? []
  const next = tasks.find(t => forTask(t.slug)?.status !== 'challenge_completed')

  return next?.slug ?? tasks[0]?.slug ?? ''
})

useSeo(() => ({
  title: `${project.value?.name} — projectlighthouse`,
  description: project.value?.shortDescription ?? '',
}))

useJsonLd('project', () => ({
  '@type': 'Course',
  'name': project.value?.name,
  'description': project.value?.shortDescription,
  'provider': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
  'hasCourseInstance': {
    '@type': 'CourseInstance',
    'courseMode': 'online',
    'courseWorkload': `PT${(project.value?.tasksCount ?? 0) * 2}H`,
  },
}))
</script>

<template>
  <div v-if="project" class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
    <nav class="pt-10 pb-8 font-mono text-sm text-faint">
      <NuxtLink to="/projects" class="hover:text-ink">projects</NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <span class="text-quiet">{{ project.slug }}</span>
    </nav>

    <section class="grid gap-12 pb-16 lg:grid-cols-[1fr_360px] lg:items-start">
      <div>
        <div class="mb-8 flex flex-wrap items-center gap-2 font-mono text-xs">
          <span class="inline-flex items-center rounded-full border border-stroke px-3 py-1 text-ink">
            {{ label }}
          </span>
          <span class="text-faint">{{ project.tasksCount }} tasks</span>
          <span v-if="project.difficulty" class="text-faint">· {{ project.difficulty }}</span>
        </div>

        <p v-if="project.headline" class="mb-3 font-mono text-sm text-teal">
          {{ project.headline }}
        </p>

        <h1
          class="font-editorial text-ink font-semibold text-hero leading-[1.05] tracking-editorial"
        >
          {{ project.name }}
        </h1>

        <p class="mt-8 max-w-xl font-serif text-lg leading-relaxed text-ink">
          {{ project.shortDescription }}
        </p>

        <div class="mt-10">
          <NuxtLink
            :to="`/projects/${project.slug}/tasks/${resume}`"
            class="inline-block rounded-md bg-ink px-5 py-3 text-base font-medium text-on-ink transition hover:bg-ink-hover"
          >
            {{ progress && progress.completed > 0 ? 'Continue' : `Start the ${label}` }}
          </NuxtLink>
        </div>

        <div class="mt-14">
          <div class="mb-6 flex items-baseline justify-between">
            <h2 class="font-serif text-2xl text-ink">Tasks</h2>
            <!-- Only once the poll has answered. A bare "0 / 8" drawn before
                 the first response reads as "you have done none of this" to a
                 reader who has finished it. -->
            <span v-if="progress" class="font-mono text-xs tabular-nums text-faint">
              {{ progress.completed }} / {{ progress.total }} done
              <span v-if="progress.points_earned > 0" class="ml-2 text-teal">
                {{ progress.points_earned }} pts
              </span>
            </span>
          </div>

          <ul>
            <li
              v-for="task in project.tasks"
              :key="task.slug"
              class="border-b border-dashed border-rule-soft py-4 last:border-b-0"
            >
              <NuxtLink
                :to="`/projects/${project.slug}/tasks/${task.slug}`"
                class="flex items-baseline gap-6 px-2 no-underline"
              >
                <span class="w-10 shrink-0 font-mono text-sm tabular-nums text-numeral">
                  {{ String(task.sortOrder).padStart(2, '0') }}
                </span>
                <span class="font-editorial text-lg text-ink">{{ task.title }}</span>

                <span class="ml-auto shrink-0 font-mono text-xs">
                  <span
                    v-if="forTask(task.slug)?.status === 'challenge_completed'"
                    class="text-teal"
                  >done</span>
                  <span
                    v-else-if="forTask(task.slug)?.status === 'challenge_failed'"
                    class="text-faint"
                  >failed</span>
                  <span
                    v-else-if="forTask(task.slug)?.is_locked"
                    class="text-faint"
                  >locked</span>
                  <span v-else class="text-faint">{{ task.points }} pts</span>
                </span>
              </NuxtLink>
            </li>
          </ul>
        </div>

        <div v-if="project.features.length" class="mt-14">
          <h2 class="mb-6 font-serif text-2xl text-ink">What you'll build</h2>
          <ul class="grid gap-6 sm:grid-cols-2">
            <li v-for="feature in project.features" :key="feature.title">
              <div class="font-editorial text-ink">{{ feature.title }}</div>
              <p v-if="feature.description" class="mt-1 text-sm leading-relaxed text-quiet">
                {{ feature.description }}
              </p>
            </li>
          </ul>
        </div>
      </div>

      <aside class="hidden lg:block">
        <div class="border-pencil-light sticky top-24 rounded-md bg-panel p-7">
          <div class="font-mono text-xs tracking-wider uppercase text-teal">run it locally</div>
          <pre class="mt-5 overflow-x-auto rounded-md bg-term-bg p-4 font-mono text-xs leading-relaxed text-term-text"><span class="text-term-dim">$</span> luxctl projects start {{ project.slug }}
<span class="text-term-dim">$</span> luxctl tasks submit</pre>
          <p class="mt-5 text-sm leading-relaxed text-quiet">
            Everything runs on your own machine. luxctl validates your work and reports back.
          </p>
        </div>
      </aside>
    </section>
  </div>
</template>
