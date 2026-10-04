<script setup lang="ts">
const { isDark, toggle } = useTheme()

// the icon shows what a click gets you, not what is already on screen
const label = computed<string>(() => (isDark.value ? 'Switch to light mode' : 'Switch to dark mode'))

</script>

<template>
  <!-- Not rendered anywhere yet: dark mode waits on a checked palette. When it
       lands, this goes in SiteHeader's right zone. -->
  <ClientOnly>
    <button
      type="button"
      class="toggle"
      :title="label"
      :aria-label="label"
      :aria-pressed="isDark"
      @click="toggle"
    >
      {{ isDark ? 'light' : 'dark' }}
    </button>

    <template #fallback>
      <span class="toggle" aria-hidden="true" />
    </template>
  </ClientOnly>
</template>

<style scoped>
.toggle {
  display: inline-flex;
  align-items: center;
  height: 28px;
  padding: 0 var(--space-3);
  border: 0;
  border-radius: var(--radius-full);
  background: transparent;
  font: var(--text-label-mono);
  color: var(--ink-muted);
  cursor: pointer;
  transition: var(--transition-control);
}

.toggle:hover { color: var(--ink); background: var(--surface-sunken); }
</style>
