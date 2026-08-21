<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import { plans } from '@/data/Plans'

const description
  = 'every voyage book and project — go, rust, dsa, os, networking, c — and every new voyage release we ship. cancel anytime.'

useSeo({
  title: 'Pricing — projectlighthouse',
  description,
})

useJsonLd('offers', {
  '@type': 'Product',
  'name': 'projectlighthouse',
  'description': 'Books and hands-on projects on systems programming.',
  'brand': { '@type': 'Brand', 'name': SITE.name },
  'offers': plans
    .filter(plan => plan.price !== '0')
    .map(plan => ({
      '@type': 'Offer',
      'name': plan.name,
      'price': plan.price,
      'priceCurrency': 'USD',
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
            class="font-editorial text-ink font-medium text-hero-lg leading-[1.05] tracking-editorial"
          >
            the whole voyage<span class="font-normal">,
              <br class="hidden sm:block">for the price of one textbook.</span>
          </h1>

          <p class="mx-auto mt-8 max-w-2xl text-base leading-relaxed text-quiet sm:text-lg">
            every voyage book and project — go, rust, dsa, os, networking, c — and every new voyage
            release we ship. cancel anytime.
          </p>
          <p class="mx-auto mt-3 max-w-2xl text-center font-mono text-xs text-faint">
            horizon (advanced track) priced separately.
          </p>
        </div>
      </div>
    </section>

    <section id="pricing" class="pb-16">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="grid gap-8 lg:grid-cols-3">
          <div
            v-for="plan in plans"
            :key="plan.key"
            class="relative flex flex-col rounded-xl bg-panel p-8"
            :class="plan.featured ? 'border-pencil' : 'border-pencil-light'"
          >
            <div class="mb-6">
              <h2 class="mb-4 text-xl font-semibold text-ink">{{ plan.name }}</h2>
              <p class="font-mono text-sm text-quiet">{{ plan.blurb }}</p>
            </div>

            <div class="mb-6">
              <div class="flex items-baseline gap-1">
                <span class="font-editorial text-4xl font-bold text-quiet">$</span>
                <span class="font-editorial text-6xl font-bold text-ink">{{ plan.price }}</span>
                <span class="text-quiet">{{ plan.period }}</span>
              </div>
              <div class="mt-1">
                <span class="text-xs text-quiet">{{ plan.note }}</span>
              </div>
            </div>

            <ul class="space-y-3 font-mono">
              <li v-for="feature in plan.features" :key="feature" class="flex items-start gap-3">
                <span class="mt-0.5 text-sm" :class="plan.featured ? 'text-ink' : 'text-quiet'">-</span>
                <span class="text-sm" :class="plan.featured ? 'text-ink' : 'text-quiet'">
                  {{ feature }}
                </span>
              </li>
            </ul>

            <div class="mt-auto pt-12">
              <NuxtLink
                v-if="plan.featured"
                to="/login"
                class="block rounded-md bg-ink px-5 py-3 text-center text-base font-medium text-on-ink transition hover:bg-ink-hover"
              >
                {{ plan.cta }}
              </NuxtLink>
              <NuxtLink
                v-else-if="plan.key === 'lifetime'"
                to="/login"
                class="block rounded-md border border-stroke bg-panel px-5 py-3 text-center text-base font-medium text-ink transition hover:bg-paper-warm"
              >
                {{ plan.cta }}
              </NuxtLink>
              <div v-else class="text-center text-sm font-medium text-quiet">{{ plan.cta }}</div>
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
