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
  <AccountShell
    title="My shelf"
    :sub="access?.track
      ? `Everything on the ${access.track} track. Pick up where you left off.`
      : owned.length ? 'Pick up where you left off.' : 'The books you buy show up here.'"
  >
    <BookGrid v-if="owned.length" :books="owned" />

    <!-- A reader who has bought nothing. Not an error and not an empty grid —
         every book has free lessons, so the shelf is a real place to send
         them, and the tracks are what they would be buying. -->
    <EmptyState
      v-else
      class="lh-card"
      eyebrow="nothing here yet"
      heading="What will you read first?"
      detail="You have not bought a track yet. Every book's first lessons are free to read."
      :action="{ label: 'See the tracks', to: '/pricing' }"
    >
      <template #secondary>
        <UiButton variant="ghost" size="lg" to="/books">Browse the shelf</UiButton>
      </template>
    </EmptyState>
  </AccountShell>
</template>
