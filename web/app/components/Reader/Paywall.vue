<script setup lang="ts">
import type { Book } from '@/types/Content'

/**
 * The card that stands where a paid region was.
 *
 * `ohara::body` leaves a `<div data-paywall data-topics="…">` at the position
 * the region held, so this lands in the middle of a lesson rather than after
 * it — the same shape the laravel app has, and the same wording.
 *
 * **It sells a track.** The laravel card offers a Pro membership at monthly,
 * yearly or lifetime with the prices written into the component. This rebuild
 * sells a track and nothing else, and the amounts come from
 * `/api/billing/plans` — one number, one place — so the card offers the
 * cheapest track that actually contains the book being read.
 */
const props = withDefaults(defineProps<{
  topics?: string[]
  /** Slug of the book this lesson belongs to, for picking the track. */
  bookSlug?: string
  /** Its title, so the offer names something concrete. */
  book?: string
}>(), {
  topics: () => [],
  bookSlug: '',
  book: '',
})

/** Three, then "and more" — the card names, it does not list. */
const named = computed<string[]>(() => props.topics.slice(0, 3))
const more = computed<boolean>(() => props.topics.length > 3)

/** The discount this plan is offered in the reader's country, if any.
 *  Advertised, not applied — the reader types `code` at stripe. */
interface Coupon {
  code: string
  percent: number
}

/** One purchasable plan, as `/api/billing/plans` reports it. */
interface Offer {
  amount: number
  currency: string
  plan: string
  recurring: boolean
  track: string
  /** The coupon that comes off *this* plan. A stripe coupon is restricted to
   *  one plan's product, so a discount is a fact about the reader and the
   *  plan together — never about the reader alone. */
  coupon: Coupon | null
}

/** The whole catalogue answer — the offers, and the country the api read from
 *  Cloudflare. */
interface Catalogue {
  country: string | null
  plans: Offer[]
}

const { data: catalogue } = await useAsyncData('billing-plans', () =>
  $fetch<Catalogue>('/_api/billing/plans')
    .catch(() => ({ country: null, plans: [] } as Catalogue)))

const offers = computed<Offer[]>(() => catalogue.value?.plans ?? [])

const { data: books } = await useAsyncData('paywall-books', () =>
  $fetch<Book[]>('/_api/books').catch(() => [] as Book[]), { default: () => [] })

/**
 * The tracks that carry this book, plus `all`, which carries everything.
 *
 * Read from the api rather than a list here: which books are on a track is the
 * `tracks:` map in each `book.yaml`, and it is what rust grants entitlements
 * from. A second copy would be a second thing to keep right.
 */
const tracks = computed<string[]>(() => {
  const book = books.value.find(b => b.slug === props.bookSlug)

  return [...Object.keys(book?.tracks ?? {}), 'all']
})

/** The cheapest offer of a kind that would actually unlock this lesson. */
function cheapest(recurring: boolean): Offer | undefined {
  return offers.value
    .filter(o => o.recurring === recurring && tracks.value.includes(o.track))
    .sort((a, b) => a.amount - b.amount)[0]
}

const yearly = computed<Offer | undefined>(() => cheapest(true))
const lifetime = computed<Offer | undefined>(() => cheapest(false))

/**
 * What an offer unlocks, named on the button that sells it.
 *
 * The two buttons are picked independently — cheapest of each kind that covers
 * this book — so they are routinely not the same bundle. On a foundation book
 * the yearly is the foundation track and the lifetime is `all`, and before this
 * the card put $99/yr beside $499 once with nothing saying the second was five
 * times the catalogue rather than five times the price.
 *
 * The slug is what the wire carries. `all` is the one that needs a word rather
 * than its name, because `track::EVERYTHING` is called `all` in the api and
 * nothing else on this page says so.
 */
function scope(o: Offer | undefined): string {
  if (!o) return ''

  return o.track === 'all' ? 'everything' : o.track
}

type Choice = 'yearly' | 'lifetime'

/**
 * Whichever kind this book actually has, preferring the yearly.
 *
 * Not a constant `'yearly'`: the two buttons each render only if an offer of
 * that kind covers the book, and there is no longer a yearly that covers every
 * book — `all` is sold outright. A book on no track but `all` therefore shows
 * one button, and defaulting to the kind that is not there selects nothing,
 * leaves `offer` undefined and disables the buy button for good.
 *
 * Safe to read here: both offers come from a `useAsyncData` this component
 * awaits, so they are settled before this runs and the server and the client
 * start from the same one.
 */
const chosen = ref<Choice>(yearly.value ? 'yearly' : 'lifetime')

const offer = computed<Offer | undefined>(
  () => (chosen.value === 'lifetime' ? lifetime.value : yearly.value),
)

/** Minor units to a price tag. The arithmetic stays in minor units on the rust
 *  side — this only reads it. */
function priced(o: Offer | undefined): string | null {
  return o ? money(o.amount) : null
}

/**
 * The plan's price once its coupon is applied, or null when it has none.
 *
 * Read off the offer rather than off the answer: the api works out which tier
 * the reader's country earns *on this plan*, and a coupon is restricted to one
 * plan's product at stripe. Quoting one plan's discount against another's price
 * is a number stripe would refuse to honour.
 */
function reduced(o: Offer | undefined): string | null {
  const percent = o?.coupon?.percent

  if (!o || !percent) return null

  return money(afterDiscount(o.amount, percent))
}

const summary = computed<string>(() =>
  chosen.value === 'lifetime'
    ? 'pay once, and every future chapter of it is yours'
    : 'best value — cancel any time')

const { checkout, busy, reason } = useBilling()
const { isSignedIn } = useReader()

const route = useRoute()

async function buy(): Promise<void> {
  const plan = offer.value?.plan
  if (!plan || busy.value) return

  // Back to the lesson afterwards, not to the shop: they were reading.
  if (!isSignedIn.value) {
    await navigateTo(`/login?redirect=${encodeURIComponent(route.fullPath)}`)
    return
  }

  await checkout(plan)
}
</script>

<template>
  <div class="wall">
    <!-- The prose above fades out rather than stopping on a line, so the cut
         reads as more rather than as an ending. -->
    <div class="wall__fade" aria-hidden="true" />

    <section class="wall__card">
      <p class="wall__eyebrow">keep reading</p>

      <h2 class="wall__title">There's more to this story</h2>

      <p class="wall__sub">
        <template v-if="named.length">
          The rest of this chapter covers
          <template v-for="(topic, i) in named" :key="topic">
            <span class="wall__topic">{{ topic }}</span
            ><template v-if="i < named.length - 2">, </template
            ><template v-else-if="i === named.length - 2"> and </template>
          </template>
          <template v-if="more">, and more</template>.
        </template>
        <template v-else>
          The rest of this chapter goes deeper, with working code and the kind
          of detail that actually sticks.
        </template>
        Unlock it — and every other lesson on the track.
      </p>

      <ul class="wall__perks">
        <li>
          <b>{{ books.length }} books</b> — OS internals, networking, Go, Rust, C and DSA
        </li>
        <li>
          <b>Build-your-own projects</b> — Docker, DNS and HTTP servers, checked on your own machine
        </li>
        <li>
          <b>Every future chapter</b> of the track, included as it ships
        </li>
      </ul>

      <h3 class="wall__plans-head">Start the voyage</h3>

      <div class="wall__plans">
        <button
          v-if="yearly"
          type="button"
          class="wall__plan"
          :class="{ 'wall__plan--on': chosen === 'yearly' }"
          :aria-pressed="chosen === 'yearly'"
          @click="chosen = 'yearly'"
        >
          <span class="wall__radio" aria-hidden="true" />
          <span class="wall__plan-body">
            <span class="wall__plan-name">Yearly</span>
            <span class="wall__plan-desc">{{ scope(yearly) }} — and everything shipped to it while you subscribe</span>
          </span>
          <span class="wall__price">
            <s v-if="reduced(yearly)" class="wall__was">{{ priced(yearly) }}</s>
            <b>{{ reduced(yearly) ?? priced(yearly) }}</b><span>/yr</span>
          </span>
        </button>

        <button
          v-if="lifetime"
          type="button"
          class="wall__plan"
          :class="{ 'wall__plan--on': chosen === 'lifetime' }"
          :aria-pressed="chosen === 'lifetime'"
          @click="chosen = 'lifetime'"
        >
          <span class="wall__radio" aria-hidden="true" />
          <span class="wall__plan-body">
            <span class="wall__plan-name">Lifetime</span>
            <span class="wall__plan-desc">{{ scope(lifetime) }} — pay once, yours for good</span>
          </span>
          <span class="wall__price">
            <s v-if="reduced(lifetime)" class="wall__was">{{ priced(lifetime) }}</s>
            <b>{{ reduced(lifetime) ?? priced(lifetime) }}</b><span>once</span>
          </span>
        </button>
      </div>

      <!-- No claim about where the reader is. The api answers with whichever
           tier they are offered and does not say whether it named their
           country or every country, and the declaration now carries one that
           names none — so "where you are" was telling a reader in London that
           their location earned them a discount everybody gets. -->
      <!-- Only against a plan the coupon actually comes off. Shown for the
           selected one, because that is the price the button is about to
           charge. -->
      <p v-if="offer?.coupon" class="wall__ppp">
        {{ offer.coupon.percent }}% off — enter
        <code>{{ offer.coupon.code }}</code> at checkout.
      </p>

      <p v-if="reason" class="wall__reason" role="alert">{{ reason }}</p>

      <div class="wall__cta">
        <p class="wall__summary">{{ summary }}</p>
        <div class="wall__actions">
          <NuxtLink to="/pricing" class="wall__view">view offering</NuxtLink>
          <button type="button" class="wall__buy" :disabled="busy || !offer" @click="buy">
            {{ busy ? 'Loading…' : `Get ${book || 'the book'}` }}
          </button>
        </div>
      </div>

      <!-- Only to somebody who is not. `isSignedIn` was already read here for
           `buy`, so the card knew and asked anyway — a signed-in reader was
           being told to sign in, on the one card that is asking them to
           spend money. -->
      <p v-if="!isSignedIn" class="wall__note">
        Already a member?
        <NuxtLink to="/login">Sign in</NuxtLink>
      </p>
    </section>
  </div>
</template>

<style scoped>

.wall {
    position: relative;
    margin: 24px 0;
}

/* Sits above the card and over the prose before it. `--surface-page` rather
   than white, so it fades into the reader's own ground in either theme. */
.wall__fade {
    pointer-events: none;
    position: absolute;
    top: -128px;
    right: 0;
    left: 0;
    height: 128px;
    background: linear-gradient(to bottom, transparent, var(--surface-page));
}

.wall__card {
    position: relative;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface-raised);
    padding: 34px 32px;
}

.wall__eyebrow {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--accent-strong);
}

.wall__title {
    margin: 14px 0 0;
    font-family: var(--font-serif);
    font-weight: 600;
    font-size: 30px;
    line-height: 1.1;
    letter-spacing: -0.015em;
    color: var(--ink);
}

.wall__sub {
    margin: 12px 0 0;
    max-width: 40em;
    font-family: var(--font-serif);
    font-size: 17px;
    line-height: 1.62;
    color: var(--ink-secondary);
    text-wrap: pretty;
}

.wall__topic {
    font-weight: 500;
    color: var(--ink);
}

.wall__perks {
    margin: 26px 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 11px;
    font-family: var(--font-serif);
    font-size: 16px;
    line-height: 1.45;
    color: var(--ink-secondary);
}

.wall__perks b {
    font-weight: 600;
    color: var(--ink);
}

.wall__plans-head {
    margin: 30px 0 12px;
    font-family: var(--font-serif);
    font-weight: 600;
    font-size: 20px;
    letter-spacing: -0.01em;
    color: var(--ink);
}

.wall__plans {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.wall__plan {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 14px 18px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: none;
    cursor: pointer;
    text-align: left;
    transition:
        border-color 140ms,
        background 140ms;
}

.wall__plan:hover {
    border-color: var(--border-strong);
}

.wall__plan--on {
    border-color: var(--accent-strong);
    background: var(--accent-tint);
}

.wall__radio {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    flex: none;
    border: 1.5px solid var(--border-strong);
    border-radius: 50%;
    transition: border-color 140ms;
}

.wall__radio::after {
    content: '';
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--accent-strong);
    transform: scale(0);
    transition: transform 160ms;
}

.wall__plan--on .wall__radio {
    border-color: var(--accent-strong);
}

.wall__plan--on .wall__radio::after {
    transform: scale(1);
}

.wall__plan-body {
    flex: 1;
    min-width: 0;
}

.wall__plan-name {
    display: block;
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 16px;
    color: var(--ink);
}

.wall__plan-desc {
    display: block;
    margin-top: 2px;
    font-family: var(--font-serif);
    font-size: 14px;
    color: var(--ink-secondary);
}

.wall__price {
    flex: none;
    white-space: nowrap;
    text-align: right;
}

.wall__price b {
    font-family: var(--font-serif);
    font-weight: 600;
    font-size: 21px;
    color: var(--ink);
}

.wall__price span {
    margin-left: 3px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ink-secondary);
}

/* The list price beside the reduced one. Quiet and small: it is context for
   the number next to it, not a second price being offered. */
.wall__was {
    margin-right: 5px;
    font-family: var(--font-serif);
    font-size: 15px;
    color: var(--ink-faint);
}

.wall__ppp {
    margin: 12px 0 0;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ink-secondary);
}

.wall__ppp code {
    padding: 1px 5px;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    color: var(--ink);
}

.wall__reason {
    margin: 12px 0 0;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ink);
}

.wall__summary {
    margin: 0;
    font-family: var(--font-serif);
    font-size: 14px;
    color: var(--ink-secondary);
}

.wall__actions {
    display: flex;
    align-items: center;
    gap: 16px;
}

.wall__view {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ink-secondary);
    transition: color 140ms;
}

.wall__view:hover {
    color: var(--ink);
}

.wall__note {
    margin: 18px 0 0;
    font-family: var(--font-serif);
    font-size: 14px;
    color: var(--ink-secondary);
}

.wall__note a {
    color: var(--accent-strong);
    text-decoration: underline;
    text-underline-offset: 2px;
}

.wall__cta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 24px;
}

.wall__buy {
    border: 0;
    cursor: pointer;
    border-radius: 6px;
    background: var(--ink);
    padding: 13px 22px;
    font-family: var(--font-sans);
    font-size: 15px;
    font-weight: 500;
    color: var(--ink-inverse);
    transition: background 140ms;
}

.wall__buy:hover:not(:disabled) {
    background: var(--ink-secondary);
}

.wall__buy:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}



@media (max-width: 560px) {
    .wall__card {
        padding: 26px 20px;
    }

    .wall__title {
        font-size: 24px;
    }

    .wall__buy {
        flex: 1 1 auto;
        text-align: center;
    }
}
</style>
