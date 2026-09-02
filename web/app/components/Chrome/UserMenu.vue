<script setup lang="ts">
/**
 * The signed-in reader's menu in the header.
 *
 * Click to open, not hover. The join panel next to it opens on hover because it
 * is an invitation — drifting into it costs a reader nothing. This one holds
 * sign-out, and a menu that appears under a passing cursor puts a destructive
 * action where nobody asked for it.
 */
const { user, initials, signOut } = useAuth()

const open = ref(false)
const root = useTemplateRef<HTMLElement>('root')

interface MenuLink {
  to: string
  label: string
}

const links: MenuLink[] = [
  { to: '/dashboard', label: 'dashboard' },
  { to: '/notes', label: 'notes' },
  { to: '/profile', label: 'profile' },
  { to: '/settings/profile', label: 'settings' },
]

function close() {
  open.value = false
}

// Pointerdown rather than click: a click that starts inside the menu and ends
// outside — a drag, a text selection — should not be read as dismissing it.
function onPointerDown(event: PointerEvent) {
  if (!root.value?.contains(event.target as Node)) close()
}

onMounted(() => document.addEventListener('pointerdown', onPointerDown))
onBeforeUnmount(() => document.removeEventListener('pointerdown', onPointerDown))

// Navigating away closes it; otherwise it hangs over the page it opened from.
watch(() => useRoute().fullPath, close)

async function onSignOut() {
  close()
  await signOut()
}
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      aria-haspopup="menu"
      :aria-expanded="open"
      class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition hover:bg-paper-warm"
      :title="user?.email ?? undefined"
      @click="open = !open"
      @keydown.esc="close"
    >
      <img
        v-if="user?.avatar"
        :src="user.avatar"
        alt=""
        class="size-7 rounded-full object-cover"
        referrerpolicy="no-referrer"
      >
      <span
        v-else
        class="flex size-7 items-center justify-center rounded-full bg-ink font-mono text-xs text-on-ink"
      >{{ initials }}</span>

      <span class="hidden font-sans text-sm text-ink sm:inline">
        {{ user?.name ?? user?.email }}
      </span>

      <svg
        class="size-3 text-faint transition-transform"
        :class="{ 'rotate-180': open }"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M3 4.5L6 7.5L9 4.5"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <Transition name="menu">
      <div
        v-if="open"
        role="menu"
        class="absolute right-0 z-50 mt-2 w-56 overflow-hidden rounded-lg border border-stroke bg-panel shadow-lg"
        @keydown.esc="close"
      >
        <!-- Who you are, so a shared machine cannot leave you acting as someone
             else without noticing. The button itself truncates on small screens. -->
        <div class="border-b border-rule px-4 py-3">
          <div class="truncate font-sans text-sm text-ink">{{ user?.name ?? 'signed in' }}</div>
          <div class="truncate font-mono text-xs text-quiet">{{ user?.email }}</div>
        </div>

        <nav class="py-1">
          <NuxtLink
            v-for="link in links"
            :key="link.to"
            :to="link.to"
            role="menuitem"
            class="block px-4 py-2 font-sans text-sm text-ink transition hover:bg-paper-warm"
            @click="close"
          >
            {{ link.label }}
          </NuxtLink>
        </nav>

        <div class="border-t border-rule py-1">
          <button
            type="button"
            role="menuitem"
            class="block w-full cursor-pointer px-4 py-2 text-left font-sans text-sm text-quiet transition hover:bg-paper-warm hover:text-ink"
            @click="onSignOut"
          >
            sign out
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Enter carries a small drop so it reads as coming from the button; leave is a
   plain fade, because dismissing should feel immediate rather than animated at
   you. */
.menu-enter-active {
  transition:
    opacity 140ms ease,
    transform 140ms cubic-bezier(0.22, 1, 0.36, 1);
}

.menu-leave-active {
  transition: opacity 100ms ease;
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
}

.menu-enter-from {
  transform: translateY(-4px);
}

@media (prefers-reduced-motion: reduce) {
  .menu-enter-active,
  .menu-leave-active {
    transition: none !important;
    transform: none !important;
  }
}
</style>
