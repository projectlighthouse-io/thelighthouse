<script setup lang="ts">
import type { RoadmapItem } from '@/types/Content'
import { columns } from '@/data/Roadmap'

const search = ref<string>('')

const matches = (item: RoadmapItem, q: string): boolean =>
  !q
  || item.title.toLowerCase().includes(q)
  || item.description.toLowerCase().includes(q)
  || item.tags.some(t => t.toLowerCase().includes(q))

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  return columns.map(col => ({ ...col, items: col.items.filter(i => matches(i, q)) }))
})

const total = computed<number>(() => filtered.value.reduce((n, c) => n + c.items.length, 0))

const description
  = 'What is shipped, what is being built, and what is still an idea. The roadmap for projectlighthouse.'

useSeo({
  title: 'Roadmap — projectlighthouse',
  description,
})

useJsonLd('roadmap', {
  '@type': 'CollectionPage',
  'name': 'Roadmap',
  'mainEntity': {
    '@type': 'ItemList',
    'itemListElement': columns.flatMap(c => c.items).map((item, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'name': item.title,
      'description': item.description,
    })),
  },
})
</script>

<template>
  <div class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
    <div class="mx-auto max-w-3xl text-center">
      <h1 class="mb-4 font-serif text-4xl tracking-tight text-ink sm:text-5xl">Roadmap</h1>
      <p class="text-mono-body">what's shipped, what's building, what's still an idea.</p>
    </div>

    <div class="mx-auto mt-10 max-w-md">
      <input
        v-model="search"
        type="search"
        placeholder="filter the roadmap…"
        class="w-full rounded-md border border-rule bg-panel px-4 py-2.5 font-mono text-sm text-ink placeholder:text-faint focus:border-stroke focus:outline-none"
      >
      <p class="mt-2 text-center font-mono text-xs text-faint">{{ total }} items</p>
    </div>

    <div class="mt-12 grid gap-8 lg:grid-cols-4">
      <section v-for="col in filtered" :key="col.label">
        <header class="mb-5 flex items-center gap-2">
          <span class="inline-block size-2 rounded-full" :class="col.dotClass" />
          <h2 class="font-mono text-sm font-semibold" :class="col.headerClass">{{ col.label }}</h2>
          <span class="font-mono text-xs text-faint">{{ col.items.length }}</span>
        </header>

        <ul class="space-y-4">
          <li
            v-for="item in col.items"
            :key="item.title"
            class="rounded-md bg-panel p-5"
            :class="col.cardClass"
          >
            <div class="flex items-start justify-between gap-2">
              <h3 class="font-editorial text-base text-ink font-semibold">
                {{ item.title }}
              </h3>
              <span v-if="item.hot" class="font-mono text-xs text-rose">hot</span>
            </div>
            <p class="mt-2 text-sm leading-relaxed text-quiet">{{ item.description }}</p>
            <div class="mt-3 flex flex-wrap gap-2">
              <span v-for="tag in item.tags" :key="tag" class="font-mono text-xs text-teal">
                #{{ tag }}
              </span>
            </div>
          </li>
        </ul>
      </section>
    </div>
  </div>
</template>
