<script setup lang="ts">
const { isDark, toggle } = useTheme()

// the icon shows what a click gets you, not what is already on screen
const label = computed<string>(() => (isDark.value ? 'Switch to light mode' : 'Switch to dark mode'))

</script>

<template>
  <!-- Theme is client state, so the server has nothing correct to render. The
       fallback holds the same box so the header does not shift on hydration. -->
  <ClientOnly>
    <button
      type="button"
      class="flex size-9 cursor-pointer items-center justify-center rounded-md text-quiet transition-colors hover:text-ink"
      :title="label"
      :aria-label="label"
      :aria-pressed="isDark"
      @click="toggle"
    >
      <Transition name="icon" mode="out-in">
        <svg
          v-if="isDark"
          key="sun"
          class="size-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="4" />
          <path
            d="M12 2v2M12 20v2M5 5l1.5 1.5M17.5 17.5L19 19M2 12h2M20 12h2M5 19l1.5-1.5M17.5 6.5L19 5"
          />
        </svg>

        <svg
          v-else
          key="moon"
          class="size-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" />
        </svg>
      </Transition>
    </button>

    <template #fallback>
      <span class="size-9" aria-hidden="true" />
    </template>
  </ClientOnly>
</template>

<style scoped>
.icon-enter-active,
.icon-leave-active {
  transition:
    opacity 160ms ease,
    transform 160ms ease;
}

.icon-enter-from {
  opacity: 0;
  transform: rotate(-90deg) scale(0.6);
}

.icon-leave-to {
  opacity: 0;
  transform: rotate(90deg) scale(0.6);
}

@media (prefers-reduced-motion: reduce) {
  .icon-enter-active,
  .icon-leave-active {
    transition: none;
  }
}
</style>
