<script setup lang="ts">
/**
 * The phone navigation: a floating bar of the four sections and the reader,
 * pinned to the bottom of the screen. Below 720px only — on anything wider it
 * is `display: none` and the header's own links do the job.
 *
 * It slides away after a moment of no input and comes back on any. The timer
 * runs at every width; on desktop it hides a bar nobody can see, which costs
 * one setTimeout and saves asking JS what the screen size is.
 */
interface Tab {
  to: string
  label: string
  /** The home-page section this tab stands for while scrolling there. */
  section: string
}

const tabs: Tab[] = [
  { to: '/books', label: 'books', section: 'shelf' },
  { to: '/projects', label: 'projects', section: 'build' },
  { to: '/pricing', label: 'pricing', section: 'pricing' },
  { to: '/blog', label: 'blog', section: '' },
]

const route = useRoute()
const { reader, isSignedIn, signOut } = useReader()

const sheetLinks = [
  { to: '/dashboard', label: 'my shelf' },
  { to: '/profile', label: 'progress' },
  { to: '/settings/billing', label: 'billing' },
]

const handle = computed<string>(() => {
  if (reader.value?.username) return `@${reader.value.username}`

  return reader.value?.name || reader.value?.email || 'account'
})

const initial = computed<string>(() =>
  (reader.value?.username || reader.value?.name || reader.value?.email || '?').charAt(0).toLowerCase(),
)

// ---------- which tab is lit ----------

/** The section the home page has scrolled to, or '' above the shelf. */
const section = ref('')
/** A tab just tapped, lit before the route or the scroll catches up. */
const tapped = ref('')

const active = computed<string>(() => {
  if (tapped.value) return tapped.value
  if (route.path === '/') return tabs.find(t => t.section && t.section === section.value)?.label ?? ''

  return tabs.find(t => route.path === t.to || route.path.startsWith(`${t.to}/`))?.label ?? ''
})

let spy: IntersectionObserver | null = null

/**
 * The last of the home sections whose top has passed 40% of the viewport.
 *
 * The observer's root is the top 40% of the screen, so it only calls back as
 * a section's top crosses that line — layout is read then, not every frame.
 */
function watchSections(): void {
  spy?.disconnect()
  spy = null
  section.value = ''
  if (route.path !== '/') return

  const els = tabs
    .map(t => (t.section ? document.getElementById(t.section) : null))
    .filter((el): el is HTMLElement => el !== null)

  const pick = (): void => {
    const line = window.innerHeight * 0.4
    section.value = els.filter(el => el.getBoundingClientRect().top < line).at(-1)?.id ?? ''
  }

  spy = new IntersectionObserver(pick, { rootMargin: '0px 0px -60% 0px' })
  els.forEach(el => spy?.observe(el))
  pick()
}

watch(() => route.path, () => {
  tapped.value = ''
  closeSheet()
  nextTick(watchSections)
})

function onTab(tab: Tab): void {
  tapped.value = tab.label
  closeSheet()
}

// ---------- the account sheet ----------

const join = useTemplateRef<{ open: (event?: MouseEvent) => void }>('join')
const sheetOpen = ref(false)
const bar = useTemplateRef<HTMLElement>('bar')
const sheet = useTemplateRef<HTMLElement>('sheet')
const avatar = useTemplateRef<HTMLButtonElement>('avatar')

function closeSheet(refocus = false): void {
  if (!sheetOpen.value) return
  sheetOpen.value = false
  if (refocus) avatar.value?.focus()
}

function toggleSheet(): void {
  if (sheetOpen.value) closeSheet(true)
  else sheetOpen.value = true
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Node
  if (bar.value?.contains(target) || sheet.value?.contains(target)) return
  closeSheet(true)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') closeSheet(true)
}

async function onSignOut(): Promise<void> {
  closeSheet(true)
  await signOut()
}

// ---------- auto-hide ----------

const IDLE_MS = 2500
const WAKE_EVENTS = ['scroll', 'touchstart', 'touchmove', 'pointermove', 'wheel', 'keydown'] as const

const hidden = ref(false)
let idle: ReturnType<typeof setTimeout> | undefined

function focusInside(): boolean {
  const el = document.activeElement
  return !!el && !!(bar.value?.contains(el) || sheet.value?.contains(el))
}

function wake(): void {
  hidden.value = false
  clearTimeout(idle)
  idle = setTimeout(() => {
    if (!sheetOpen.value && !focusInside()) hidden.value = true
  }, IDLE_MS)
}

// An open sheet holds the bar up; closing it starts the clock again.
watch(sheetOpen, wake)

onMounted(() => {
  WAKE_EVENTS.forEach(e => window.addEventListener(e, wake, { passive: true }))
  document.addEventListener('pointerdown', onPointerDown, { passive: true })
  document.addEventListener('keydown', onKeydown)
  wake()
  watchSections()
})

onBeforeUnmount(() => {
  WAKE_EVENTS.forEach(e => window.removeEventListener(e, wake))
  document.removeEventListener('pointerdown', onPointerDown)
  document.removeEventListener('keydown', onKeydown)
  clearTimeout(idle)
  spy?.disconnect()
})
</script>

<template>
  <nav
    ref="bar"
    class="bar"
    :class="{ 'is-hidden': hidden }"
    aria-label="Primary"
    :inert="hidden || undefined"
  >
    <NuxtLink
      v-for="tab in tabs"
      :key="tab.to"
      :to="tab.to"
      class="tab"
      :class="{ 'is-active': active === tab.label }"
      :aria-current="active === tab.label ? 'page' : undefined"
      @click="onTab(tab)"
    >
      {{ tab.label }}
    </NuxtLink>

    <span class="divider" aria-hidden="true" />

    <ClientOnly>
      <button
        v-if="isSignedIn"
        ref="avatar"
        type="button"
        class="avatar"
        aria-label="Account"
        :aria-expanded="sheetOpen"
        aria-controls="account-sheet"
        @click="toggleSheet"
      >
        <span class="initial">{{ initial }}</span>
      </button>

      <button v-else type="button" class="join-btn" @click="join?.open($event)">join</button>

      <!-- Before hydration there is no dialog to open; /login is the same
           sign-in on its own page. -->
      <template #fallback>
        <NuxtLink to="/login" class="join-btn">join</NuxtLink>
      </template>
    </ClientOnly>
  </nav>

  <SiteJoinDialog ref="join" />

  <div v-if="sheetOpen" id="account-sheet" ref="sheet" class="sheet">
    <div class="sheet-handle">{{ handle }}</div>
    <NuxtLink
      v-for="item in sheetLinks"
      :key="item.to"
      :to="item.to"
      class="row"
      @click="closeSheet(true)"
    >
      {{ item.label }}
    </NuxtLink>
    <div class="sheet-divider" />
    <button type="button" class="row row-quiet" @click="onSignOut">sign out</button>
  </div>
</template>

<style scoped>
.bar,
.sheet { display: none; }

@media (max-width: 720px) {
  .bar {
    position: fixed;
    left: 12px;
    right: 12px;
    bottom: calc(12px + env(safe-area-inset-bottom));
    z-index: 40;
    display: flex;
    align-items: center;
    gap: 2px;
    box-sizing: border-box;
    padding: 4px;
    /* Concentric: 12px tabs + 4px padding = 16px. Change one, change both. */
    border: 1px solid var(--ink);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    box-shadow: var(--shadow-lg);
    transition: transform 0.35s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.35s ease-out;
  }

  .bar.is-hidden {
    transform: translateY(calc(100% + 24px));
    pointer-events: none;
  }

  .sheet {
    position: fixed;
    left: 12px;
    right: 12px;
    bottom: calc(68px + env(safe-area-inset-bottom));
    z-index: 39;
    display: flex;
    flex-direction: column;
    padding: 8px;
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    box-shadow: var(--shadow-lg);
  }
}

@media (max-width: 720px) and (prefers-reduced-motion: reduce) {
  .bar { transition: opacity 0.35s ease-out; }
  .bar.is-hidden { transform: none; opacity: 0; }
}

.tab {
  flex: 1 1 0;
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 40px;
  padding: 0 8px;
  border-radius: var(--radius-md);
  background: transparent;
  font: var(--text-caption);
  white-space: nowrap;
  color: var(--ink-secondary);
  text-decoration: none;
  transition: background 0.2s ease-out, color 0.2s ease-out;
}

.tab:hover { color: var(--ink); }

.tab.is-active {
  background: var(--surface-sunken);
  color: var(--ink);
}

.divider {
  flex: none;
  width: 1px;
  height: 20px;
  margin: 0 4px;
  background: var(--border);
}

.join-btn {
  flex: none;
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 16px;
  border-radius: var(--radius-md);
  background: var(--surface-inverse);
  color: var(--ink-inverse);
  font: var(--text-caption);
  font-weight: 500;
  white-space: nowrap;
  border: 0;
  text-decoration: none;
  cursor: pointer;
}

.join-btn:hover { opacity: 0.9; color: var(--ink-inverse); }

.avatar {
  flex: none;
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  padding: 0;
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  cursor: pointer;
}

.avatar:hover { background: var(--surface-sunken); }

.initial {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  background: var(--surface-inverse);
  color: var(--ink-inverse);
  font: var(--text-label-mono);
}

.sheet-handle {
  padding: 8px 12px 4px;
  font: var(--text-label-mono);
  color: var(--ink-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row {
  display: flex;
  align-items: center;
  min-height: 48px;
  padding: 0 12px;
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  font: var(--text-body);
  color: var(--ink);
  text-align: left;
  text-decoration: none;
  cursor: pointer;
}

.row:hover { background: var(--surface-sunken); color: var(--ink); }

.row-quiet { color: var(--ink-muted); }

.sheet-divider {
  height: 1px;
  margin: 4px 12px;
  background: var(--border);
}

.tab:focus-visible,
.join-btn:focus-visible,
.avatar:focus-visible,
.row:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}
</style>
