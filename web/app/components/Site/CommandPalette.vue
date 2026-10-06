<script setup lang="ts">
import type { SearchEntry } from '@/types/Content'

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
  { kind: 'page', title: 'Books', context: '', to: '/books' },
  { kind: 'page', title: 'Pricing', context: '', to: '/pricing' },
  { kind: 'page', title: 'Profile', context: 'your progress', to: '/profile' },
  { kind: 'page', title: 'My notes', context: 'account', to: '/notes' },
  { kind: 'page', title: 'Settings', context: 'account', to: '/settings/profile' },
  { kind: 'page', title: 'Public profile', context: 'settings', to: '/settings/public-profile' },
  { kind: 'page', title: 'Billing', context: 'settings', to: '/settings/billing' },
  { kind: 'page', title: 'luxctl api tokens', context: 'settings', to: '/settings/tokens' },
]

// ponytail: render cap, not a ranking. Typing narrows it.
const LIMIT = 50

const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const input = useTemplateRef<HTMLInputElement>('input')
const list = useTemplateRef<HTMLUListElement>('list')

const query = ref('')
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

/** Every word typed has to appear somewhere in the title or where it sits. */
const results = computed<SearchEntry[]>(() => {
  const words = query.value.toLowerCase().split(/\s+/).filter(Boolean)
  const all = [...PAGES, ...(content.value ?? [])]
  const hits = words.length
    ? all.filter((entry) => {
        const text = `${entry.title} ${entry.context}`.toLowerCase()

        return words.every(word => text.includes(word))
      })
    : all

  return hits.slice(0, LIMIT)
})

watch(query, () => { active.value = 0 })

function open(): void {
  const el = dialog.value
  if (!el || el.open) return

  query.value = ''
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
  const count = results.value.length
  if (!count) return

  active.value = (active.value + by + count) % count
  nextTick(() => list.value?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' }))
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
  <dialog ref="dialog" class="palette" aria-label="search" @click="onClick">
    <input
      ref="input"
      v-model="query"
      class="query"
      type="search"
      placeholder="Search books, lessons, settings…"
      role="combobox"
      aria-controls="palette-results"
      aria-expanded="true"
      :aria-activedescendant="results.length ? `palette-${active}` : undefined"
      autocomplete="off"
      spellcheck="false"
      @keydown.down.prevent="move(1)"
      @keydown.up.prevent="move(-1)"
      @keydown.enter.prevent="go(results[active])"
    >

    <ul id="palette-results" ref="list" class="results" role="listbox">
      <li
        v-for="(entry, i) in results"
        :id="`palette-${i}`"
        :key="entry.to + entry.kind"
        class="row"
        role="option"
        :aria-selected="i === active"
        @mousemove="active = i"
        @click="go(entry)"
      >
        <span class="kind">{{ entry.kind }}</span>
        <span class="title">{{ entry.title }}</span>
        <span v-if="entry.context" class="context">{{ entry.context }}</span>
      </li>

      <li v-if="!results.length" class="empty">
        {{ content ? 'Nothing matches.' : 'Loading…' }}
      </li>
    </ul>
  </dialog>
</template>

<style scoped>
.palette {
  inset: 0;
  margin: 12vh auto auto;
  width: min(640px, calc(100vw - 32px));
  max-height: 70dvh;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-page);
  color: var(--ink);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.palette[open] { display: flex; flex-direction: column; }

.palette::backdrop {
  background: rgb(0 0 0 / 0.45);
  backdrop-filter: blur(2px);
}

.query {
  width: 100%;
  padding: var(--space-4) var(--space-5);
  border: 0;
  border-bottom: 1px solid var(--border);
  background: none;
  color: var(--ink);
  font: var(--text-body);
  outline: none;
}

.results {
  margin: 0;
  padding: var(--space-2);
  list-style: none;
  overflow-y: auto;
}

.row {
  display: grid;
  grid-template-columns: 64px minmax(0, 1fr);
  column-gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.row[aria-selected="true"] { background: var(--surface-sunken); }

.kind {
  grid-row: span 2;
  color: var(--ink-muted);
  font: var(--text-label-mono);
}

.title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context {
  grid-column: 2;
  overflow: hidden;
  color: var(--ink-secondary);
  font: var(--text-caption);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty {
  padding: var(--space-4) var(--space-3);
  color: var(--ink-secondary);
}
</style>
