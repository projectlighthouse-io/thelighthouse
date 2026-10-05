<script setup lang="ts">
import { plannedBooks, plannedProjects } from '@/data/Horizon'

/**
 * What comes after the desk: the planned books, grouped by the bundle each is
 * meant for, and the luxctl projects planned beside them. Under "what is being
 * written now", so the page reads now → next.
 */
const bundles = computed(() => {
  const order = [...new Set(plannedBooks.map(book => book.bundle))]

  return order.map(name => ({
    name,
    titles: plannedBooks.filter(book => book.bundle === name).map(book => book.title),
  }))
})
</script>

<template>
  <section class="horizon lh-wide lh-gap">
    <div class="head">
      <p class="lh-eyebrow">on the horizon</p>
      <h2 class="lh-h2">And after that</h2>
      <p class="lh-sub">
        The books and projects planned next. Nothing here has a date yet — they move to the desk
        when the writing starts.
      </p>
    </div>

    <div class="columns">
      <div class="column">
        <p class="lh-eyebrow">books · {{ plannedBooks.length }}</p>
        <div v-for="bundle in bundles" :key="bundle.name" class="bundle">
          <span class="bundle-name">{{ bundle.name }}</span>
          <ul class="list">
            <li v-for="title in bundle.titles" :key="title">{{ title }}</li>
          </ul>
        </div>
      </div>

      <div class="column">
        <p class="lh-eyebrow">projects · {{ plannedProjects.length }}</p>
        <ul class="list projects">
          <li v-for="project in plannedProjects" :key="project.title">
            <span class="title">{{ project.title }}</span>
            <span class="lh-caption">{{ project.line }}</span>
          </li>
        </ul>
      </div>
    </div>
  </section>
</template>

<style scoped>
.horizon {
  display: grid;
  gap: var(--space-12);
}

.head {
  display: grid;
  gap: var(--space-4);
  max-width: var(--measure-text);
}

.head > * { margin: 0; }

.columns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-16);
  align-items: start;
}

.column {
  display: grid;
  gap: var(--space-6);
}

.column > .lh-eyebrow { margin: 0; }

.bundle {
  display: grid;
  gap: var(--space-2);
}

.bundle-name {
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: var(--space-2);
  font: var(--text-body-sm);
  color: var(--ink);
}

.projects { gap: var(--space-4); }

.projects li {
  display: grid;
  gap: 2px;
}

.title { font-weight: 500; }

@media (max-width: 760px) {
  .columns {
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-12);
  }
}
</style>
