<script setup lang="ts">
/**
 * What the reader is paying for, and the three things they can do about it.
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

const { membership, resolved, busy, reason, subscribed, ending, track, load, cancel, resume }
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
  <AccountShell
    title="Billing"
    sub="What you are paying for. Books you bought outright are not listed here — they do not renew, and they stay yours."
  >
    <p v-if="!resolved" class="lh-sub">Loading…</p>

    <div v-else-if="!membership" class="lh-card empty">
      <p class="lh-eyebrow">no subscription</p>
      <p class="lh-sub">You are not subscribed to a track.</p>
      <UiButton variant="inverse" size="lg" to="/pricing">See the tracks</UiButton>
    </div>

    <div v-else class="lh-card plan">
      <dl class="lh-facts">
        <dt>track</dt>
        <dd>{{ track }}</dd>
        <dt>billed</dt>
        <dd>{{ period }}</dd>
        <dt>status</dt>
        <dd>
          <!-- Grace is a failed payment being retried. Access has already
               stopped, so saying "active" here would be a lie the reader can
               see through. -->
          <template v-if="membership.status === 'grace'">
            A payment did not go through, and your books are closed until it does.
            Update your card with your bank or try again.
          </template>
          <template v-else-if="ending">
            Ends on {{ on(membership.cancel_at) }}. You keep everything until then.
          </template>
          <template v-else-if="membership.period_ends_at">
            Renews on {{ on(membership.period_ends_at) }}.
          </template>
          <template v-else>{{ membership.status }}</template>
        </dd>
      </dl>

      <p v-if="reason" class="lh-error" role="alert">{{ reason }}</p>

      <hr class="lh-dashed">

      <div class="actions">
        <UiButton v-if="ending" variant="inverse" size="md" :disabled="busy" @click="resume()">
          {{ busy ? 'Working…' : 'Keep it' }}
        </UiButton>
        <UiButton v-else-if="subscribed" variant="ghost" size="md" :disabled="busy" @click="cancel()">
          {{ busy ? 'Working…' : 'Cancel' }}
        </UiButton>

        <p v-if="subscribed && !ending" class="lh-hint">
          Cancelling stops the renewal. You keep your books until
          {{ on(membership.period_ends_at) }}.
        </p>
      </div>
    </div>
  </AccountShell>
</template>

<style scoped>
.empty,
.plan {
  display: grid;
  gap: var(--space-5);
  justify-items: start;
}

.plan .lh-facts,
.plan hr { width: 100%; }

.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-4);
}
</style>
