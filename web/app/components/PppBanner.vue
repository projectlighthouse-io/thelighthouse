<script setup lang="ts">
import type { PppOffer } from '@/types/Content'
import { tracks } from '@/data/Tracks'

/**
 * The discount banner: the code for each plan and what it takes off, typed at
 * checkout. Shown whenever a plan is discounted for this reader — the
 * everyone-else price included — unless DISCOUNT_BANNER=false switches it off.
 *
 * Fetched in the browser, never during SSR: the offer depends on where the
 * reader is, and the pages around it are prerendered or edge-cached for
 * everyone. Renders nothing until there is an offer, and nothing again once
 * dismissed — for the rest of the session.
 */
const STORAGE_KEY = 'lh:ppp-dismissed'

const offer = ref<PppOffer | null>(null)
const dismissed = ref(true)

const countryName = computed<string>(() => {
  const country = offer.value?.country
  if (!country) return ''

  try {
    return new Intl.DisplayNames(['en'], { type: 'region' }).of(country) ?? country
  }
  catch {
    return country
  }
})

/**
 * The one offer the banner advertises: the plan "Get Pro" sells. Every plan's
 * code is still on its own pricing panel; the banner is one line, not a list.
 */
const lead = computed(() => offer.value?.offers.find(o => o.plan === PRO_PLAN) ?? null)

/** `foundation_yearly` reads as Foundations — the name its panel carries. */
function planName(plan: string): string {
  const track = plan.slice(0, plan.lastIndexOf('_'))

  return tracks.find(t => t.key === track)?.name ?? plan
}


onMounted(async () => {
  try {
    dismissed.value = sessionStorage.getItem(STORAGE_KEY) === '1'
  }
  catch {
    dismissed.value = false
  }

  if (dismissed.value) return

  offer.value = await $fetch<PppOffer | null>('/_api/billing/offer').catch(() => null)
})

function dismiss(): void {
  dismissed.value = true

  try {
    sessionStorage.setItem(STORAGE_KEY, '1')
  }
  catch {
    // storage refused — the banner still goes for this page view
  }
}
</script>

<template>
  <div v-if="offer && lead && !dismissed" class="ppp lh-figure">
    <div class="bar" role="region" aria-label="regional pricing">
      <!-- Two offers, two messages: a country with prices of its own hears
           why, everyone else hears it as the launch price it is. -->
      <p v-if="!lead.rest">
        <template v-if="countryName">You're in <strong>{{ countryName }}</strong>, so </template>
        {{ planName(lead.plan) }} is priced for where you live: use
        <code>{{ lead.code }}</code> at checkout for <strong>{{ offLabel(lead) }}</strong>.
      </p>
      <p v-else>
        Launch offer: use <code>{{ lead.code }}</code> at checkout for
        <strong>{{ offLabel(lead) }}</strong> {{ planName(lead.plan) }}.
      </p>
      <button type="button" class="dismiss" aria-label="dismiss" @click="dismiss">
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.ppp { margin-top: var(--space-4); }

.bar {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-3) var(--space-3) var(--space-3) var(--space-5);
  border-radius: var(--radius-md);
  /* off-system, named in the handoff */
  background: #fbe3e3;
  color: var(--ink);
  font: var(--text-body-sm);
}

p { margin: 0; min-width: 0; text-wrap: pretty; }

strong { font-weight: var(--weight-medium); }

code {
  font: var(--text-label-mono);
  padding: 3px 10px;
  border-radius: var(--radius-full);
  background: var(--surface-raised);
  color: var(--ink);
  word-break: break-all;
}

.dismiss {
  flex: none;
  margin-left: auto;
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: var(--radius-full);
  background: transparent;
  color: var(--ink-muted);
  font: var(--text-body-sm);
  cursor: pointer;
  transition: var(--transition-control);
}

.dismiss:hover { background: var(--surface-raised); color: var(--ink); }
</style>
