<script setup lang="ts">
import { plannedProjects, plannedTracks } from '@/data/Horizon'

/**
 * "And after that": the books planned next, as one card per track, and the
 * luxctl projects planned beside them. Directly under the desk, so the page
 * reads now → next. Static — nothing here links anywhere until it has a page.
 * Every count is derived from `data/Horizon.ts`.
 */
const pad = (n: number): string => String(n).padStart(2, '0')

const bookCount = plannedTracks.reduce((n, track) => n + track.books.length, 0)
</script>

<template>
  <section id="after" class="after">
    <div class="head">
      <p class="eyebrow">06 — on the horizon</p>
      <h2 class="title">And after that</h2>
      <p class="lede">
        The books and projects planned next. Nothing here has a date yet. Each one moves to the
        desk when the writing starts.
      </p>
    </div>

    <div class="label books-label">
      <span>books</span>
      <span class="count">{{ bookCount }} planned · {{ plannedTracks.length }} tracks</span>
    </div>

    <div class="tracks">
      <article v-for="(track, t) in plannedTracks" :key="track.name" class="track">
        <div class="track-row">
          <span class="track-index">track {{ pad(t + 1) }}</span>
          <span class="track-count">{{ track.books.length }} books</span>
        </div>
        <h3 class="track-name">{{ track.name }}</h3>
        <ol class="track-books">
          <li v-for="(book, b) in track.books" :key="book">
            <span class="n">{{ pad(b + 1) }}</span>{{ book }}
          </li>
        </ol>
      </article>
    </div>

    <div class="label projects-label">
      <span>projects</span>
      <span class="count">{{ plannedProjects.length }} planned · any language</span>
    </div>

    <div class="projects">
      <div v-for="project in plannedProjects" :key="project.title" class="project">
        <span class="kind">project · planned</span>
        <span class="project-title">{{ project.title }}</span>
        <span class="blurb">{{ project.blurb }}</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.after {
  max-width: 1200px;
  margin: 0 auto;
  padding: 128px 24px 0;
}

/* head */

.head {
  max-width: 640px;
  display: grid;
  gap: 20px;
}

.head > * { margin: 0; }

.eyebrow {
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.title {
  font: var(--text-h1);
  letter-spacing: var(--tracking-h1);
  text-wrap: balance;
}

.lede {
  font: var(--text-body);
  color: var(--ink-secondary);
}

/* group label rows */

.label {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 16px;
  padding-bottom: 12px;
  border-bottom: 1px dashed var(--border-dashed);
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.label .count {
  color: var(--ink-faint);
  font-variant-numeric: tabular-nums;
}

.books-label { margin-top: 56px; }
.projects-label { margin-top: 64px; }

/* track cards */

.tracks {
  margin-top: 24px;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  align-items: stretch;
}

.track {
  display: grid;
  grid-template-rows: auto auto 1fr;
  gap: 20px;
  padding: 24px 28px 28px;
  background: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}

.track-row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  font: var(--text-label-mono);
}

.track-index { color: var(--ink-muted); }

.track-count {
  color: var(--ink-faint);
  font-variant-numeric: tabular-nums;
}

.track-name {
  margin: 0;
  font: var(--text-h3);
  /* No --tracking-h3 token exists; the system's own .lh-h3 uses this one. */
  letter-spacing: var(--tracking-title);
}

.track-books {
  margin: 0;
  padding: 16px 0 0;
  list-style: none;
  display: grid;
  gap: 12px;
  align-content: start;
  border-top: 1px dashed var(--border-dashed);
}

.track-books li {
  display: grid;
  grid-template-columns: 28px 1fr;
  gap: 8px;
  font: var(--text-body-sm);
  color: var(--ink);
}

.n {
  padding-top: 4px;
  font: var(--text-label-mono);
  color: var(--ink-faint);
  font-variant-numeric: tabular-nums;
}

/* projects: a 1px gap over the border colour draws the inner hairlines */

.projects {
  margin-top: 24px;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1px;
  background: var(--border);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.project {
  display: grid;
  gap: 10px;
  align-content: start;
  padding: 24px 28px;
  background: var(--surface-raised);
}

.kind {
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

.project-title {
  font: 500 16px/22px var(--font-sans);
  color: var(--ink);
}

.blurb {
  font: var(--text-caption);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

@media (max-width: 900px) {
  .tracks,
  .projects { grid-template-columns: minmax(0, 1fr); }
}
</style>
