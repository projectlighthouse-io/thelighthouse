<script setup lang="ts">
/**
 * The three-zone header every page opens on: logo, the three sections, and
 * whoever is reading.
 *
 * Who the reader is is client state, so the server always renders the join
 * button — one cached document for everyone — and the browser swaps in the
 * account menu once `useReader` knows. The `lh_reader` hint keeps that swap
 * from being a visible flash for somebody who is signed in.
 */
interface NavLink {
  to: string
  label: string
}

const links: NavLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/pricing', label: 'pricing' },
]

const menu: NavLink[] = [
  { to: '/dashboard', label: 'my shelf' },
  { to: '/profile', label: 'progress' },
  { to: '/settings/billing', label: 'billing' },
]

const route = useRoute()
const { reader, isSignedIn, signOut } = useReader()

const isActive = (link: NavLink): boolean =>
  route.path === link.to || route.path.startsWith(`${link.to}/`)

/** `@handle`, or the name for an account that has not picked one yet. */
const handle = computed<string>(() => {
  if (reader.value?.username) return `@${reader.value.username}`

  return reader.value?.name || reader.value?.email || 'account'
})

const open = ref(false)
const root = useTemplateRef<HTMLElement>('root')
const toggle = useTemplateRef<HTMLButtonElement>('toggle')

function close(refocus = false): void {
  open.value = false
  if (refocus) toggle.value?.focus()
}

function onPointerDown(event: PointerEvent): void {
  if (!root.value?.contains(event.target as Node)) close()
}

onMounted(() => document.addEventListener('pointerdown', onPointerDown))
onBeforeUnmount(() => document.removeEventListener('pointerdown', onPointerDown))

watch(() => route.fullPath, () => close())

async function onSignOut(): Promise<void> {
  close()
  await signOut()
}
</script>

<template>
  <header class="header lh-figure">
    <NuxtLink to="/" class="brand">
      <img src="/lighthouse.svg" alt="" width="22" height="22">
      <span class="wordmark">projectlighthouse</span>
      <span class="lh-sr">home</span>
    </NuxtLink>

    <nav class="nav" aria-label="primary">
      <NuxtLink
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        class="nav-link"
        :class="{ 'is-active': isActive(link) }"
        :aria-current="isActive(link) ? 'page' : undefined"
      >
        {{ link.label }}
      </NuxtLink>
    </nav>

    <div class="who">
      <ClientOnly>
        <div v-if="isSignedIn" ref="root" class="account" @keydown.esc="close(true)">
          <button
            ref="toggle"
            type="button"
            class="pill"
            aria-haspopup="menu"
            :aria-expanded="open"
            @click="open = !open"
          >
            <span class="handle">{{ handle }}</span>
            <span class="caret" aria-hidden="true">▾</span>
          </button>

          <div v-if="open" class="menu" role="menu">
            <NuxtLink
              v-for="item in menu"
              :key="item.to"
              :to="item.to"
              class="item"
              role="menuitem"
            >
              {{ item.label }}
            </NuxtLink>
            <button type="button" class="item item-quiet" role="menuitem" @click="onSignOut">
              sign out
            </button>
          </div>
        </div>

        <UiButton v-else variant="inverse" size="sm" to="/pricing">join</UiButton>

        <template #fallback>
          <UiButton variant="inverse" size="sm" to="/pricing">join</UiButton>
        </template>
      </ClientOnly>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: var(--space-6);
  padding-top: var(--space-5);
  padding-bottom: var(--space-5);
}

.brand {
  justify-self: start;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--ink);
  text-decoration: none;
}

.brand:hover { color: var(--ink); }

.brand img { width: 22px; height: 22px; }

.wordmark { font: var(--text-label-mono); color: var(--ink); }

.nav {
  display: flex;
  align-items: center;
  gap: 28px;
}

.nav-link {
  font: var(--text-caption);
  color: var(--ink-secondary);
  text-decoration: none;
  transition: color var(--duration) var(--ease-out);
}

.nav-link:hover,
.nav-link.is-active { color: var(--ink); }

.who {
  justify-self: end;
  display: flex;
  align-items: center;
  min-width: 0;
}

.account { position: relative; }

.pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  max-width: 220px;
  padding: 0 10px 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-full);
  background: var(--surface-raised);
  font: var(--text-label-mono);
  color: var(--ink);
  cursor: pointer;
  transition: var(--transition-control);
}

.pill:hover { background: var(--surface-sunken); }

.handle {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.caret { color: var(--ink-muted); }

.menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  z-index: 20;
  display: grid;
  gap: 2px;
  min-width: 180px;
  padding: 6px;
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  box-shadow: var(--shadow-md);
}

.item {
  display: block;
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  font: var(--text-caption);
  color: var(--ink);
  text-align: left;
  text-decoration: none;
  cursor: pointer;
  transition: background-color var(--duration) var(--ease-out);
}

.item:hover { background: var(--surface-sunken); color: var(--ink); }

.item-quiet { color: var(--ink-muted); }

/* phones: logo mark only, the nav takes the middle and gives way first,
   the handle truncates rather than running into it */
@media (max-width: 560px) {
  .header { grid-template-columns: auto minmax(0, 1fr) auto; gap: var(--space-3); }
  .wordmark { display: none; }
  .nav { justify-content: center; gap: var(--space-3); min-width: 0; }
  .pill { max-width: 104px; }
}
</style>
