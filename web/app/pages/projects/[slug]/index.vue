<script setup lang="ts">
import type { ProjectPage, TocSection } from '@/types/Content'

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

const unit = computed<string>(() => (project.value?.isChallenge ? 'tasks' : 'stages'))

const eyebrow = computed<string>(() => {
  const parts = [label.value, `${project.value?.tasksCount ?? 0} ${unit.value}`]
  if (project.value?.difficulty) parts.push(project.value.difficulty)

  return parts.join(' · ')
})

/** What a reader's progress says about one task, in a word. */
function noteFor(slug: string): string | undefined {
  const status = forTask(slug)?.status

  if (status === 'challenge_completed') return 'done'
  if (status === 'challenge_failed') return 'failed'

  return undefined
}

const sections = computed<TocSection[]>(() => [{
  key: 'stages',
  eyebrow: progress.value
    ? `${progress.value.completed} of ${progress.value.total} done`
    : `${project.value?.tasks.length ?? 0} ${unit.value}`,
  title: project.value?.isChallenge ? 'Tasks' : 'Stages',
  rows: (project.value?.tasks ?? []).map(task => ({
    n: String(task.sortOrder).padStart(2, '0'),
    title: task.title,
    blurb: `${task.points} points`,
    to: `/projects/${slug.value}/tasks/${task.slug}`,
    locked: !task.isFree || Boolean(forTask(task.slug)?.is_locked),
    note: noteFor(task.slug),
  })),
}])

const hasPro = computed<boolean>(() => (project.value?.tasks ?? []).some(task => !task.isFree))

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

// Straight to checkout for the plan on sale; sign-in first when signed out.
const { getPro } = useGetPro()
</script>

<template>
  <div v-if="project" class="project">
    <DetailHead
      :eyebrow="eyebrow"
      :title="project.name"
      :description="project.shortDescription"
    >
      <template #actions>
        <UiButton
          v-if="resume"
          variant="inverse"
          size="lg"
          cta="pro"
          :to="`/projects/${project.slug}/tasks/${resume}`"
        >
          {{ progress && progress.completed > 0 ? 'Continue →' : `Start the ${label} →` }}
        </UiButton>
        <UiButton v-if="hasPro" variant="ghost" size="lg" cta="free" flame @click="getPro">
          Get Pro
        </UiButton>
      </template>
    </DetailHead>

    <div class="lh-figure run">
      <TerminalPanel tag="luxctl" note="run it on your own machine">
        <pre class="commands"><span class="dim">$ </span>luxctl projects start {{ project.slug }}
<span class="dim">$ </span>luxctl tasks submit</pre>
      </TerminalPanel>
    </div>

    <div class="lh-figure stages">
      <TocList :sections="sections" :unit="unit" />
    </div>

    <section v-if="project.features.length" class="lh-figure features">
      <div class="head">
        <p class="lh-eyebrow">what you will build</p>
      </div>
      <ul class="feature-grid">
        <li v-for="feature in project.features" :key="feature.title" class="lh-card">
          <h3 class="lh-h3">{{ feature.title }}</h3>
          <p v-if="feature.description" class="lh-sub">{{ feature.description }}</p>
        </li>
      </ul>
    </section>

    <div class="end" aria-hidden="true">
      <UiLogo variant="mark" :size="20" class="end-mark" />
    </div>
  </div>
</template>

<style scoped>
.run { margin-top: var(--space-16); }

.commands {
  margin: 0;
  padding: var(--space-4) 28px var(--space-6);
  overflow-x: auto;
  font: var(--text-code);
  color: var(--term-bright);
}

.dim { color: var(--term-grey); }

.stages { margin-top: var(--space-24); }

.features { margin-top: var(--space-24); display: grid; gap: var(--space-4); }

.features .head { padding: 0 var(--space-4); }

.feature-grid {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
}

.feature-grid li { display: grid; gap: var(--space-2); align-content: start; }

.end {
  display: flex;
  justify-content: center;
  margin-top: var(--space-24);
}

.end-mark { opacity: 0.35; }

@media (max-width: 700px) {
  .feature-grid { grid-template-columns: minmax(0, 1fr); }
}
</style>
