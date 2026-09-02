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
  <div class="mx-auto max-w-[1120px] bg-panel px-[52px] pt-[52px] pb-24 max-[820px]:px-10 max-[820px]:pt-10">
    <!-- `bg-panel` so the body's dotted paper stops at the edge of the slab,
         the same way the book page's does. -->
    <header class="text-center">
      <h1 class="masthead-title">Books</h1>
      <p class="masthead-dek mx-auto max-w-[30em]">
        Programming concepts in depth, one topic at a time.
      </p>
    </header>

    <!-- The shelf owns the tabs, the filtering and the covers. This page owns
         the slab, the title and what a crawler reads. -->
    <BookShelf :books="books" />
  </div>
</template>
