<script setup lang="ts">
import { plans } from '@/data/Catalogue'

/**
 * `/checkout?plan=…` — where every "Get Pro" button goes, signed in or not.
 *
 * It exists so the plan a reader chose survives signing in. A reader without a
 * session is sent to sign in with this page, plan and all, as the place to come
 * back to; back here with a session, it hands them to stripe for that plan — or,
 * when they already pay, to their billing page, which is where a plan changes.
 * Nothing to read, so it is client-only and kept out of the index.
 */
definePageMeta({ layout: 'default' })

useSeo({
  title: 'Checkout',
  description: 'Taking you to a secure checkout for the plan you picked on ProjectLighthouse.',
  noindex: true,
})

const route = useRoute()
const { resolve, isSignedIn } = useReader()
const { checkout, reason, refusal } = useBilling()

/** The plan asked for, if it is one that is on sale. */
const plan = computed<string | null>(() => {
  const asked = route.query.plan

  return typeof asked === 'string' && plans.some(p => p.plan === asked) ? asked : null
})

onMounted(async () => {
  if (!plan.value) return

  await resolve()

  if (!isSignedIn.value) {
    await navigateTo({ path: '/login', query: { redirect: route.fullPath } }, { replace: true })

    return
  }

  // Navigates away to stripe on success; anything after this is a refusal.
  await checkout(plan.value)

  if (refusal.value === 'already_subscribed') {
    await navigateTo('/settings/billing', { replace: true })
  }
})
</script>

<template>
  <section class="checkout lh-text lh-gap">
    <template v-if="!plan">
      <h1 class="lh-h2">Pick a plan first</h1>
      <p class="lh-sub">That link does not name a plan on sale.</p>
      <UiButton variant="inverse" size="md" to="/pricing">See pricing</UiButton>
    </template>

    <template v-else-if="reason">
      <h1 class="lh-h2">Checkout did not open</h1>
      <p class="lh-error" role="alert">{{ reason }}</p>
      <UiButton variant="inverse" size="md" to="/pricing">Back to pricing</UiButton>
    </template>

    <template v-else>
      <h1 class="lh-h2">Taking you to checkout…</h1>
      <p class="lh-sub">One moment.</p>
    </template>
  </section>
</template>

<style scoped>
.checkout {
  display: grid;
  gap: var(--space-4);
  justify-items: start;
}

.checkout > * { margin: 0; }
</style>
