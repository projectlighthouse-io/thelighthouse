<script setup lang="ts">
import type { PppOffer } from '@/types/Content'

/**
 * The purchasing-power banner.
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
  if (!offer.value) return ''

  try {
    return new Intl.DisplayNames(['en'], { type: 'region' }).of(offer.value.country) ?? offer.value.country
  }
  catch {
    return offer.value.country
  }
})

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
  <div v-if="offer && !dismissed" class="ppp lh-figure">
    <div class="bar" role="region" aria-label="regional pricing">
      <p>
        Prices are adjusted for <strong>{{ countryName }}</strong>. The lower price is applied
        automatically at checkout — see <NuxtLink to="/pricing" class="lh-inline">pricing</NuxtLink>.
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
