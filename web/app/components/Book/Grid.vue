<script setup lang="ts">
import type { Book } from '@/types/Content'

/**
 * The shelf as a grid — three across, two at 900px, one at 560px.
 *
 * `fade` is the home page's version: the last visible row is clipped under a
 * gradient with "See all books" sitting on it. It only applies once there is
 * more than a full shelf to hint at; a short shelf shows in full with the
 * button underneath.
 */
const props = defineProps<{
  books: Book[]
  fade?: boolean
}>()

const clipped = computed<boolean>(() => Boolean(props.fade) && props.books.length > 9)
</script>

<template>
  <div class="wrap" :class="{ 'is-clipped': clipped }">
    <div class="clip">
      <div class="grid">
        <BookCard v-for="book in books" :key="book.slug" :book="book" />
      </div>
    </div>

    <template v-if="fade">
      <div v-if="clipped" class="fade" aria-hidden="true" />
      <div class="more">
        <UiButton variant="inverse" size="lg" to="/books">See all books →</UiButton>
      </div>
    </template>
  </div>
</template>

<style scoped>
.wrap { position: relative; }

.clip { overflow: hidden; }

.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-5);
  align-items: stretch;
}

.more {
  display: flex;
  justify-content: center;
  margin-top: 40px;
}

/* the clipped shelf: 9 full cards, then a row cut off under the gradient */
.is-clipped .grid > :nth-child(n + 10) { max-height: 160px; align-self: start; }
.is-clipped .grid > :nth-child(n + 13) { display: none; }

.fade {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 240px;
  pointer-events: none;
  background: linear-gradient(180deg, rgba(252, 252, 252, 0), var(--surface-page) 70%);
}

.is-clipped .more {
  position: absolute;
  left: 0;
  right: 0;
  bottom: var(--space-8);
  margin: 0;
}

@media (max-width: 900px) {
  .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }

  .is-clipped .grid > :nth-child(n + 7) { max-height: 160px; align-self: start; }
  .is-clipped .grid > :nth-child(n + 9) { display: none; }
}

@media (max-width: 560px) {
  .grid { grid-template-columns: minmax(0, 1fr); }

  .is-clipped .grid > :nth-child(n + 4) { max-height: 160px; align-self: start; }
  .is-clipped .grid > :nth-child(n + 5) { display: none; }
}
</style>
