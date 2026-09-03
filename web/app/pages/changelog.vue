<script setup lang="ts">
import { changelog } from '@/data/Changelog'

const activeTag = ref<string>('all')

const tags = computed<string[]>(() => ['all', ...new Set(changelog.map(e => e.tag))].sort())

const shown = computed(() =>
  activeTag.value === 'all' ? changelog : changelog.filter(e => e.tag === activeTag.value),
)

const description = 'Every change shipped to projectlighthouse — books, projects, and platform.'

useSeo({
  title: 'Changelog — projectlighthouse',
  description,
})

useJsonLd('changelog', {
  '@type': 'CollectionPage',
  'name': 'Changelog',
  'mainEntity': {
    '@type': 'ItemList',
    'numberOfItems': changelog.length,
    'itemListElement': changelog.slice(0, 30).map((e, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'name': e.title,
    })),
  },
})
</script>

<template>
  <div class="mx-auto max-w-3xl px-2 py-16 sm:px-6 lg:px-8">
    <div class="text-center">
      <h1 class="mb-4 font-serif text-4xl tracking-tight text-ink sm:text-5xl">Changelog</h1>
      <p class="text-mono-body">{{ changelog.length }} changes, newest first.</p>
    </div>

    <div class="mt-10 flex flex-wrap justify-center gap-2">
      <button
        v-for="tag in tags"
        :key="tag"
        type="button"
        class="cursor-pointer rounded-md px-4 py-1.5 font-mono text-xs"
        :class="activeTag === tag ? 'border-pencil-solid-black text-ink' : 'text-quiet hover:text-ink'"
        @click="activeTag = tag"
      >
        {{ tag }}
      </button>
    </div>

    <ol class="mt-12">
      <li
        v-for="(entry, i) in shown"
        :key="`${entry.date}-${i}`"
        class="border-b border-dashed border-rule-soft py-6 last:border-b-0"
      >
        <div class="flex flex-wrap items-baseline gap-3 font-mono text-xs text-faint">
          <time>{{ entry.date }}</time>
          <span class="text-teal">#{{ entry.tag }}</span>
        </div>
        <h2 class="mt-2 font-editorial text-lg text-ink font-semibold">
          {{ entry.title }}
        </h2>
        <p class="mt-2 text-sm leading-relaxed text-quiet">{{ entry.description }}</p>
      </li>
    </ol>
  </div>
</template>
