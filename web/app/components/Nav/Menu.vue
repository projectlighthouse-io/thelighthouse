<script setup lang="ts">
/**
 * The navigation, for screens too narrow to show it in the header.
 *
 * The header's own `<nav>` is `hidden sm:flex`, so below `sm` there has never
 * been any way to reach books, projects, syntax, blog or pricing except by
 * knowing the urls. This is that way.
 *
 * **`books` is a plain link here, not the shelf panel.** `NavBooksDropdown`
 * is three columns and a hover preview, which a phone has no room for and no
 * pointer to drive — on this size the honest answer to "books" is the books
 * page.
 */
interface NavLink {
  to: string
  label: string
}

defineProps<{ links: NavLink[] }>()

const open = ref<boolean>(false)
const root = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const panel = ref<HTMLElement | null>(null)

const close = (restoreFocus = false): void => {
  open.value = false

  if (restoreFocus) trigger.value?.focus()
}

const onPointerDown = (event: MouseEvent): void => {
  if (!open.value) return
  if (root.value?.contains(event.target as Node)) return
  if (panel.value?.contains(event.target as Node)) return

  close()
}

const onKeydown = (event: KeyboardEvent): void => {
  if (open.value && event.key === 'Escape') close(true)
}

onMounted(() => {
  document.addEventListener('pointerdown', onPointerDown)
  document.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onPointerDown)
  document.removeEventListener('keydown', onKeydown)
})

// A panel left hanging over the next page is one the reader has to dismiss
// twice — and here it would cover the page they just asked for.
const route = useRoute()
watch(() => route.fullPath, () => close())
</script>

<template>
  <div ref="root" class="relative sm:hidden">
    <button
      ref="trigger"
      type="button"
      class="flex cursor-pointer items-center rounded-md p-2 text-ink transition hover:text-link-hover"
      aria-haspopup="dialog"
      :aria-expanded="open"
      aria-label="Menu"
      @click="open = !open"
    >
      <svg class="size-5" viewBox="0 0 20 20" fill="none" aria-hidden="true">
        <path
          :d="open ? 'M5 5l10 10M15 5L5 15' : 'M3 6h14M3 10h14M3 14h14'"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
        />
      </svg>
    </button>

    <!-- Teleported for the reason `NavBooksDropdown` gives: a fixed element
         inside a transformed ancestor is positioned against that ancestor
         rather than the page. -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition-opacity duration-150 ease-out"
        leave-active-class="transition-opacity duration-150 ease-in"
        enter-from-class="opacity-0"
        leave-to-class="opacity-0"
      >
        <!-- Positioning on the outer element and the pencil border on the
             inner one, because `.border-pencil-light` sets `position: relative`
             and would otherwise beat `fixed` on source order — see
             `NavBooksDropdown` for what that bug looks like. -->
        <div
          v-if="open"
          ref="panel"
          class="fixed inset-x-4 top-20 z-50 sm:hidden"
          role="dialog"
          aria-label="Menu"
        >
          <div class="border-pencil-light overflow-hidden rounded-xl bg-panel p-2 shadow-2xl">
            <NuxtLink
              v-for="link in links"
              :key="link.to"
              :to="link.to"
              class="hover-border-pencil block rounded-md px-4 py-3 font-sans text-sm font-light text-ink"
            >
              {{ link.label }}
            </NuxtLink>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
