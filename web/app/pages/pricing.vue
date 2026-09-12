<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import type { Book } from '@/types/Content'
import type { Track } from '@/data/Tracks'
import { tracks } from '@/data/Tracks'

/** One purchasable plan, as `/api/billing/plans` reports it. */
interface Offer {
  plan: string
  track: string
  recurring: boolean
  amount: number | null
  currency: string | null
}

const description
  = 'four tracks — foundation, go, rust, or everything. subscribe yearly, or buy a track outright and keep it.'

useSeo({ title: 'Pricing — projectlighthouse', description })

/**
 * What everything costs, from rust.
 *
 * Fetched rather than written here because the amount has exactly one home:
 * the declaration `lighthouse-prices` reconciles against stripe. A number
 * typed into this file is a number that drifts from what is actually charged,
 * and the drift is invisible until somebody compares a receipt with this page.
 *
 * Through nitro's `/_api`, like the other public reads, so it resolves during
 * SSR and the amounts are in the prerendered html rather than appearing a
 * moment after hydration — which is what a price on a pricing page has to do.
 *
 * Allowed to fail: a build with no api behind it renders the tracks without
 * amounts rather than failing the build.
 */
/** What `/_api/billing/plans` answers: the offers, the country Cloudflare
 *  reported, and the coupon that country gets — null until coupons exist. */
/** The discount a country is offered, if any. Advertised, not applied — the
 *  reader types `code` at stripe. */
interface Coupon {
  code: string
  percent: number
}

interface Catalogue {
  country: string | null
  plans: Offer[]
  coupon: Coupon | null
}

const { data: catalogue } = await useAsyncData('billing-plans', () =>
  $fetch<Catalogue>('/_api/billing/plans')
    .catch(() => ({ country: null, plans: [], coupon: null } as Catalogue)),
)

/** The offers alone. The country and the discount slots ride on the same
 *  answer, and nothing on this page reads them yet. */
const offers = computed<Offer[]>(() => catalogue.value?.plans ?? [])

function offerFor(track: string, recurring: boolean): Offer | undefined {
  return offers.value.find(
    offer => offer.track === track && offer.recurring === recurring,
  )
}

/** `4900` reads as `$49`; a price nobody knows reads as nothing at all. */
function priced(offer: Offer | undefined): string | null {
  if (!offer?.amount) return null

  const whole = offer.amount / 100

  return `$${Number.isInteger(whole) ? whole : whole.toFixed(2)}`
}

/**
 * The shelf, for turning a track's book slugs into titles.
 *
 * From the api rather than a copy in `app/data`: that copy drifted — it was two
 * books and every track behind before the home page stopped reading it — and a
 * price page naming a book that no longer exists is worse than one naming none.
 *
 * Server side, unlike the private pages: this is public, the same for everyone,
 * and part of the ranking surface.
 */
const { data: books } = await useAsyncData<Book[]>(
  'pricing-books',
  () => $fetch<Book[]>('/_api/books'),
  { default: () => [] },
)

/**
 * Which books are on a track, in that track's reading order.
 *
 * Derived, not listed. `Tracks.ts` used to carry the slugs itself and its own
 * comment conceded the problem — "if the two ever disagree, this file is the
 * one that is wrong", because what a track actually contains is the `tracks:`
 * map in each `book.yaml`, which is what rust grants entitlements from. Reading
 * the same source the grant reads means they cannot disagree.
 *
 * `all` is deliberately empty: the card for it says "every book on every track,
 * plus the ones on none", which is a claim about the future as much as the
 * present and does not want a list.
 */
function booksOn(key: Track['key']): Book[] {
  if (key === 'all') return []

  return books.value
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
  'offers': offers.value
    .filter(offer => offer.amount !== null)
    .map(offer => ({
      '@type': 'Offer',
      'name': offer.plan,
      'price': String((offer.amount ?? 0) / 100),
      'priceCurrency': (offer.currency ?? 'usd').toUpperCase(),
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
            four tracks. subscribe yearly and keep up with everything we ship to it, or buy
            one outright and keep it for good.
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

        <div class="grid gap-8 lg:grid-cols-4">
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
              <div class="mt-1">
                <span class="text-xs text-quiet">
                  <template v-if="priced(offerFor(track.key, false))">
                    or {{ priced(offerFor(track.key, false)) }} once, yours for good
                  </template>
                  <template v-else>&nbsp;</template>
                </span>
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
              <button
                type="button"
                :disabled="busy || !offerFor(track.key, false)"
                class="border-stroke block w-full rounded-md border bg-panel px-5 py-3 text-center text-base font-medium text-ink transition hover:bg-paper-warm disabled:opacity-50"
                @click="buy(offerFor(track.key, false)?.plan)"
              >
                Buy outright
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
