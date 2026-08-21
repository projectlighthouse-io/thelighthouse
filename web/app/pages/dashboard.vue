<script setup lang="ts">
import { books } from '@/data/Books'

definePageMeta({ middleware: 'auth' })

// signed-in shell only — real progress arrives with the api in phase 6
const inProgress = computed(() => books.slice(0, 3))

useSeo({
  title: 'Dashboard — projectlighthouse',
  description: 'Your books and reading progress on projectlighthouse.',
  noindex: true,
})
</script>

<template>
  <div class="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
    <h1 class="mb-2 font-serif text-3xl tracking-tight text-ink sm:text-4xl">My books</h1>
    <p class="text-mono-body mb-10">pick up where you left off.</p>

    <div v-if="inProgress.length === 0" class="border-pencil-light rounded-md bg-panel py-16 text-center">
      <p class="text-sm text-quiet">No books yet.</p>
      <NuxtLink
        to="/books"
        class="mt-4 inline-block rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
      >
        Browse the shelf
      </NuxtLink>
    </div>

    <div v-else class="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
      <BookCard v-for="book in inProgress" :key="book.slug" :book="book" />
    </div>
  </div>
</template>
