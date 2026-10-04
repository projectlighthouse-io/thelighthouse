<script setup lang="ts">
import type { Project } from '@/types/Content'

// From ohara, through the rust api. During SSR this calls the handler
// directly, so it costs no HTTP round trip.
const { data } = await useAsyncData('projects', () =>
  $fetch<Project[]>('/_api/projects'))

const all = computed<Project[]>(() => data.value ?? [])

// Split here rather than served as two lists: which tab a project belongs in
// is `is_challenge` in its own yaml, so there is no second list that could
// disagree with the first.
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
  <div class="projects">
    <MarketingBuildAndDrill :projects="all" eyebrow="build and drill" heading="h1" sync-url />
  </div>
</template>
