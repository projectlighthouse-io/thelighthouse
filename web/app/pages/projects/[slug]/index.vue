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
  <div v-if="project" class="mx-auto max-w-3xl bg-panel px-6 sm:px-10">
    <nav class="pt-10 pb-8 font-sans text-xs text-crumb">
      <NuxtLink to="/projects" class="hover:text-ink">projects</NuxtLink>
      <span class="mx-2">/</span>
      <span class="text-faint">{{ project.slug }}</span>
    </nav>

    <section class="pb-16">
      <div>
        <h1 class="masthead-title">
          {{ project.name }}
        </h1>

        <p class="masthead-dek">
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
            <h2 class="part-title">Tasks</h2>
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

          <NuxtLink
            v-for="task in project.tasks"
            :key="task.slug"
            :to="`/projects/${project.slug}/tasks/${task.slug}`"
            class="ch"
          >
            <span class="ch-no">{{ String(task.sortOrder).padStart(2, '0') }}</span>
            <span class="ch-title">
              {{ task.title }}
              <span v-if="forTask(task.slug)?.status === 'challenge_completed'" class="pill pill-done">
                done
              </span>
              <span
                v-else-if="forTask(task.slug)?.status === 'challenge_failed'"
                class="pill pill-muted"
              >failed</span>
              <span v-else-if="forTask(task.slug)?.is_locked" class="pill pill-muted">locked</span>
              <span v-else class="pill pill-muted">{{ task.points }} pts</span>
            </span>
          </NuxtLink>
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

      <aside class="mt-14">
        <div class="border-pencil-light rounded-md bg-panel p-7">
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

<style scoped>
/* Lighter than the book's chapter rows. A lesson title is the only thing on
 * its row that carries weight; a task row is shorter and reads as a list of
 * steps rather than a table of contents, and 600 made it shout. */
.ch-title {
    font-weight: 500;
}
</style>
