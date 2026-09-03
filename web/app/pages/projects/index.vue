<script setup lang="ts">
import type { Project } from '@/types/Content'

type Tab = 'projects' | 'challenges'

const activeTab = ref<Tab>('projects')

// From ohara, through the rust api. During SSR this calls the handler
// directly, so it costs no HTTP round trip.
const { data } = await useAsyncData('projects', () =>
  $fetch<Project[]>('/_api/projects'))

const all = computed<Project[]>(() => data.value ?? [])

// Split here rather than served as two lists: which tab a project belongs in
// is `is_challenge` in its own yaml, so there is no second list that could
// disagree with the first.
const projects = computed<Project[]>(() => all.value.filter(p => !p.isChallenge))
const challenges = computed<Project[]>(() => all.value.filter(p => p.isChallenge))

const shown = computed<Project[]>(() =>
  activeTab.value === 'projects' ? projects.value : challenges.value)

const description
  = 'Hands-on coding projects: build Docker, HTTP servers, and DNS resolvers from scratch, or sharpen your grep, sed, and CLI skills. Automated validation and hints, run on your own machine.'

useSeo({
  title: 'Hands-on Projects - Build, Practice, and Validate Your Skills',
  description,
})

useJsonLd('projects', () => ({
  '@type': 'CollectionPage',
  'name': 'Projects and challenges',
  'mainEntity': {
    '@type': 'ItemList',
    'itemListElement': all.value.map((p, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'url': `${SITE.url}/projects/${p.slug}`,
      'name': p.name,
    })),
  },
}))
</script>

<template>
  <div>
    <section class="py-16">
      <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
        <div class="mx-auto max-w-3xl text-center">
          <h1 class="mb-4 font-serif text-4xl tracking-tight text-ink sm:text-5xl">Projects</h1>
          <p class="text-mono-body">build real systems from scratch, or sharpen your tools.</p>
        </div>

        <div class="mt-10 flex justify-center gap-2">
          <button
            type="button"
            class="cursor-pointer rounded-md px-5 py-2 text-sm font-medium"
            :class="activeTab === 'projects'
              ? 'border-pencil-solid-black text-ink'
              : 'text-quiet hover:text-ink'"
            @click="activeTab = 'projects'"
          >
            projects
            <span class="ml-1 text-xs opacity-60">{{ projects.length }}</span>
          </button>
          <button
            type="button"
            class="cursor-pointer rounded-md px-5 py-2 text-sm font-medium"
            :class="activeTab === 'challenges'
              ? 'border-pencil-solid-black text-ink'
              : 'text-quiet hover:text-ink'"
            @click="activeTab = 'challenges'"
          >
            challenges
            <span class="ml-1 text-xs opacity-60">{{ challenges.length }}</span>
          </button>
        </div>
      </div>
    </section>

    <section class="pb-16">
      <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
        <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          <ProjectCard v-for="project in shown" :key="project.slug" :project="project" />
        </div>
      </div>
    </section>
  </div>
</template>
