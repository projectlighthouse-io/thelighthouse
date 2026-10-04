<script setup lang="ts">
/**
 * The head a book or project page opens on: mono eyebrow, serif title,
 * description, the CTAs, hashtags — and a plate on the right, the cover by
 * default or whatever the `figure` slot puts there.
 */
defineProps<{
  eyebrow: string
  title: string
  description?: string
  topics?: string[]
  cover?: string
  coverAlt?: string
}>()
</script>

<template>
  <div class="head lh-figure" :class="{ 'is-bare': !$slots.figure && !cover }">
    <div class="words">
      <p class="lh-eyebrow">{{ eyebrow }}</p>
      <h1 class="lh-title-serif">{{ title }}</h1>
      <p v-if="description" class="lh-lede">{{ description }}</p>

      <div v-if="$slots.actions" class="actions">
        <slot name="actions" />
      </div>

      <ul v-if="topics?.length" class="topics" aria-label="topics">
        <li v-for="topic in topics" :key="topic">#{{ topic }}</li>
      </ul>
    </div>

    <div v-if="$slots.figure || cover" class="figure">
      <slot name="figure">
        <div class="plate">
          <div class="plate-inner">
            <img :src="cover" :alt="coverAlt ?? ''" width="600" height="400">
          </div>
        </div>
      </slot>
    </div>
  </div>
</template>

<style scoped>
.head {
  margin-top: var(--space-16);
  display: grid;
  grid-template-columns: minmax(0, 1fr) 300px;
  gap: var(--space-8) var(--space-16);
  align-items: start;
}

.head.is-bare { grid-template-columns: minmax(0, 1fr); }

.words {
  min-width: 0;
  display: grid;
  gap: var(--space-6);
  align-content: start;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-2);
}

.topics {
  margin: var(--space-2) 0 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: 6px var(--space-4);
  font: var(--text-label-mono);
  color: var(--ink-muted);
}

.plate {
  padding: 7px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--surface-page);
}

.plate-inner {
  aspect-ratio: 3 / 2;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--surface-raised);
}

.plate-inner img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: var(--surface-raised);
}

@media (max-width: 760px) {
  .head { grid-template-columns: minmax(0, 1fr); margin-top: var(--space-8); }
  .figure { order: -1; max-width: 360px; width: 100%; }
}
</style>
