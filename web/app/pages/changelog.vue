<script setup lang="ts">
import { changelog } from '@/data/Changelog'

const activeTag = ref<string>('all')

const tags = computed<string[]>(() => ['all', ...new Set(changelog.map(e => e.tag))].sort())

const tagOptions = computed(() => tags.value.map(tag => ({
  key: tag,
  label: tag,
  count: tag === 'all' ? changelog.length : changelog.filter(e => e.tag === tag).length,
})))

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
  <div class="changelog lh-text lh-gap">
    <div class="lh-head">
      <p class="lh-eyebrow">changelog</p>
      <h1 class="lh-h1">What changed</h1>
      <p class="lh-sub">{{ changelog.length }} changes, newest first.</p>
    </div>

    <div class="filter">
      <SegmentedFilter v-model="activeTag" :options="tagOptions" label="filter by tag" />
    </div>

    <ol class="list">
      <li v-for="(entry, i) in shown" :key="`${entry.date}-${i}`">
        <PostRow
          :eyebrow="`${entry.date} · #${entry.tag}`"
          :title="entry.title"
          :caption="entry.description"
        />
      </li>
    </ol>
  </div>
</template>

<style scoped>
.filter { margin-top: var(--space-8); }

.list {
  margin: var(--space-8) calc(var(--space-4) * -1) 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 2px;
}
</style>
