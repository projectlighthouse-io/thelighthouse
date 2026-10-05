<script setup lang="ts">
import type { Manuscript } from '@/types/Content'

/**
 * The desk: books being written, one open at a time to its outline and a line
 * from the draft. Shared by the home page and the roadmap.
 */
const props = withDefaults(
  defineProps<{
    manuscripts: Manuscript[]
    eyebrow?: string
    heading?: 'h1' | 'h2'
  }>(),
  { eyebrow: '05 — on the desk', heading: 'h2' },
)

const open = ref<number>(0)

function toggle(i: number): void {
  open.value = open.value === i ? -1 : i
}

const written = computed<number>(() => props.manuscripts.reduce((n, m) => n + m.written, 0))
const planned = computed<number>(() => props.manuscripts.reduce((n, m) => n + m.chapters.length, 0))

const STATUS: Record<Manuscript['status'], string> = {
  next: 'next on the shelf',
  drafting: 'drafting',
  outlined: 'outlined',
}

const pad = (n: number): string => String(n).padStart(2, '0')

// The date as written until the page is running in a browser, then relative.
// Relative on the server would be frozen at whenever the page was prerendered.
const now = ref<number | null>(null)
onMounted(() => { now.value = Date.now() })

function edited(iso: string): string {
  const at = new Date(`${iso}T00:00:00Z`)

  if (now.value === null) {
    return at.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' })
  }

  const days = Math.max(0, Math.floor((now.value - at.getTime()) / 86_400_000))

  if (days === 0) return 'today'
  if (days === 1) return 'yesterday'
  if (days < 14) return `${days} days ago`
  if (days < 60) return `${Math.floor(days / 7)} weeks ago`

  return `${Math.floor(days / 30)} months ago`
}
</script>

<template>
  <section id="horizon" class="desk-section lh-wide lh-gap-lg">
    <div class="desk">
      <div class="intro">
        <p class="lh-eyebrow">{{ eyebrow }}</p>
        <component :is="heading" class="lh-display">
          What is being written <em class="now">now.</em>
        </component>
        <p class="lh-lede">
          The shelf covers how your program works and how networks work. These close the gaps
          between them. Chapters land as they are finished.
        </p>
        <dl class="counts">
          <div>
            <dt class="lh-eyebrow">chapters written</dt>
            <dd class="lh-h1 lh-num">{{ written }}</dd>
          </div>
          <div>
            <dt class="lh-eyebrow">planned</dt>
            <dd class="lh-h1 lh-num lh-muted">{{ planned }}</dd>
          </div>
        </dl>
      </div>

      <div class="list">
        <div v-for="(m, i) in manuscripts" :key="m.title" class="manuscript">
          <button
            :id="`desk-head-${i}`"
            type="button"
            class="head"
            :aria-expanded="open === i"
            :aria-controls="`desk-body-${i}`"
            @click="toggle(i)"
          >
            <span class="lh-h2">{{ m.title }}</span>
            <span class="status" :class="`is-${m.status}`">{{ STATUS[m.status] }}</span>
            <span class="lh-ticks ticks" aria-hidden="true">
              <span v-for="(c, k) in m.chapters" :key="c" :class="{ 'is-done': k < m.written }" />
            </span>
            <span class="progress lh-num">{{ m.written }} of {{ m.chapters.length }} chapters</span>
          </button>

          <div
            v-if="open === i"
            :id="`desk-body-${i}`"
            class="body"
            role="region"
            :aria-labelledby="`desk-head-${i}`"
          >
            <ol class="chapters">
              <li v-for="(c, k) in m.chapters" :key="c" :class="{ 'is-written': k < m.written }">
                <span class="n lh-num">{{ pad(k + 1) }}</span>
                <span>{{ c }}</span>
              </li>
            </ol>

            <div class="excerpt">
              <span class="lh-eyebrow">{{ m.excerpt ? `from the draft · ${m.excerpt.from}` : m.description }}</span>
              <p v-if="m.excerpt" class="quote">{{ m.excerpt.text }}</p>
              <span class="lh-mono lh-faint">last edited {{ edited(m.editedAt) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.desk {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr);
  gap: var(--space-16);
  align-items: start;
}

.intro {
  position: sticky;
  top: var(--space-12);
  display: grid;
  gap: var(--space-6);
}

/* the one italic on the home page, by design */
.now {
  font: var(--text-display-serif);
  font-size: inherit;
  line-height: inherit;
  color: var(--ink-muted);
}

.counts {
  margin: var(--space-2) 0 0;
  display: grid;
  grid-auto-flow: column;
  justify-content: start;
  gap: 40px;
}

.counts > div {
  display: flex;
  flex-direction: column-reverse;
  gap: 2px;
}

.counts dd { margin: 0; }

.list {
  display: grid;
  gap: var(--space-3);
  min-width: 0;
}

.manuscript {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  overflow: hidden;
}

.head {
  width: 100%;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2) var(--space-6);
  align-items: center;
  padding: var(--space-6) 28px;
  border: 0;
  background: transparent;
  color: var(--ink);
  text-align: left;
  cursor: pointer;
  transition: background-color var(--duration) var(--ease-out);
}

.head:hover { background: var(--surface-sunken); }

.status {
  justify-self: end;
  font: var(--text-label-mono);
  white-space: nowrap;
}

.is-next { color: var(--accent-strong); }
.is-drafting { color: var(--ink-muted); }
.is-outlined { color: var(--ink-faint); }

.ticks { grid-column: 1 / -1; margin-top: var(--space-2); }

.progress {
  grid-column: 1 / -1;
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr);
  gap: 40px;
  padding: var(--space-2) 28px var(--space-8);
}

.chapters {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 10px;
  align-content: start;
}

.chapters li {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: var(--space-2);
  font: var(--text-body-sm);
  color: var(--ink-faint);
}

.chapters li.is-written { color: var(--ink); }

.n {
  padding-top: 4px;
  font: var(--text-label-mono);
  color: var(--ink-faint);
}

.excerpt {
  display: grid;
  gap: var(--space-4);
  align-content: start;
  padding: var(--space-5) var(--space-6);
  border-radius: var(--radius-md);
  background: var(--surface-sunken);
}

.quote {
  margin: 0;
  font: var(--text-quote);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

@media (max-width: 960px) {
  .desk { grid-template-columns: minmax(0, 1fr); }
  .intro { position: static; }
}

@media (max-width: 640px) {
  .body { grid-template-columns: minmax(0, 1fr); }
}

@media (max-width: 600px) {
  .head { padding: var(--space-5); }
  .body { padding: var(--space-2) var(--space-5) var(--space-6); }
  .intro :deep(.lh-display) { font-size: 44px; line-height: 48px; }
}
</style>
