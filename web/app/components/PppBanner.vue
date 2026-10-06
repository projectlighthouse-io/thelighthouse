<script setup lang="ts">
import type { PppOffer } from '@/types/Content'
import type { Promo } from '@/utils/Promo'
import { PROMO_COPY } from '@/data/Promo'
import { promoFrom } from '@/utils/Promo'

/**
 * The regional promo strip above the navbar: what comes off, for which
 * country, with which code.
 *
 * Fetched in the browser, never during SSR: the offer depends on where the
 * reader is, and the pages around it are prerendered or edge-cached for
 * everyone. So nothing renders — not even an empty strip — until there is a
 * promo with all three of country, amount and code.
 *
 * Dismissing it is remembered per promo, in localStorage: that code stays
 * gone, and a new one shows again.
 */
const promo = ref<Promo | null>(null)
const closing = ref(false)
const copied = ref(false)

const chip = useTemplateRef<HTMLButtonElement>('chip')
/** The chip's width with the code in it, held while it says `copied`. */
const chipWidth = ref('')

const storageKey = (id: string): string => `promo-dismissed:${id}`

onMounted(async () => {
  const offer = await $fetch<PppOffer | null>('/_api/billing/offer').catch(() => null)
  const found = promoFrom(offer, PRO_PLAN)
  if (!found) return

  try {
    if (localStorage.getItem(storageKey(found.promoId))) return
  }
  catch {
    // storage refused — show it; dismissing still works for this page view
  }

  promo.value = found
  await nextTick()
  if (chip.value) chipWidth.value = `${chip.value.offsetWidth}px`
})

let revert: ReturnType<typeof setTimeout> | undefined

async function copy(): Promise<void> {
  if (!promo.value) return

  try {
    await navigator.clipboard.writeText(promo.value.code)
  }
  catch {
    return
  }

  copied.value = true
  clearTimeout(revert)
  revert = setTimeout(() => { copied.value = false }, 1400)
}

const reducedMotion = (): boolean => window.matchMedia('(prefers-reduced-motion: reduce)').matches

/** Fades, then collapses — 0.2s each, or the fade alone under reduced motion. */
function dismiss(): void {
  const id = promo.value?.promoId
  if (!id) return

  try {
    localStorage.setItem(storageKey(id), '1')
  }
  catch {
    // storage refused — the banner still goes for this page view
  }

  closing.value = true
  setTimeout(() => { promo.value = null }, reducedMotion() ? 200 : 400)
}

onBeforeUnmount(() => clearTimeout(revert))
</script>

<template>
  <aside v-if="promo" class="promo" :class="{ 'promo--closing': closing }" aria-label="Regional offer">
    <div class="promo__clip">
      <div class="promo__row">
        <p class="promo__copy">
          {{ PROMO_COPY.lead }}
          <strong class="promo__off">{{ promo.amount }} {{ PROMO_COPY.off }}</strong>
          {{ PROMO_COPY.for }} <span class="promo__country">{{ promo.country }}</span>,
          <span class="promo__with">
            {{ PROMO_COPY.with }}
            <button
              ref="chip"
              type="button"
              class="promo__code"
              :style="{ minWidth: chipWidth }"
              :aria-label="`Copy code ${promo.code}`"
              @click="copy"
            >{{ copied ? PROMO_COPY.copied : promo.code }}</button>
          </span>
        </p>

        <span class="lh-sr" aria-live="polite">{{ copied ? 'Copied' : '' }}</span>

        <button type="button" class="promo__dismiss" aria-label="Dismiss" @click="dismiss">✕</button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
/* Full bleed, in flow above the navbar. The grid row is what collapses on
   dismiss: 1fr to 0fr animates the strip's height without measuring it. */
.promo {
  display: grid;
  grid-template-rows: 1fr;
  background: #202020;
  transition:
    opacity 0.2s ease-out,
    grid-template-rows 0.2s ease-out 0.2s;
}

.promo--closing {
  grid-template-rows: 0fr;
  opacity: 0;
}

.promo__clip {
  min-height: 0;
  overflow: hidden;
}

.promo__row {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 6px 10px;
  max-width: 912px;
  margin: 0 auto;
  padding: 10px 56px;
  text-align: center;
  font: var(--text-caption);
  font-style: normal;
  color: #c9c9c9;
}

.promo__copy {
  margin: 0;
  text-wrap: balance;
}

.promo__off {
  font-weight: 700;
  color: #ff5a6e;
}

.promo__country {
  font-weight: 400;
  color: #f2f2f2;
}

/* `with` and the chip break as one, so the chip never starts a line alone. */
.promo__with {
  white-space: nowrap;
}

.promo__code {
  flex: none;
  box-sizing: border-box;
  /* with the space before it, the reference's 10px gap from `with` */
  margin-left: 6px;
  padding: 1px 10px;
  border: 1px dashed rgba(242, 164, 31, 0.5);
  border-radius: 4px;
  background: transparent;
  color: #f2a41f;
  font: var(--text-label-mono);
  font-weight: 700;
  letter-spacing: 0.04em;
  white-space: nowrap;
  cursor: pointer;
  transition: background-color 0.2s ease-out;
}

.promo__code:hover {
  background: rgba(242, 164, 31, 0.1);
}

.promo__dismiss {
  position: absolute;
  top: 50%;
  right: 16px;
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  padding: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: #8a8a8a;
  font-size: 11px;
  line-height: 1;
  cursor: pointer;
  transform: translateY(-50%);
  transition: background-color 0.2s ease-out, color 0.2s ease-out;
}

.promo__dismiss:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f2f2f2;
}

.promo__code:focus-visible,
.promo__dismiss:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

@media (max-width: 620px) {
  .promo__row {
    padding: 10px 44px 10px 16px;
  }
}

/* The fade alone: no collapse, the strip goes when the fade ends. */
@media (prefers-reduced-motion: reduce) {
  .promo {
    transition: opacity 0.2s ease-out;
  }

  .promo--closing {
    grid-template-rows: 1fr;
  }
}
</style>
