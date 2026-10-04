<script setup lang="ts">
import type { TocSection } from '@/types/Content'

/**
 * A numbered contents list in sections — a book's chapters and lessons, a
 * project's stages. Rows are quiet links that fill on hover.
 *
 * Free and pro are told apart without colour: the first locked row is
 * preceded by a dashed rule and a mono "pro · n lessons" label, and every
 * locked row carries a faint "pro". No pills.
 */
const props = withDefaults(
  defineProps<{
    sections: TocSection[]
    /** What a row is called in the pro label — lessons, stages, tasks. */
    unit?: string
    compact?: boolean
  }>(),
  { unit: 'lessons', compact: false },
)

/** The first locked row anywhere, which is where the pro block starts. */
const firstLocked = computed<string | null>(() => {
  for (const section of props.sections) {
    const row = section.rows.find(r => r.locked)
    if (row) return row.to
  }

  return null
})

const lockedCount = computed<number>(() =>
  props.sections.reduce((n, s) => n + s.rows.filter(r => r.locked).length, 0))

// Only worth a rule when there is something free before it.
const showsSplit = computed<boolean>(() => {
  const total = props.sections.reduce((n, s) => n + s.rows.length, 0)

  return lockedCount.value > 0 && lockedCount.value < total
})
</script>

<template>
  <div class="toc" :class="{ 'is-compact': compact }">
    <section v-for="section in sections" :key="section.key" class="section">
      <div class="head">
        <span class="lh-eyebrow">{{ section.eyebrow }}</span>
        <h2 class="lh-h2 title">{{ section.title }}</h2>
      </div>

      <ol class="rows">
        <template v-for="row in section.rows" :key="row.to">
          <li v-if="showsSplit && row.to === firstLocked" class="split" aria-hidden="true">
            <hr class="lh-dashed">
            <span class="lh-eyebrow">pro · {{ lockedCount }} {{ unit }}</span>
          </li>
          <li>
            <NuxtLink
              :to="row.to"
              class="row"
              :class="{ 'is-current': row.current }"
              :aria-current="row.current ? 'page' : undefined"
            >
              <span class="n lh-num">{{ row.n }}</span>
              <span class="name">{{ row.title }}</span>
              <span class="access">
                <template v-if="row.locked">pro<span class="lh-sr"> — part of this needs a plan</span></template>
              </span>
              <span v-if="row.blurb && !compact" class="blurb">{{ row.blurb }}</span>
            </NuxtLink>
          </li>
        </template>
      </ol>
    </section>
  </div>
</template>

<style scoped>
.toc {
  display: grid;
  gap: 72px;
}

.section {
  display: grid;
  gap: var(--space-4);
}

.head {
  display: grid;
  gap: 6px;
  padding: 0 var(--space-4);
}

.rows {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 2px;
}

.row {
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr) auto;
  gap: var(--space-1) var(--space-3);
  align-items: baseline;
  padding: 14px var(--space-4);
  border-radius: var(--radius-md);
  color: var(--ink);
  text-decoration: none;
  transition: background-color var(--duration) var(--ease-out);
}

.row:hover,
.row.is-current { background: var(--surface-sunken); color: var(--ink); }

.n {
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

.name { font: 500 17px/24px var(--font-serif); }

.access {
  justify-self: end;
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

.blurb {
  grid-column: 2 / -1;
  max-width: 640px;
  font: var(--text-caption);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

.split {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-4) var(--space-2);
}

/* the reader's sidebar: same rows, tighter */
.is-compact { gap: var(--space-8); }
.is-compact .head { padding: 0 var(--space-3); }
.is-compact .title { font: var(--text-h3); letter-spacing: var(--tracking-title); }
.is-compact .row { grid-template-columns: 32px minmax(0, 1fr) auto; padding: var(--space-2) var(--space-3); }
.is-compact .name { font: var(--text-body-sm); }
.is-compact .split { padding: var(--space-3) var(--space-3) var(--space-1); }
</style>
