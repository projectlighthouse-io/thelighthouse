<script setup lang="ts">
import { languages } from '@/data/Syntax'

const description
  = 'Quick syntax reference for Go, Rust, and C. One page per language, heavily commented code you can scan in minutes.'

useSeo({
  title: 'Syntax Reference — projectlighthouse',
  description,
})

useJsonLd('syntax', {
  '@type': 'CollectionPage',
  'name': 'Syntax Reference',
  'mainEntity': {
    '@type': 'ItemList',
    'itemListElement': languages.map((l, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'url': `${SITE.url}/syntax/${l.slug}`,
      'name': `${l.name} Syntax Reference`,
    })),
  },
})
</script>

<template>
  <div class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
    <div class="font-mono text-xs tracking-[0.2em] uppercase text-teal">reference</div>
    <h1 class="mt-3 font-editorial text-4xl text-ink sm:text-5xl font-semibold">
      Syntax Reference
    </h1>
    <p class="mt-6 font-serif text-lg leading-relaxed text-quiet">
      One page per language. Heavily commented code you can scan in minutes. Not a tutorial &mdash;
      a reference you keep open in a tab.
    </p>

    <div class="mt-12 grid gap-6 sm:grid-cols-2">
      <NuxtLink
        v-for="lang in languages"
        :key="lang.slug"
        :to="`/syntax/${lang.slug}`"
        class="border-pencil group rounded-md bg-panel p-6 no-underline"
      >
        <h2 class="font-editorial text-2xl text-ink font-semibold">{{ lang.name }}</h2>
        <p class="mt-2 font-serif text-base leading-relaxed text-quiet">{{ lang.description }}</p>
        <span class="btn-chalk mt-4 inline-flex text-sm font-medium text-ink">
          open <span class="ml-1">———→</span>
        </span>
      </NuxtLink>
    </div>
  </div>
</template>
