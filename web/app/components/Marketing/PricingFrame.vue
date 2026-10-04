<script setup lang="ts">
import type { CataloguePlan } from '@/data/Catalogue'
import { plans, shelf } from '@/data/Catalogue'
import { tracks } from '@/data/Tracks'

/**
 * The pricing frame: an image panel with the pitch, and a panel per track.
 *
 * What is for sale, what it costs and which books each track holds are
 * compiled in from `Catalogue.ts`, not fetched. `/pricing` is prerendered from
 * `web/` alone, with no api to ask, and a fetch there baked an em dash into
 * every price. `lighthouse-prices catalogue` writes that file from the same
 * declaration it reconciles against stripe, so the number is still the one
 * charged. `Tracks.ts` only supplies the name and the line under it.
 *
 * The one thing not compiled in is the purchasing-power coupon, which depends
 * on where the reader is — see `couponFor`.
 */
defineProps<{
  eyebrow?: string
}>()

// The coupons the api offers this reader, per plan. Read in the browser only.
const { data: offers } = await usePlans()

interface Panel {
  key: string
  name: string
  blurb: string
  featured: boolean
  /** What the panel prices and the button buys: yearly, or outright when a
   *  track is only sold that way. */
  lead: CataloguePlan | undefined
  /** The outright plan, offered beside a yearly one. */
  outright: CataloguePlan | undefined
  titles: string[]
}

/**
 * The books on a track, in that track's reading order. `all` is every book on
 * the shelf, including the ones on no track.
 */
function titlesOn(key: string): string[] {
  if (key === 'all') return shelf.map(book => book.title)

  return shelf
    .filter(book => book.tracks[key] !== undefined)
    .sort((a, b) => (a.tracks[key] ?? 0) - (b.tracks[key] ?? 0))
    .map(book => book.title)
}

// The catalogue's tracks, in the order Tracks.ts reads them; anything it does
// not know about still gets a panel, after the rest.
const order = tracks.map(track => track.key as string)
const keys = [...new Set(plans.map(plan => plan.track))]
  .sort((a, b) => (order.indexOf(a) + 1 || 99) - (order.indexOf(b) + 1 || 99))

const panels: Panel[] = keys.map((key) => {
  const known = tracks.find(track => track.key === key)
  const yearly = plans.find(plan => plan.track === key && plan.recurring)
  const outright = plans.find(plan => plan.track === key && !plan.recurring)

  return {
    key,
    name: known?.name ?? key,
    blurb: known?.blurb ?? '',
    featured: known?.featured ?? false,
    lead: yearly ?? outright,
    outright: yearly ? outright : undefined,
    titles: titlesOn(key),
  }
})

/**
 * The coupon that comes off this particular plan, if any. Asked per plan, not
 * per reader: a stripe coupon is restricted to one plan's product, so two
 * panels can carry different codes in the same country.
 */
function couponFor(offer: CataloguePlan | undefined): { code: string, percent: number } | null {
  if (!offer) return null

  return (offers.value ?? []).find(o => o.plan === offer.plan)?.coupon ?? null
}

/** What a plan costs once its coupon is applied, or null when the list price is the price. */
function reduced(offer: CataloguePlan | undefined): string | null {
  const percent = couponFor(offer)?.percent

  if (!offer?.amount || !percent) return null

  return money(afterDiscount(offer.amount, percent))
}

/** `4900` reads as `$49`; a price nobody knows reads as a dash. */
function priced(offer: CataloguePlan | undefined): string {
  return offer?.amount ? money(offer.amount) : '—'
}

const pad = (n: number): string => String(n).padStart(2, '0')

const { checkout, busy, reason } = useBilling()
const { isSignedIn } = useReader()
const route = useRoute()

/**
 * Buying needs a session, because the api ties the purchase to an account.
 * An anonymous reader signs in first and comes back to where they were.
 */
async function buy(plan: string | undefined): Promise<void> {
  if (!plan) return

  if (!isSignedIn.value) {
    await navigateTo(`/login?redirect=${encodeURIComponent(`${route.path}#pricing`)}`)

    return
  }

  await checkout(plan)
}

// TODO(pricing-image): an owned placeholder — the lighthouse painting from the
// book art. Swap for a dedicated pricing image once one is made; keep it on
// the spaces CDN and object-position 50% 40%.
const IMAGE = 'https://spaces.projectlighthouse.io/books/art/lighthouse.001.jpeg'
</script>

<template>
  <div id="pricing" class="pricing lh-wide">
    <div class="frame lh-inverse">
      <div class="pitch">
        <img :src="IMAGE" alt="" loading="lazy">
        <div class="shade" aria-hidden="true" />
        <div class="words">
          <p class="eyebrow">{{ eyebrow ?? '04 — pricing' }}</p>
          <h2 class="lh-h1">Your lighthouse</h2>
          <p class="lede">
            A year of every book on the track, including the chapters that ship during it.
            Billed yearly, or bought once and kept.
          </p>
        </div>
      </div>

      <div class="plans">
        <p v-if="reason" class="reason" role="alert">{{ reason }}</p>

        <div v-for="panel in panels" :key="panel.key" class="plan" :class="{ 'is-featured': panel.featured }">
          <div class="plan-head">
            <div class="who">
              <span class="name">{{ panel.name }}</span>
              <span class="blurb">{{ panel.blurb }}</span>
            </div>
            <div class="price">
              <!-- The list price stays, struck through: the reduced number
                   means nothing without the one it came down from. -->
              <span v-if="reduced(panel.lead)" class="was lh-num">{{ priced(panel.lead) }}</span>
              <span class="amount lh-num">{{ reduced(panel.lead) ?? priced(panel.lead) }}</span>
              <span class="per">
                {{ panel.lead?.recurring === false ? 'once' : 'per year' }} · {{ panel.titles.length }} books
              </span>
              <!-- Advertised, not applied: the reader types the code at
                   stripe. Beside the plan it comes off, since a coupon is
                   restricted to one plan and two panels can differ. -->
              <span v-if="couponFor(panel.lead)" class="coupon">
                {{ couponFor(panel.lead)?.percent }}% off — enter
                <code>{{ couponFor(panel.lead)?.code }}</code> at checkout
              </span>
            </div>
          </div>

          <ol v-if="panel.titles.length" class="titles">
            <li v-for="(title, i) in panel.titles" :key="title">
              <span class="n lh-num">{{ pad(i + 1) }}</span>{{ title }}
            </li>
          </ol>

          <div class="plan-foot">
            <button
              v-if="panel.outright"
              type="button"
              class="outright"
              :disabled="busy"
              @click="buy(panel.outright?.plan)"
            >
              or {{ reduced(panel.outright) ?? priced(panel.outright) }} once, yours to keep{{
                couponFor(panel.outright) ? ` with ${couponFor(panel.outright)?.code}` : '' }} →
            </button>

            <UiButton
              variant="inverse"
              size="lg"
              cta="pro-dark"
              flame
              :disabled="busy || !panel.lead"
              @click="buy(panel.lead?.plan)"
            >
              {{ busy ? 'One moment…' : 'Get Pro' }}
            </UiButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pricing { margin-top: 128px; }

.frame {
  display: grid;
  grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
  gap: var(--space-2);
}

.pitch {
  position: sticky;
  top: var(--space-6);
  align-self: start;
  min-height: 420px;
  height: min(640px, calc(100vh - 48px));
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--surface-inverse);
}

.pitch img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: 50% 40%;
}

.shade {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgba(32, 32, 32, 0) 45%, rgba(32, 32, 32, 0.9));
}

.words {
  position: absolute;
  left: 28px;
  right: 28px;
  bottom: 28px;
  display: grid;
  gap: var(--space-3);
}

.eyebrow {
  margin: 0;
  font: var(--text-label-mono);
  color: var(--ink-inverse-secondary);
}

.lede {
  margin: 0;
  max-width: 360px;
  font: var(--text-body-sm);
  color: var(--ink-inverse-secondary);
  text-wrap: pretty;
}

.plans {
  display: grid;
  gap: var(--space-2);
  min-width: 0;
  align-content: start;
}

.reason {
  margin: 0;
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.08);
  font: var(--text-body-sm);
}

.plan {
  display: grid;
  gap: var(--space-6);
  padding: 28px var(--space-8);
  border: 1px solid var(--ink-inverse-faint);
  border-radius: var(--radius-md);
  background: transparent;
}

.plan.is-featured {
  border-color: rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.04);
}

.plan-head {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2) var(--space-8);
  align-items: start;
}

.who {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.name {
  font: var(--text-h2);
  letter-spacing: var(--tracking-h2);
  text-wrap: balance;
}

.blurb {
  max-width: 440px;
  font: var(--text-body-sm);
  color: var(--ink-inverse-secondary);
  text-wrap: pretty;
}

.price {
  display: grid;
  justify-items: end;
  gap: 2px;
  text-align: right;
}

.amount {
  font: 500 44px/44px var(--font-sans);
  letter-spacing: var(--tracking-display);
}

.per {
  white-space: nowrap;
  font: var(--text-label-mono);
  color: var(--ink-inverse-muted);
}

.was {
  font: var(--text-body-sm);
  color: var(--ink-inverse-muted);
  text-decoration: line-through;
}

.coupon {
  font: var(--text-label-mono);
  color: var(--ink-inverse-secondary);
}

.titles {
  margin: 0;
  padding: var(--space-5) 0 0;
  list-style: none;
  border-top: 1px dashed var(--ink-inverse-faint);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-2) var(--space-6);
}

.titles li {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr);
  gap: var(--space-2);
  font: var(--text-body-sm);
  color: var(--ink-inverse);
}

.n {
  padding-top: 3px;
  font: var(--text-label-mono);
  color: var(--ink-inverse-muted);
}

.plan-foot {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: var(--space-3) var(--space-6);
}

.outright {
  padding: 0;
  border: 0;
  background: transparent;
  font: var(--text-label-mono);
  color: var(--ink-inverse-secondary);
  cursor: pointer;
  transition: color var(--duration) var(--ease-out);
}

.outright:hover:not(:disabled) { color: var(--ink-inverse); }
.outright:disabled { cursor: not-allowed; opacity: 0.4; }

@media (max-width: 960px) {
  .frame { grid-template-columns: minmax(0, 1fr); }
  .pitch { position: relative; top: 0; min-height: 0; height: auto; aspect-ratio: 16 / 10; }
}

@media (max-width: 600px) {
  .plan { padding: var(--space-6); }
  .plan-head { grid-template-columns: minmax(0, 1fr); }
  .price { justify-items: start; text-align: left; }
  .titles { grid-template-columns: minmax(0, 1fr); }
  .plan-foot { justify-content: flex-start; }
}
</style>
