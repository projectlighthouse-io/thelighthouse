<script setup lang="ts">
/**
 * What the reader is paying for, and a door to Stripe for doing anything about
 * it.
 *
 * Only subscriptions appear here. A track bought outright has nothing to
 * manage — it does not renew, cannot be cancelled, and the books stay open —
 * so it is not a row on a settings page waiting for a button.
 */

definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Billing — projectlighthouse',
  description: 'Your subscription.',
  noindex: true,
})

const { membership, resolved, busy, reason, ending, track, load, manage }
  = useBilling()

onMounted(() => load())

/** `rust_yearly` reads as "Rust, yearly" rather than as an identifier. */
const period = computed(() => {
  const plan = membership.value?.plan ?? ''

  return plan.endsWith('_lifetime') ? 'bought outright' : 'renews yearly'
})

function on(date: string | null): string {
  if (!date) return ''

  return new Date(date).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}
</script>

<template>
  <SettingsShell>
    <h2 class="font-serif text-2xl text-ink">Billing</h2>
    <p class="mt-2 text-sm leading-relaxed text-quiet">
      What you are paying for. Books you bought outright are not listed here —
      they do not renew, and they stay yours.
    </p>

    <p v-if="!resolved" class="mt-8 text-sm text-quiet">Loading…</p>

    <div v-else-if="!membership" class="mt-8 rounded-lg border border-dashed border-rule p-8 text-center">
      <p class="text-sm text-quiet">You have no subscription.</p>
      <NuxtLink
        to="/pricing"
        class="mt-4 inline-block rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
      >
        See the tracks
      </NuxtLink>
    </div>

    <div v-else class="mt-8 rounded-lg border border-rule p-6">
      <div class="flex items-baseline justify-between">
        <span class="font-editorial text-xl text-ink capitalize">{{ track }}</span>
        <span class="text-sm text-quiet">{{ period }}</span>
      </div>

      <!-- Grace is a failed payment being retried. Access has already stopped,
           so saying "active" here would be a lie the reader can see through. -->
      <p v-if="membership.status === 'grace'" class="mt-4 text-sm leading-relaxed text-quiet">
        A payment did not go through, and your books are closed until it does.
        Update your card under manage billing.
      </p>
      <p v-else-if="ending" class="mt-4 text-sm leading-relaxed text-quiet">
        Ends on {{ on(membership.cancel_at) }}. You keep everything until then.
      </p>
      <p v-else-if="membership.period_ends_at" class="mt-4 text-sm leading-relaxed text-quiet">
        Renews on {{ on(membership.period_ends_at) }}.
      </p>

      <p v-if="reason" class="mt-4 text-sm text-ink">{{ reason }}</p>

      <button
        type="button"
        :disabled="busy"
        class="mt-6 rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover disabled:opacity-50"
        @click="manage()"
      >
        {{ busy ? 'Opening Stripe…' : 'Manage billing' }}
      </button>
      <p class="mt-4 text-sm text-quiet">
        Cancel, change plan, update your card or download invoices on Stripe.
      </p>
    </div>
  </SettingsShell>
</template>
