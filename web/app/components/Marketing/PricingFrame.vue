<script setup lang="ts">
import type { CatalogueBook, CataloguePlan } from '@/data/Catalogue'
import type { Coupon } from '@/composables/UsePlans'
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
  /** How many books the plan opens today. */
  count: number
  /** The books, grouped by family, for a plan that is a set of books. */
  groups: Group[]
  /** What the plan adds, for the one that is everything — listing the same
   *  books again under it would hide the difference rather than show it. */
  includes: string[]
}

interface Group {
  name: string
  titles: string[]
}

/**
 * Which family each book belongs to, so related books sit together — Go
 * Fundamentals directly above Go Intermediate rather than across the panel.
 * In reading order within a group; a book not listed here lands in "more"
 * until it is, rather than disappearing.
 *
 * ponytail: a hand-kept list, not a field in ohara. Ten books in four
 * families; move it into book.yaml if the shelf outgrows a glance.
 */
const FAMILIES: { name: string, slugs: string[] }[] = [
  { name: 'fundamentals', slugs: ['c-programming', 'dsa-fundamentals', 'os-fundamentals', 'networking-fundamentals'] },
  { name: 'go', slugs: ['go-fundamentals', 'go-intermediate', 'shipping-go-web-services'] },
  { name: 'rust', slugs: ['rust-from-zero', 'rust-101s'] },
  { name: 'interview', slugs: ['crack-the-interview'] },
]

/**
 * The books on a track, in that track's reading order. `all` is every book on
 * the shelf, including the ones on no track.
 */
function booksOn(key: string): CatalogueBook[] {
  if (key === 'all') return shelf

  return shelf
    .filter(book => book.tracks[key] !== undefined)
    .sort((a, b) => (a.tracks[key] ?? 0) - (b.tracks[key] ?? 0))
}

/** A track's books, in their families, each family in its own reading order. */
function grouped(books: CatalogueBook[]): Group[] {
  const groups = FAMILIES
    .map(family => ({
      name: family.name,
      titles: family.slugs
        .map(slug => books.find(book => book.slug === slug)?.title)
        .filter((title): title is string => !!title),
    }))
    .filter(group => group.titles.length)

  const known = FAMILIES.flatMap(family => family.slugs)
  const rest = books.filter(book => !known.includes(book.slug)).map(book => book.title)

  return rest.length ? [...groups, { name: 'more', titles: rest }] : groups
}

/** The books every other plan sells, which the everything plan contains. */
const foundationCount = booksOn('foundation').length

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
    count: booksOn(key).length,
    groups: key === 'all' ? [] : grouped(booksOn(key)),
    includes: key === 'all'
      ? [
          `Everything in Foundations: all ${foundationCount} books`,
          'Every book published from now on',
          'Every project, new ones included',
          'Pay once, keep it for good',
        ]
      : [],
  }
})

/**
 * The coupon that comes off this particular plan, if any. Asked per plan, not
 * per reader: a stripe coupon is restricted to one plan's product, so two
 * panels can carry different codes in the same country.
 */
function couponFor(offer: CataloguePlan | undefined): Coupon | null {
  if (!offer) return null

  return (offers.value ?? []).find(o => o.plan === offer.plan)?.coupon ?? null
}

/** What a plan costs once its coupon is applied, or null when the list price is the price. */
function reduced(offer: CataloguePlan | undefined): string | null {
  const coupon = couponFor(offer)

  if (!offer?.amount || !coupon) return null

  return money(afterOff(offer.amount, coupon))
}

/** `4900` reads as `$49`; a price nobody knows reads as a dash. */
function priced(offer: CataloguePlan | undefined): string {
  return offer?.amount ? money(offer.amount) : '—'
}


/**
 * Buying goes through `/checkout`, which signs an anonymous reader in first and
 * comes back with the plan intact — then stripe, or billing for a reader who
 * already pays. See `pages/checkout.vue`.
 */
async function buy(plan: string | undefined): Promise<void> {
  if (!plan) return

  await navigateTo(checkoutUrl(plan))
}

// Served from public/, so it ships in the image rather than off the CDN.
const IMAGE = '/pricing-lighthouse.jpg'
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
                {{ panel.lead?.recurring === false ? 'once · forever' : `per year · ${panel.count} books` }}
              </span>
              <!-- Advertised, not applied: the reader types the code at
                   stripe. Beside the plan it comes off, since a coupon is
                   restricted to one plan and two panels can differ. -->
              <span v-if="couponFor(panel.lead)" class="coupon">
                {{ offLabel(couponFor(panel.lead)!) }} — enter
                <code>{{ couponFor(panel.lead)?.code }}</code> at checkout
              </span>
            </div>
          </div>

          <div v-if="panel.groups.length" class="groups">
            <div v-for="group in panel.groups" :key="group.name" class="group">
              <span class="group-name">{{ group.name }}</span>
              <ul class="titles">
                <li v-for="title in group.titles" :key="title">{{ title }}</li>
              </ul>
            </div>
            <!-- Foundations opens the projects too — every membership does — so
                 it says so, and the everything panel's list is not read as
                 claiming them for itself alone. -->
            <p class="also">Plus every project, while you are subscribed.</p>
          </div>

          <ul v-if="panel.includes.length" class="includes">
            <li v-for="line in panel.includes" :key="line">{{ line }}</li>
          </ul>

          <div class="plan-foot">
            <button
              v-if="panel.outright"
              type="button"
              class="outright"
              @click="buy(panel.outright?.plan)"
            >
              or {{ reduced(panel.outright) ?? priced(panel.outright) }} once, yours to keep →
            </button>

            <UiButton
              variant="inverse"
              size="lg"
              cta="pro-dark"
              flame
              :disabled="!panel.lead"
              @click="buy(panel.lead?.plan)"
            >
              Get Pro
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
  position: relative;
  min-height: 420px;
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

.groups {
  margin: 0;
  padding: var(--space-5) 0 0;
  border-top: 1px dashed var(--ink-inverse-faint);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-5) var(--space-6);
}

.group {
  display: grid;
  gap: var(--space-2);
  align-content: start;
}

.group-name {
  font: var(--text-label-mono);
  color: var(--ink-inverse-muted);
}

.titles,
.includes {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: var(--space-2);
}

.titles li,
.includes li {
  font: var(--text-body-sm);
  color: var(--ink-inverse);
}

.also {
  grid-column: 1 / -1;
  margin: 0;
  font: var(--text-body-sm);
  color: var(--ink-inverse-muted);
}

.includes {
  padding: var(--space-5) 0 0;
  border-top: 1px dashed var(--ink-inverse-faint);
}

/* A tick before each, so the list reads as what you get rather than as books. */
.includes li::before {
  content: '✓';
  margin-right: var(--space-3);
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
  .groups { grid-template-columns: minmax(0, 1fr); }
  .plan-foot { justify-content: flex-start; }
}
</style>
