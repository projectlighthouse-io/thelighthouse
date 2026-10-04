<script setup lang="ts">
/**
 * Where the provider sends a reader who has just paid.
 *
 * The url carries a checkout session id — `{CHECKOUT_SESSION_ID}`, which the
 * provider substitutes on the way back — and nothing else. What was bought is
 * read from the provider by rust, which refuses a session that names anybody
 * but the reader asking. So this page shows the truth immediately without
 * trusting a query string.
 *
 * **Nothing here fulfils anything.** A reader can pay and close the tab before
 * this ever loads, which is why the webhook writes the membership and this
 * only reports. The two are deliberately independent: if the webhook is late,
 * the page still says what was bought, and access follows a moment later.
 */

definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Thank you — projectlighthouse',
  description: 'Your purchase is confirmed.',
  noindex: true,
})

interface Bought {
  plan: string | null
  track: string | null
  paid: boolean
  books: { slug: string, title: string }[]
}

const route = useRoute()
const session = computed(() => String(route.query.session ?? ''))

const { data: bought, error } = await useAsyncData('bought', () =>
  session.value
    ? $fetch<Bought>(`/api/billing/stripe/bought/${session.value}`)
    : Promise.resolve(null), { server: false, watch: [session] })

/** `go` reads as `Go`, `all` as `Everything`. */
const trackName = computed(() => {
  const track = bought.value?.track
  if (!track) return ''

  return track === 'all' ? 'Everything' : track[0]!.toUpperCase() + track.slice(1)
})
</script>

<template>
  <div class="thanks lh-text lh-gap">
    <div class="lh-card card">
      <p class="lh-eyebrow">checkout</p>
      <h1 class="lh-h2">Thank you.</h1>

      <template v-if="bought">
        <p class="lh-sub">
          You bought <strong>{{ trackName }}</strong>.
          <template v-if="bought.paid">Your books are open.</template>
          <template v-else>
            Your payment is still clearing — the books open as soon as it lands.
          </template>
        </p>

        <ol v-if="bought.books.length" class="books">
          <li v-for="(book, i) in bought.books" :key="book.slug">
            <NuxtLink :to="`/books/${book.slug}`" class="lh-row row">
              <span class="lh-mono lh-faint lh-num">{{ String(i + 1).padStart(2, '0') }}</span>
              <span>{{ book.title }}</span>
            </NuxtLink>
          </li>
        </ol>

        <UiButton variant="inverse" size="lg" to="/dashboard">Go to my shelf →</UiButton>
      </template>

      <!-- No session in the url, or one that is not this reader's. Nothing is
           wrong with their purchase; there is simply nothing to report here. -->
      <p v-else-if="error || !session" class="lh-sub">
        Your purchase is being confirmed. It will appear in
        <NuxtLink to="/settings/billing" class="lh-inline">billing</NuxtLink> shortly.
      </p>

      <p v-else class="lh-sub">Confirming your purchase…</p>

      <p class="lh-hint">
        Manage it any time in <NuxtLink to="/settings/billing" class="lh-inline">billing</NuxtLink>.
      </p>
    </div>
  </div>
</template>

<style scoped>
.card {
  display: grid;
  gap: var(--space-5);
  justify-items: start;
}

strong { font-weight: var(--weight-medium); color: var(--ink); }

.books {
  width: 100%;
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 2px;
}

.row {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  padding: var(--space-2) var(--space-3);
  font: var(--text-body-sm);
}
</style>
