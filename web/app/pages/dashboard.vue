<script setup lang="ts">
import type { Book } from '@/types/Content'

definePageMeta({ middleware: 'auth' })

useSeo({
  title: 'Dashboard — projectlighthouse',
  description: 'Your books and reading progress on projectlighthouse.',
  noindex: true,
})

/**
 * What this reader may read, and what bought it.
 *
 * Answers for both shapes a purchase takes — a track subscribed to leaves a
 * membership and no entitlements, a track bought outright leaves entitlements
 * and no membership — so the page does not have to know which happened.
 *
 * Client side: it carries the session cookie, and this page is `ssr: false`
 * and `no-store` for exactly that reason.
 */
const { data: access } = await useAsyncData('dashboard-access', () =>
  $fetch<{ track: string | null, books: { slug: string }[] }>(
    '/api/billing/access',
  ).catch(() => null), { server: false })

/** The shelf, for the cover and blurb the api's answer does not carry. */
const { data: shelf } = await useAsyncData('dashboard-books', () =>
  $fetch<Book[]>('/_api/books').catch(() => [] as Book[]))

/**
 * The reader's own books, in the shelf's order.
 *
 * Matched by slug rather than sent whole from the billing endpoint: which
 * books exist and what they look like is the catalogue's answer, and copying
 * covers and blurbs into a billing response would be a second place for them
 * to go stale.
 */
const owned = computed<Book[]>(() => {
  const mine = new Set((access.value?.books ?? []).map(book => book.slug))

  return (shelf.value ?? []).filter(book => mine.has(book.slug))
})
</script>

<template>
  <div class="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
    <!-- Only for a reader who owns something. A reader who has bought nothing
         is not told they have an empty shelf; there is simply nothing here to
         pick up, and the pricing page is where the invitation belongs. -->
    <template v-if="owned.length">
      <h1 class="mb-2 font-serif text-3xl tracking-tight text-ink sm:text-4xl">My books</h1>
      <p class="text-mono-body mb-10">
        <template v-if="access?.track">
          everything on the {{ access.track }} track. pick up where you left off.
        </template>
        <template v-else>pick up where you left off.</template>
      </p>

      <div class="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
        <BookCard v-for="book in owned" :key="book.slug" :book="book" />
      </div>
    </template>
  </div>
</template>
