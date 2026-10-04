<script setup lang="ts">
import type { RoadmapItem } from '@/types/Content'
import { columns } from '@/data/Roadmap'
import { manuscripts } from '@/data/Desk'

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
  <div class="roadmap">
    <MarketingOnTheDesk :manuscripts="manuscripts" eyebrow="roadmap · on the desk" heading="h1" />

    <section class="lh-figure lh-gap-lg board">
      <div class="board-head">
        <div class="lh-head">
          <h2 class="lh-h2">Everything else</h2>
          <p class="lh-sub">What has shipped, what is next, and what is still an idea.</p>
        </div>

        <div class="search">
          <label class="lh-sr" for="roadmap-search">filter the roadmap</label>
          <input
            id="roadmap-search"
            v-model="search"
            type="search"
            class="lh-input"
            placeholder="filter the roadmap…"
          >
          <span class="lh-mono lh-muted lh-num">{{ total }} items</span>
        </div>
      </div>

      <section v-for="col in filtered" :key="col.label" class="column">
        <div class="column-head">
          <span class="lh-eyebrow">{{ col.label.toLowerCase() }} · {{ col.items.length }}</span>
        </div>

        <ul class="rows">
          <li v-for="item in col.items" :key="item.title" class="item">
            <span class="item-top">
              <span class="lh-h3">{{ item.title }}</span>
              <span v-if="item.hot" class="lh-mono lh-faint">in demand</span>
            </span>
            <span class="lh-caption">{{ item.description }}</span>
            <span v-if="item.tags.length" class="tags">
              <span v-for="tag in item.tags" :key="tag">#{{ tag }}</span>
            </span>
          </li>
        </ul>

        <p v-if="!col.items.length" class="lh-hint empty">Nothing matches.</p>
      </section>
    </section>
  </div>
</template>

<style scoped>
.board {
  display: grid;
  gap: 72px;
}

.board-head {
  display: grid;
  gap: var(--space-6);
}

.search {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  max-width: 480px;
}

.column {
  display: grid;
  gap: var(--space-3);
}

.column-head { padding: 0 var(--space-4); }

.rows {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 2px;
}

.item {
  display: grid;
  gap: var(--space-2);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  transition: background-color var(--duration) var(--ease-out);
}

.item:hover { background: var(--surface-sunken); }

.item-top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-4);
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px var(--space-4);
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.empty { padding: 0 var(--space-4); }
</style>
