<script setup lang="ts">
/**
 * The way back up from a page in the reader: Books / <book> / <this page>.
 *
 * Every crumb but the last is a link; the last is the page itself and is
 * marked as such rather than linked to. Built only from what the server
 * already rendered, so it is the same markup on both sides of hydration.
 */

export interface Crumb {
  label: string
  /** Omitted on the current page. */
  to?: string
}

defineProps<{ items: Crumb[] }>()
</script>

<template>
  <nav aria-label="Breadcrumb" class="crumbs">
    <ol>
      <li v-for="(item, i) in items" :key="i">
        <span v-if="i > 0" class="sep" aria-hidden="true">/</span>
        <NuxtLink v-if="item.to" :to="item.to" class="lh-link crumb">{{ item.label }}</NuxtLink>
        <span v-else class="crumb is-current" aria-current="page">{{ item.label }}</span>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.crumbs ol {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
  margin: 0;
  padding: 0;
  list-style: none;
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

/* One line at every width: long titles give way with an ellipsis instead of
   wrapping, so the head below never moves. "Books" never shrinks. */
.crumbs li {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
  min-width: 0;
  flex: 0 1 auto;
}

.crumbs li:first-child { flex-shrink: 0; }

.crumb {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.crumbs .lh-link { color: var(--ink-muted); }
.crumbs .lh-link:hover { color: var(--ink); }

.is-current { color: var(--ink-secondary); }

.sep {
  flex-shrink: 0;
  color: var(--ink-faint);
}
</style>
