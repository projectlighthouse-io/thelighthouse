<script setup lang="ts">
import type { Book } from '@/types/Content'

const description
  = 'Interactive programming books on Go, Rust, data structures and algorithms, networking fundamentals, and operating systems. Learn software engineering fundamentals from first principles with hands-on examples.'

// From ohara, through the rust api. During SSR this calls the handler directly,
// so it costs no HTTP round trip.
const { data } = await useAsyncData('books', () => $fetch<Book[]>('/_api/books'))

const books = computed<Book[]>(() => data.value ?? [])

useSeo({
  title: 'Programming Books - Go, Rust, DSA, Networking, OS',
  description,
})

useJsonLd('books', () => ({
  '@type': 'CollectionPage',
  'name': 'Books',
  'mainEntity': {
    '@type': 'ItemList',
    'itemListElement': books.value.map((b, i) => ({
      '@type': 'ListItem',
      'position': i + 1,
      'url': `${SITE.url}/books/${b.slug}`,
      'name': b.title,
    })),
  },
}))
</script>

<template>
  <div>
    <section class="py-16">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mx-auto max-w-3xl text-center">
          <h1 class="mb-4 font-serif text-4xl tracking-tight text-ink sm:text-5xl">Books</h1>
          <p class="text-mono-body">Programming concepts in depth, one topic at a time.</p>
        </div>
      </div>
    </section>

    <section class="pb-16">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div
          v-if="books.length === 0"
          class="rounded-lg border border-rule bg-panel py-12 text-center"
        >
          <p class="text-sm text-quiet">No books available yet. Check back soon!</p>
        </div>

        <div v-else class="lg:solid-gray-bg grid gap-8 rounded-md p-0 md:grid-cols-2 lg:grid-cols-3 lg:p-6">
          <BookCard v-for="book in books" :key="book.slug" :book="book" />
        </div>
      </div>
    </section>
  </div>
</template>
