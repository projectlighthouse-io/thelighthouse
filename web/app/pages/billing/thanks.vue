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
  <div class="mx-auto max-w-2xl px-4 py-24 sm:px-6 lg:px-8">
    <h1 class="font-editorial text-4xl font-bold text-ink">Thank you.</h1>

    <template v-if="bought">
      <p class="text-mono-body mt-4">
        You bought <strong class="font-semibold text-ink">{{ trackName }}</strong>.
        <template v-if="bought.paid">Your books are open.</template>
        <template v-else>
          Your payment is still clearing — the books open as soon as it lands.
        </template>
      </p>

      <ul v-if="bought.books.length" class="mt-8 space-y-3">
        <li v-for="book in bought.books" :key="book.slug" class="flex items-start gap-3">
          <span class="mt-0.5 font-mono text-sm text-quiet">-</span>
          <NuxtLink :to="`/books/${book.slug}`" class="text-ink underline underline-offset-4">
            {{ book.title }}
          </NuxtLink>
        </li>
      </ul>
    </template>

    <!-- No session in the url, or one that is not this reader's. Nothing is
         wrong with their purchase; there is simply nothing to report here. -->
    <p v-else-if="error || !session" class="text-mono-body mt-4">
      Your purchase is being confirmed. It will appear in
      <NuxtLink to="/settings/billing" class="underline">your billing settings</NuxtLink>
      shortly.
    </p>

    <p v-else class="text-mono-body mt-4">
      Confirming your purchase…
    </p>

    <p class="mt-10 text-sm text-quiet">
      Manage it any time in
      <NuxtLink to="/settings/billing" class="underline">settings</NuxtLink>.
    </p>
  </div>
</template>
