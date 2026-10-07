<script setup lang="ts" generic="K extends string">
/**
 * A pill track of tabs with a count beside each label. Client-side only: it
 * reports which key was chosen and the page decides what that shows.
 */
defineProps<{
  options: { key: K, label: string, count?: number }[]
  label: string
}>()

const model = defineModel<K>({ required: true })
</script>

<template>
  <div class="track" role="tablist" :aria-label="label">
    <button
      v-for="option in options"
      :key="option.key"
      type="button"
      role="tab"
      class="tab"
      :class="{ 'is-active': model === option.key }"
      :aria-selected="model === option.key"
      @click="model = option.key"
    >
      {{ option.label }}
      <span v-if="option.count !== undefined" class="count lh-num">{{ option.count }}</span>
    </button>
  </div>
</template>

<style scoped>
.track {
  display: inline-flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 2px;
  padding: 3px;
  border-radius: var(--radius-full);
  background: var(--surface-sunken);
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  height: 32px;
  padding: 0 var(--space-4);
  border: 0;
  border-radius: var(--radius-full);
  background: transparent;
  font: var(--text-caption);
  color: var(--ink-muted);
  cursor: pointer;
  transition: var(--transition-control);
}

.tab:hover { color: var(--ink); }

.tab.is-active {
  background: var(--surface-raised);
  color: var(--ink);
  box-shadow: var(--shadow-sm);
}

.count { font: var(--text-label-mono); color: var(--ink-muted); }

/* phones: one row across the full width, rather than a centred track that
   wraps its last tab onto a second line. Each tab sizes to its label and
   shares what is left — equal thirds clip "Challenges 12" at 360px. */
@media (max-width: 720px) {
  .track {
    display: flex;
    flex-wrap: nowrap;
    width: 100%;
  }

  .tab {
    flex: 1 1 auto;
    justify-content: center;
    gap: 6px;
    padding: 0 var(--space-2);
    white-space: nowrap;
  }
}
</style>
