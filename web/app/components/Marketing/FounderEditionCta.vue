<script setup lang="ts">
withDefaults(
  defineProps<{
    priceLabel?: string
    savingsLabel?: string
    bullets?: string[]
    pricingHref?: string
  }>(),
  {
    priceLabel: '$49/year',
    savingsLabel: '$119',
    bullets: () => [
      '<strong class="font-semibold">9 books, 210+ lessons</strong> — go, rust, c, os, networking, dsa',
      '3 build-your-own projects',
      '2 book-based projects',
      '9 cli challenges',
      'systems-first teaching — memory up, not framework down',
      'validate locally with luxctl, our open-source CLI',
      '<strong class="font-semibold">more on the way</strong>, regularly updating',
    ],
    pricingHref: '/pricing',
  },
)
</script>

<template>
  <div
    class="rounded-md border border-night-rule bg-night p-6 text-night-head shadow-2xl sm:p-10 lg:p-12"
  >
    <div class="grid gap-10 lg:grid-cols-[1.4fr_1fr] lg:items-start lg:gap-16">
      <div>
        <p class="mb-4 font-mono text-xs tracking-[0.2em] uppercase text-teal-bright">
          learn to build, <span class="italic text-night-dim">not just prompt</span>
        </p>

        <div class="mb-8">
          <div
            class="inline-flex items-center rounded-full border border-night-rule-soft bg-night p-1.5 shadow-sm"
          >
            <span class="px-5 py-1.5 font-mono text-sm text-night-head">founders edition</span>
            <span
              class="rounded-full bg-gold px-5 py-1.5 font-mono text-sm font-semibold text-night"
            >
              save <span class="font-bold">{{ savingsLabel }}</span>
            </span>
          </div>
        </div>

        <h2
          class="font-editorial text-night-head"
          style="
            font-weight: 600;
            font-size: clamp(2rem, 4.5vw, 3.25rem);
            line-height: 1.05;
            letter-spacing: -0.01em;
          "
        >
          the whole voyage<span class="font-normal text-night-text">, for the price of one
            textbook.</span>
        </h2>

        <p class="mt-6 max-w-xl text-base leading-relaxed text-night-text sm:text-lg">
          every voyage book, lab, and project — and every new voyage release.
          <strong class="font-semibold text-teal-bright">{{ priceLabel }}</strong> at a
          <strong
            class="font-semibold text-teal-bright underline decoration-teal-bright/40 decoration-2 underline-offset-4"
          >50% discount</strong>, billed once a year. cancel anytime.
        </p>
        <p class="mt-2 max-w-xl font-mono text-xs text-night-dim">
          horizon (advanced track) priced separately.
        </p>

        <div class="mt-8 flex flex-wrap items-center gap-4">
          <NuxtLink
            :to="pricingHref"
            class="inline-flex cursor-pointer items-center justify-center gap-2 rounded-md bg-gold px-6 py-3.5 font-mono text-sm font-semibold text-night shadow-[0_0_24px_rgba(251,191,36,0.35)] transition hover:bg-gold-bright"
          >
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path
                d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            Get Pro
          </NuxtLink>
          <NuxtLink
            :to="pricingHref"
            class="font-mono text-xs text-night-dim underline underline-offset-4 transition hover:text-night-head"
          >
            see offering
          </NuxtLink>
        </div>
      </div>

      <ul class="bullet-squiggle relative space-y-3 pl-7 lg:pl-16">
        <li
          v-for="(bullet, i) in bullets"
          :key="i"
          class="font-editorial text-base text-night-head sm:text-lg"
        >
          <!-- eslint-disable-next-line vue/no-v-html -- authored copy, not user input -->
          <span v-html="bullet" />
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.bullet-squiggle::before {
  content: '';
  position: absolute;
  top: 0.4rem;
  bottom: 0.4rem;
  left: 0;
  width: 8px;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='8' height='22' viewBox='0 0 8 22' fill='none' stroke='%23a8a8a0' stroke-width='1.2' stroke-linecap='round'><path d='M 4 0 C 8 5, 0 8, 4 11 C 8 14, 0 17, 4 22'/></svg>");
  background-repeat: repeat-y;
  background-position: left center;
}
</style>
