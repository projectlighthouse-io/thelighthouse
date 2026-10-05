<script setup lang="ts">
import type { Book } from '@/types/Content'

/**
 * One book on a shelf: the cover on white, then pages, title and blurb.
 *
 * The one place a border and a shadow share an element — the handoff names
 * book cards as the exception. `progress`, when given, adds the desk's tick
 * bar for the reader's own shelf.
 */
const props = defineProps<{
  book: Book
  progress?: { done: number, total: number }
}>()

const ticks = computed<boolean[]>(() => {
  const progress = props.progress
  if (!progress) return []

  return Array.from({ length: progress.total }, (_, i) => i < progress.done)
})
</script>

<template>
  <NuxtLink :to="`/books/${book.slug}`" class="card">
    <span class="inner">
      <span class="cover">
        <img
          :src="book.thumbnailUrl"
          :alt="`${book.title} cover`"
          width="480"
          height="320"
          loading="lazy"
        >
      </span>

      <span class="body">
        <span class="pages lh-num">{{ book.pages }} pages</span>
        <span class="title">{{ book.title }}</span>
        <span class="blurb">{{ book.description }}</span>

        <span v-if="progress" class="progress">
          <span class="lh-ticks" aria-hidden="true">
            <span v-for="(done, i) in ticks" :key="i" :class="{ 'is-done': done }" />
          </span>
          <span class="lh-mono lh-muted lh-num">{{ progress.done }} of {{ progress.total }} lessons</span>
        </span>
      </span>
    </span>
  </NuxtLink>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 7px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--surface-raised);
  box-shadow: var(--shadow-md);
  color: var(--ink);
  text-decoration: none;
  overflow: hidden;
}

.card:hover { color: var(--ink); }
.card:hover .body { background: var(--surface-sunken); }

.inner {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.cover {
  display: block;
  aspect-ratio: 3 / 2;
  background: var(--surface-raised);
}

.cover img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: var(--surface-raised);
}

.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: var(--space-4) var(--space-4) var(--space-3);
  transition: background-color var(--duration) var(--ease-out);
}

.pages { font: var(--text-caption); color: var(--ink-muted); }

.title {
  font: 500 16px/22px var(--font-sans);
  letter-spacing: var(--tracking-title);
}

.blurb {
  font: var(--text-body-sm);
  color: var(--ink-secondary);
  text-wrap: pretty;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.progress {
  display: grid;
  gap: var(--space-2);
  margin-top: auto;
  padding-top: var(--space-2);
}
</style>
