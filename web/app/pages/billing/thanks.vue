<script setup lang="ts">
/**
 * Where the provider sends a reader who has just paid.
 *
 * **The payment is not confirmed here.** It is confirmed by a webhook landing
 * on rust, which may arrive before this page renders or a few seconds after —
 * the browser coming back is not the event that moves money. So this asks, and
 * keeps asking for a short while, rather than asserting.
 *
 * Nothing is written from this page. A page that wrote the subscription itself
 * would be a second writer racing the webhook, and the two would disagree the
 * first time somebody closed the tab.
 */

definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Thank you — projectlighthouse',
  description: 'Your purchase is being confirmed.',
  noindex: true,
})

const { membership, subscribed, load } = useBilling()

/** How long to wait for the webhook before saying so plainly. */
const ATTEMPTS = 10
const EVERY = 1500

const settled = ref(false)
const timedOut = ref(false)

onMounted(async () => {
  for (let attempt = 0; attempt < ATTEMPTS; attempt += 1) {
    await load(true)

    // A membership *or* an entitlement can be the outcome: a track bought
    // outright leaves no membership at all, only books that have opened. The
    // reliable signal for both is simply that the api answered.
    if (membership.value !== null) {
      settled.value = true
      return
    }

    await new Promise(resolve => setTimeout(resolve, EVERY))
  }

  timedOut.value = true
})
</script>

<template>
  <div class="mx-auto max-w-2xl px-4 py-24 sm:px-6 lg:px-8">
    <h1 class="font-editorial text-4xl font-bold text-ink">Thank you.</h1>

    <p v-if="settled" class="text-mono-body mt-4">
      Your payment went through and your books are open.
      <NuxtLink to="/books" class="underline">Start reading</NuxtLink>.
    </p>

    <p v-else-if="timedOut" class="text-mono-body mt-4">
      Your payment went through. It is taking a moment to show up here — this
      page will be right after a refresh, and nothing is lost either way. If it
      still looks wrong in a few minutes,
      <NuxtLink to="/support" class="underline">tell us</NuxtLink>.
    </p>

    <p v-else class="text-mono-body mt-4">
      Confirming your payment…
    </p>

    <p v-if="subscribed" class="mt-8 text-sm text-quiet">
      Manage it any time in
      <NuxtLink to="/settings/billing" class="underline">settings</NuxtLink>.
    </p>
  </div>
</template>
