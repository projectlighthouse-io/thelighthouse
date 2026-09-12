<script setup lang="ts">
import type { CatalogueBook, CataloguePlan } from '@/data/Catalogue'
import { plans, shelf } from '@/data/Catalogue'
import { faqs } from '@/data/Faqs'
import type { Track } from '@/data/Tracks'
import { tracks } from '@/data/Tracks'

const description
  = 'two tracks — foundation, or everything. subscribed yearly, and everything shipped to the track while you are on it.'

useSeo({ title: 'Pricing — projectlighthouse', description })

/**
 * What everything costs, and what is on each track — compiled in, not fetched.
 *
 * **This page is prerendered.** `routeRules` in `nuxt.config.ts` says so, and
 * the web image is built from `web/` alone: no api, no database, no network.
 * Fetching the amounts during that build therefore could not work, and did
 * not — the connection was refused, the `.catch` fallback rendered, and the
 * page shipped with an em dash where each price belongs. Reading a file that is
 * already on disk cannot fail that way.
 *
 * **Still one home for the number.** `lighthouse-prices catalogue` writes
 * `Catalogue.ts` from the same `pricing.yaml` it reconciles against stripe, so
 * an amount here is the amount charged. What changed is when it is read, not
 * where it comes from.
 *
 * The cost of compiling it in is that a price or a track changing needs a
 * rebuild. That is the right trade for this page: a price is a deliberate
 * decision that ships, unlike a lesson.
 */
function offerFor(
  track: string,
  recurring: boolean,
): CataloguePlan | undefined {
  return plans.find(
    offer => offer.track === track && offer.recurring === recurring,
  )
}

/** `4900` reads as `$49`; a price nobody knows reads as nothing at all. */
function priced(offer: CataloguePlan | undefined): string | null {
  if (!offer?.amount) return null

  const whole = offer.amount / 100

  return `$${Number.isInteger(whole) ? whole : whole.toFixed(2)}`
}

/**
 * Which books are on a track, in that track's reading order.
 *
 * Derived, not listed. `Tracks.ts` used to carry the slugs itself and its own
 * comment conceded the problem — "if the two ever disagree, this file is the
 * one that is wrong", because what a track actually contains is the `tracks:`
 * map in each `book.yaml`. The generated shelf is read straight out of those
 * same maps, so this and the entitlement rust grants cannot disagree.
 *
 * `all` is deliberately empty: the card for it says "every book on every track,
 * plus the ones on none", which is a claim about the future as much as the
 * present and does not want a list.
 */
function booksOn(key: Track['key']): CatalogueBook[] {
  if (key === 'all') return []

  return shelf
    .filter(book => book.tracks[key] !== undefined)
    .sort((a, b) => (a.tracks[key] ?? 0) - (b.tracks[key] ?? 0))
}

const { checkout, busy, reason } = useBilling()
const { isSignedIn } = useReader()

/**
 * Buying needs a session, because the api ties the purchase to an account.
 * An anonymous reader signs in first and comes back here.
 *
 * `isSignedIn` falls back to the reader hint cookie until the session endpoint
 * answers, so this does not bounce somebody who is in fact signed in and has
 * simply not been confirmed yet.
 */
async function buy(plan: string | undefined): Promise<void> {
  if (!plan) return

  if (!isSignedIn.value) {
    await navigateTo(`/login?redirect=${encodeURIComponent('/pricing')}`)

    return
  }

  await checkout(plan)
}

useJsonLd('offers', {
  '@type': 'Product',
  'name': 'projectlighthouse',
  'description': 'Books and hands-on projects on systems programming.',
  'brand': { '@type': 'Brand', 'name': SITE.name },
  'offers': plans
    .filter(offer => offer.amount !== null)
    .map(offer => ({
      '@type': 'Offer',
      'name': offer.plan,
      'price': String((offer.amount ?? 0) / 100),
      'priceCurrency': offer.currency.toUpperCase(),
      'url': `${SITE.url}/pricing`,
      'availability': 'https://schema.org/InStock',
    })),
})

useJsonLd('faq', {
  '@type': 'FAQPage',
  'mainEntity': faqs.map(f => ({
    '@type': 'Question',
    'name': f.question,
    'acceptedAnswer': { '@type': 'Answer', 'text': f.answer },
  })),
})
</script>

<template>
  <div>
    <section class="pt-10 pb-6 sm:pt-14 sm:pb-8">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mx-auto max-w-4xl text-center">
          <h1
            class="font-editorial text-ink text-hero-lg tracking-editorial font-medium leading-[1.05]"
          >
            pick a track<span class="font-normal">,
              <br class="hidden sm:block">not a subscription tier.</span>
          </h1>

          <p class="mx-auto mt-8 max-w-2xl text-base leading-relaxed text-quiet sm:text-lg">
            two tracks. subscribe yearly and keep up with everything we ship to it for as
            long as you are on it.
          </p>
          <p class="mx-auto mt-3 max-w-2xl text-center font-mono text-xs text-faint">
            every track includes foundation.
          </p>
        </div>
      </div>
    </section>

    <section id="pricing" class="pb-16">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <p v-if="reason" class="mb-6 text-center text-sm text-ink">{{ reason }}</p>

        <div class="mx-auto grid max-w-4xl gap-8 sm:grid-cols-2">
          <div
            v-for="track in tracks"
            :key="track.key"
            class="relative flex flex-col rounded-xl bg-panel p-8"
            :class="track.featured ? 'border-pencil' : 'border-pencil-light'"
          >
            <div class="mb-6">
              <h2 class="mb-4 text-xl font-semibold text-ink">{{ track.name }}</h2>
              <p class="font-mono text-sm text-quiet">{{ track.blurb }}</p>
            </div>

            <div class="mb-6">
              <div class="flex items-baseline gap-1">
                <span class="font-editorial text-5xl font-bold text-ink">
                  {{ priced(offerFor(track.key, true)) ?? '—' }}
                </span>
                <span class="text-quiet">/ year</span>
              </div>
            </div>

            <ul v-if="booksOn(track.key).length" class="space-y-3 font-mono">
              <li v-for="book in booksOn(track.key)" :key="book.slug" class="flex items-start gap-3">
                <span class="mt-0.5 text-sm text-quiet">-</span>
                <span class="text-sm text-quiet">{{ book.title }}</span>
              </li>
            </ul>
            <p v-else class="font-mono text-sm text-quiet">
              every book on every track, plus the ones on none.
            </p>

            <div class="mt-auto space-y-3 pt-12">
              <button
                type="button"
                :disabled="busy || !offerFor(track.key, true)"
                class="block w-full rounded-md bg-ink px-5 py-3 text-center text-base font-medium text-on-ink transition hover:bg-ink-hover disabled:opacity-50"
                @click="buy(offerFor(track.key, true)?.plan)"
              >
                {{ busy ? 'One moment…' : 'Subscribe' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="pb-20">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <MarketingFounderEditionCta />
      </div>
    </section>

    <MarketingFaqSection />
  </div>
</template>
