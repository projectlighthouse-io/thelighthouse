<script setup lang="ts">
import type { SearchEntry } from '@/types/Content'
import type { SearchFilter, SearchGroup } from '@/utils/Search'
import { groupEntries, matchEntries } from '@/utils/Search'

/**
 * ⌘K / Ctrl+K: jump to any book, chapter, lesson or account page.
 *
 * Content comes from `/_api/search` in one request, made on the first open and
 * never again for the life of the tab — every keystroke after that filters in
 * memory. A page that never opens the palette never asks.
 *
 * A native <dialog>, as `SiteJoinDialog` is: backdrop, focus trap and Escape
 * come free.
 */
const PAGES: SearchEntry[] = [
  { kind: 'page', title: 'Books', description: '', group: 'pages', to: '/books' },
  { kind: 'page', title: 'Pricing', description: '', group: 'pages', to: '/pricing' },
  { kind: 'setting', title: 'Profile', description: 'Your progress.', group: 'settings', to: '/profile' },
  { kind: 'setting', title: 'My notes', description: '', group: 'settings', to: '/notes' },
  { kind: 'setting', title: 'Settings', description: '', group: 'settings', to: '/settings/profile' },
  { kind: 'setting', title: 'Public profile', description: '', group: 'settings', to: '/settings/public-profile' },
  { kind: 'setting', title: 'Billing and plan', description: '', group: 'settings', to: '/settings/billing' },
  { kind: 'setting', title: 'luxctl api tokens', description: '', group: 'settings', to: '/settings/tokens' },
]

const FILTERS: { key: SearchFilter, label: string }[] = [
  { key: 'all', label: 'all' },
  { key: 'book', label: 'books' },
  { key: 'chapter', label: 'chapters' },
  { key: 'lesson', label: 'lessons' },
  { key: 'setting', label: 'settings' },
]

// ponytail: render cap, not a ranking. Typing narrows it; the count says all.
const LIMIT = 60

const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const input = useTemplateRef<HTMLInputElement>('input')
const list = useTemplateRef<HTMLDivElement>('list')

const query = ref('')
const filter = ref<SearchFilter>('all')
const active = ref(0)
const content = ref<SearchEntry[] | null>(null)
let loading: Promise<void> | null = null

function load(): Promise<void> {
  loading ??= $fetch<SearchEntry[]>('/_api/search')
    .then((entries) => { content.value = entries })
    // A failed load leaves the pages searchable and lets the next open retry.
    .catch(() => { loading = null })

  return loading
}

const matches = computed<SearchEntry[]>(() =>
  matchEntries([...PAGES, ...(content.value ?? [])], query.value, filter.value))

const groups = computed<SearchGroup[]>(() => groupEntries(matches.value.slice(0, LIMIT)))

const rows = computed<SearchEntry[]>(() => groups.value.flatMap(g => g.items.map(i => i.entry)))

const count = computed<string>(() => {
  const n = matches.value.length

  return `${n} ${n === 1 ? 'result' : 'results'}`
})

watch([query, filter], () => { active.value = 0 })

function open(): void {
  const el = dialog.value
  if (!el || el.open) return

  query.value = ''
  filter.value = 'all'
  active.value = 0
  el.showModal()
  input.value?.focus()
  load()
}

function close(): void {
  dialog.value?.close()
}

function go(entry: SearchEntry | undefined): void {
  if (!entry) return

  close()
  navigateTo(entry.to)
}

function move(by: number): void {
  const last = rows.value.length - 1
  if (last < 0) return

  active.value = Math.min(Math.max(active.value + by, 0), last)
  nextTick(() => list.value?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' }))
}

/** Tab walks the filters rather than leaving the input. */
function cycle(by: number): void {
  const at = FILTERS.findIndex(f => f.key === filter.value)
  const next = FILTERS[(at + by + FILTERS.length) % FILTERS.length]
  if (next) filter.value = next.key
}

/** Escape clears a query first, and closes only an empty palette. */
function onEscape(event: KeyboardEvent): void {
  if (!query.value) return

  event.preventDefault()
  query.value = ''
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key.toLowerCase() !== 'k' || !(event.metaKey || event.ctrlKey)) return

  event.preventDefault()
  if (dialog.value?.open) close()
  else open()
}

function onClick(event: MouseEvent): void {
  if (event.target === dialog.value) close()
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))

defineExpose({ open })
</script>

<template>
  <dialog ref="dialog" class="palette" aria-label="Search" @click="onClick">
    <div class="bar">
      <input
        ref="input"
        v-model="query"
        class="query"
        type="text"
        placeholder="Search books, lessons, settings"
        role="combobox"
        aria-controls="palette-results"
        aria-expanded="true"
        :aria-activedescendant="rows.length ? `palette-${active}` : undefined"
        autocomplete="off"
        spellcheck="false"
        @keydown.down.prevent="move(1)"
        @keydown.up.prevent="move(-1)"
        @keydown.enter.prevent="go(rows[active])"
        @keydown.tab.exact.prevent="cycle(1)"
        @keydown.shift.tab.prevent="cycle(-1)"
        @keydown.esc="onEscape"
      >
      <span class="hint">esc</span>
    </div>

    <div class="filters" role="tablist" aria-label="filter">
      <button
        v-for="f in FILTERS"
        :key="f.key"
        type="button"
        role="tab"
        class="filter"
        :aria-selected="filter === f.key"
        tabindex="-1"
        @click="filter = f.key; input?.focus()"
      >
        {{ f.label }}
      </button>
    </div>

    <div id="palette-results" ref="list" class="results" role="listbox">
      <div v-for="group in groups" :key="group.label" role="group" :aria-label="group.label">
        <p class="group">{{ group.label }}</p>
        <div
          v-for="{ entry, at } in group.items"
          :id="`palette-${at}`"
          :key="entry.to + entry.kind"
          class="row"
          :style="{ '--at': at }"
          role="option"
          :aria-selected="at === active"
          @mousemove="active = at"
          @click="go(entry)"
        >
          <span class="title">{{ entry.title }}</span>
          <span v-if="entry.description" class="description">{{ entry.description }}</span>
        </div>
      </div>

      <div v-if="!rows.length" class="empty">
        <template v-if="content || query">
          <p class="empty-title">Nothing matches “{{ query }}”</p>
          <p class="empty-hint">Try a book, chapter or lesson name.</p>
        </template>
        <p v-else class="empty-hint">Loading…</p>
      </div>
    </div>

    <div class="foot">
      <div class="keys">
        <span>↑↓ move</span>
        <span>↵ open</span>
        <span>tab filter</span>
      </div>
      <span>{{ count }}</span>
    </div>
  </dialog>
</template>

<style scoped>
.palette {
  inset: 0;
  margin: 72px auto auto;
  width: min(640px, calc(100vw - 32px));
  max-height: calc(100dvh - 144px);
  padding: 0;
  border: 0;
  border-radius: var(--radius-lg);
  background: var(--surface-raised);
  color: var(--ink);
  box-shadow: var(--shadow-lg);
  font-family: var(--font-sans);
  overflow: hidden;
}

/*
 * In: the house entrance (motion.css) — up from a touch small and blurred, at
 * the slow duration. Out: quieter, a fade and a slight shrink at the fast one.
 * `allow-discrete` keeps `display` and the top layer until the exit finishes;
 * a browser without it just opens and closes without the motion.
 */
.palette {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
  filter: blur(2px);
  transition:
    opacity var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out),
    filter var(--duration-fast) var(--ease-out),
    display var(--duration-fast) allow-discrete,
    overlay var(--duration-fast) allow-discrete;
}

.palette[open] {
  display: flex;
  flex-direction: column;
  opacity: 1;
  transform: none;
  filter: none;
  transition-duration: var(--duration-slow);
}

@starting-style {
  .palette[open] {
    opacity: 0;
    transform: translateY(-12px) scale(0.96);
    filter: blur(var(--enter-blur));
  }
}

.palette::backdrop {
  background: rgb(0 0 0 / 0);
  backdrop-filter: blur(0);
  transition:
    background-color var(--duration-fast) var(--ease-out),
    backdrop-filter var(--duration-fast) var(--ease-out),
    display var(--duration-fast) allow-discrete,
    overlay var(--duration-fast) allow-discrete;
}

.palette[open]::backdrop {
  background: rgb(0 0 0 / 0.45);
  backdrop-filter: blur(2px);
  transition-duration: var(--duration-slow);
}

@starting-style {
  .palette[open]::backdrop {
    background: rgb(0 0 0 / 0);
    backdrop-filter: blur(0);
  }
}

.bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
}

.query {
  flex: 1;
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--ink);
  font: 400 15px/24px var(--font-sans);
}

.query::placeholder { color: var(--ink-faint); }

.hint,
.foot {
  font: 400 10px/16px var(--font-mono);
}

.hint { color: var(--ink-faint); }

.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  padding: 0 20px 12px;
}

.filter {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--ink-faint);
  font: 400 11px/18px var(--font-mono);
  cursor: pointer;
  transition: color 0.2s ease-out;
}

.filter:hover { color: var(--ink-secondary); }

/* The underline is drawn, not decorated, so it can grow from the centre. */
.filter {
  position: relative;
}

.filter::after {
  content: '';
  position: absolute;
  right: 0;
  bottom: -4px;
  left: 0;
  height: 2px;
  border-radius: 1px;
  background: var(--accent);
  transform: scaleX(0);
  transition: transform var(--duration) var(--ease-out);
}

.filter[aria-selected="true"] { color: var(--ink); }

.filter[aria-selected="true"]::after { transform: scaleX(1); }

.results {
  max-height: 440px;
  padding: 8px;
  overflow-y: auto;
}

.group {
  margin: 0;
  padding: 12px 12px 6px;
  color: var(--ink-muted);
  font: 400 11px/16px var(--font-mono);
}

.row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s ease-out;
}

.row[aria-selected="true"] { background: var(--surface-sunken); }

/* A row that appears — on open, or when typing brings it in — rises into
   place. The first ten stagger; the rest are below the fold anyway. */
.row {
  animation: rise var(--duration) var(--ease-out) both;
  animation-delay: calc(min(var(--at), 10) * 16ms);
}

@keyframes rise {
  from { opacity: 0; transform: translateY(4px); }
}

.title,
.description {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title {
  color: var(--ink);
  font-size: 13px;
  line-height: 20px;
}

.description {
  color: var(--ink-muted);
  font-size: 12px;
  line-height: 17px;
}

.empty {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 40px 12px;
  text-align: center;
}

.empty p { margin: 0; }

.empty-title { color: var(--ink); font-size: 13px; }

.empty-hint { color: var(--ink-muted); font-size: 12px; }

.foot {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 20px;
  color: var(--ink-muted);
}

.keys { display: flex; gap: 16px; }

/* motion.css zeroes the durations; the row delay and the blur go too. */
@media (prefers-reduced-motion: reduce) {
  .row { animation: none; }
  .palette, .palette[open] { filter: none; transform: none; }
}
</style>
