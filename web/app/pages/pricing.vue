<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import type { Book } from '@/types/Content'

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

// The same request, and the same cache key, as the frame below — so the
// structured data and the visible prices are one answer.
const { data: offers } = await useAsyncData('billing-plans', () =>
  $fetch<Offer[]>('/_api/billing/plans').catch(() => [] as Offer[]))

const { data: books } = await useAsyncData('books', () =>
  $fetch<Book[]>('/_api/books').catch(() => [] as Book[]))

useJsonLd('offers', {
  '@type': 'Product',
  'name': 'projectlighthouse',
  'description': 'Books and hands-on projects on systems programming.',
  'brand': { '@type': 'Brand', 'name': SITE.name },
  'offers': (offers.value ?? [])
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
  <div class="pricing-page">
    <section class="lh-text lh-gap">
      <div class="lh-head">
        <p class="lh-eyebrow">pricing</p>
        <h1 class="lh-h1">Pick a track, not a tier</h1>
        <p class="lh-sub">
          Subscribe yearly and keep up with everything shipped to it, or buy one
          outright and keep it for good. Every track includes Foundation.
        </p>
      </div>
    </section>

    <MarketingPricingFrame :books="books ?? []" eyebrow="the tracks" class="frame" />

    <MarketingFaqSection />
  </div>
</template>

<style scoped>
.frame { margin-top: var(--space-12); }
</style>
