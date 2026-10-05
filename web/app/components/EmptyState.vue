<script setup lang="ts">
/**
 * The 404 and empty-state pattern: a mono eyebrow, a question for a heading,
 * a line of explanation, and one way forward. A second, quieter action goes
 * in the `secondary` slot when there genuinely is one.
 */
defineProps<{
  eyebrow: string
  heading: string
  detail?: string
  action?: { label: string, to: string }
  as?: 'h1' | 'h2'
}>()
</script>

<template>
  <div class="empty">
    <p class="lh-eyebrow">{{ eyebrow }}</p>
    <component :is="as ?? 'h2'" class="lh-h2">{{ heading }}</component>
    <p v-if="detail" class="lh-sub">{{ detail }}</p>
    <slot />
    <div v-if="action || $slots.secondary" class="actions">
      <UiButton v-if="action" variant="inverse" size="lg" :to="action.to">{{ action.label }}</UiButton>
      <slot name="secondary" />
    </div>
  </div>
</template>

<style scoped>
.empty {
  display: grid;
  gap: var(--space-4);
  justify-items: start;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  margin-top: var(--space-2);
}
</style>
