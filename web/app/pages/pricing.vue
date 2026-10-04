<script setup lang="ts">
import { plans } from '@/data/Catalogue'
import { faqs } from '@/data/Faqs'

const description
  = 'two tracks — foundation, or everything. subscribe yearly, or buy a track outright and keep it.'

useSeo({ title: 'Pricing — projectlighthouse', description })

// Compiled in, like the frame's prices, so the structured data and the visible
// amounts are one answer — and both survive prerendering with no api.
useJsonLd('offers', {
  '@type': 'Product',
  'name': 'projectlighthouse',
  'description': 'Books and hands-on projects on systems programming.',
  'brand': { '@type': 'Brand', 'name': SITE.name },
  'offers': plans
    .map(offer => ({
      '@type': 'Offer',
      'name': offer.plan,
      'price': String(offer.amount / 100),
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

    <MarketingPricingFrame eyebrow="the tracks" class="frame" />

    <MarketingFaqSection />
  </div>
</template>

<style scoped>
.frame { margin-top: var(--space-12); }
</style>
