<script setup lang="ts">
import { challenges, projects } from '@/data/Projects'

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))

const all = [...projects, ...challenges]
const project = computed(() => all.find(p => p.slug === slug.value))
const isChallenge = computed<boolean>(() => challenges.some(c => c.slug === slug.value))

if (!project.value) {
  throw createError({ statusCode: 404, statusMessage: 'Project not found', fatal: true })
}

// task names are not in the static data yet — the api provides them in phase 7
const taskNumbers = computed<number[]>(() =>
  Array.from({ length: project.value?.tasksCount ?? 0 }, (_, i) => i + 1),
)

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
            {{ isChallenge ? 'challenge' : 'project' }}
          </span>
          <span class="text-faint">{{ project.tasksCount }} tasks</span>
        </div>

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
            :to="`/projects/${project.slug}/tasks/1`"
            class="inline-block rounded-md bg-ink px-5 py-3 text-base font-medium text-on-ink transition hover:bg-ink-hover"
          >
            Start the {{ isChallenge ? 'challenge' : 'project' }}
          </NuxtLink>
        </div>

        <div class="mt-14">
          <h2 class="mb-6 font-serif text-2xl text-ink">Tasks</h2>
          <ul>
            <li
              v-for="n in taskNumbers"
              :key="n"
              class="border-b border-dashed border-rule-soft py-4 last:border-b-0"
            >
              <NuxtLink
                :to="`/projects/${project.slug}/tasks/${n}`"
                class="flex items-baseline gap-6 px-2 no-underline"
              >
                <span class="w-10 shrink-0 font-mono text-sm tabular-nums text-numeral">
                  {{ String(n).padStart(2, '0') }}
                </span>
                <span class="font-editorial text-lg text-ink">Task {{ n }}</span>
              </NuxtLink>
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
