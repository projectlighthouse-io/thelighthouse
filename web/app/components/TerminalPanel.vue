<script setup lang="ts">
/**
 * The dark terminal frame: a tag, a note, two lights, and whatever the body
 * slot prints. The hero's animated run and the project pages' commands both
 * sit in it.
 */
defineProps<{
  tag: string
  note?: string
}>()
</script>

<template>
  <div class="lh-terminal panel">
    <div class="bar">
      <span class="tag">{{ tag }}</span>
      <span v-if="note" class="note">{{ note }}</span>
      <span class="lights" aria-hidden="true">
        <span class="on" />
        <span class="off" />
      </span>
    </div>
    <slot />
  </div>
</template>

<style scoped>
.bar {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 18px 28px 6px;
}

.tag {
  padding: 4px 10px;
  border-radius: var(--radius-full);
  background: rgba(255, 255, 255, 0.08);
  color: var(--term-text);
  font: var(--text-label-mono);
}

.note {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font: var(--text-label-mono);
  color: var(--term-grey);
}

.lights {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.lights > span {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
}

.on { background: var(--term-green); }
.off { background: var(--term-line); }

@media (max-width: 600px) {
  .bar { padding: 16px 20px 6px; }
}
</style>
