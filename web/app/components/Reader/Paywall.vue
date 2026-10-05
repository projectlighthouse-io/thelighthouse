<script setup lang="ts">
import type { CataloguePlan } from '@/data/Catalogue'
import type { Coupon } from '@/composables/UsePlans'
import type { Book } from '@/types/Content'
import type { PaywallBook, PaywallPlan, PaywallSection } from '@/types/Paywall'
import { plans as catalogue } from '@/data/Catalogue'
import { tracks as trackWords } from '@/data/Tracks'

/**
 * The card that stands where a paid region was.
 *
 * `ohara::body` leaves a `<div data-paywall>` at the position the region held,
 * so this lands directly after the last free paragraph — see the lesson page,
 * which teleports it there.
 *
 * **Nothing on it is a literal count.** Sections, minutes, chapters and prices
 * all arrive as data or are counted from it here, so a change to the lesson,
 * the book or the catalogue changes the card without anyone editing it.
 *
 * **It sells a track.** The card offers the cheapest yearly and the cheapest
 * outright plan that actually contain the book being read.
 *
 * **Priced exactly as `Marketing/PricingFrame.vue` prices.** The plans and
 * their amounts are compiled in from `Catalogue.ts`; the purchasing-power
 * coupon comes from `usePlans()`, in the browser only, and is applied with the
 * same `afterOff`. Two reasons it is not fetched on the server: a lesson is
 * edge-cached (`s-maxage` on `/books/**`), so a server-side read baked one
 * country's tier into the html every other country was served; and the
 * server-side `$fetch` did not forward `cf-ipcountry`, so that one country was
 * always "nowhere".
 */
const props = defineProps<{
  book: PaywallBook
  /** `04` */
  chapter: { number: string }
  sections: PaywallSection[]
  /** The last open section — where the reader stopped. Empty when none is. */
  currentSection: string
}>()

/* ---------- the lesson, counted ---------- */

const readCount = computed<number>(() => props.sections.filter(s => !s.locked).length)
const total = computed<number>(() => props.sections.length)
const minutesLeft = computed<number>(() =>
  props.sections.filter(s => s.locked).reduce((sum, s) => sum + s.minutes, 0))
const locked = computed<PaywallSection[]>(() => props.sections.filter(s => s.locked))

/** Five rows is enough to show what is behind the card; past that the list
 *  would be the whole card. */
const SHOWN = 5
const lockedShown = computed<PaywallSection[]>(() => locked.value.slice(0, SHOWN))

/** What is left of the book after this lesson, for the line under the list. */
const moreLessonsWords = computed<string>(() => {
  const { moreLessons: lessons, moreLessonChapters: chapters } = props.book

  return `and ${lessons} more ${lessons === 1 ? 'lesson' : 'lessons'} over ${chapters} ${chapters === 1 ? 'chapter' : 'chapters'}`
})

/** Headings keep their markdown backticks in the contents; on the card they
 *  read as stray punctuation. */
const plain = (title: string): string => title.replaceAll('`', '')

type StopState = 'read' | 'current' | 'locked'

function stateOf(section: PaywallSection): StopState {
  if (section.locked) return 'locked'

  return section.n === props.currentSection ? 'current' : 'read'
}

/** Past eight stops the read labels go, so the row does not run together. */
const dense = computed<boolean>(() => total.value > 8)

/** The stop after the current one, whose label the "· here" would run into
 *  on a crowded row. */
const afterCurrent = computed<number>(() => {
  const at = props.sections.findIndex(s => s.n === props.currentSection)

  // No current stop (nothing free was read) means no "· here" to collide
  // with — otherwise this pointed at the first stop and hid its label.
  return at === -1 ? -1 : at + 1
})

/* ---------- the plans ---------- */

// The coupons the api offers this reader, per plan. Read in the browser only,
// so until it lands the card shows list prices — the same neutral state the
// pricing frame shows.
const { data: coupons } = await usePlans()

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
  const book = books.value.find(b => b.slug === props.book.slug)

  return [...Object.keys(book?.tracks ?? {}), 'all']
})

/** The cheapest plan of a kind that would actually unlock this lesson. */
function cheapest(recurring: boolean): CataloguePlan | undefined {
  return catalogue
    .filter(o => o.recurring === recurring && tracks.value.includes(o.track))
    .sort((a, b) => a.amount - b.amount)[0]
}

/**
 * The coupon that comes off this particular plan, if any. Asked per plan, not
 * per reader: a stripe coupon is restricted to one plan's product, so two rows
 * can carry different codes in the same country.
 */
function couponFor(o: CataloguePlan): Coupon | null {
  return (coupons.value ?? []).find(c => c.plan === o.plan)?.coupon ?? null
}

/** `$70` or `30%` — what `offLabel` says, without its " off", which the card
 *  writes itself. */
function discountAmount(coupon: Coupon): string {
  return coupon.amount_off !== undefined ? money(coupon.amount_off) : `${coupon.percent ?? 0}%`
}

/**
 * What a plan unlocks, named on its row.
 *
 * The two rows are picked independently — cheapest of each kind that covers
 * this book — so they are routinely not the same bundle, and the note says
 * which each one is.
 */
function trackName(o: CataloguePlan): string {
  return trackWords.find(t => t.key === o.track)?.name ?? o.track
}

function priced(o: CataloguePlan): PaywallPlan {
  const coupon = couponFor(o)
  const reduced = coupon && o.amount ? afterOff(o.amount, coupon) : null

  return {
    id: o.plan,
    name: o.recurring ? 'Yearly' : 'Lifetime',
    note: o.recurring
      ? `${trackName(o)}, and everything shipped to it while you subscribe`
      : `${trackName(o)}, paid once and yours for good`,
    price: money(reduced ?? o.amount),
    listPrice: reduced !== null ? money(o.amount) : undefined,
    per: o.recurring ? '/ year' : 'once',
    discount: coupon ? { amount: discountAmount(coupon), code: coupon.code } : undefined,
    cta: o.button_text ?? (o.recurring ? 'Get yearly access' : 'Get lifetime access'),
    checkoutUrl: checkoutUrl(o.plan),
  }
}

const yearly = computed<CataloguePlan | undefined>(() => cheapest(true))
const lifetime = computed<CataloguePlan | undefined>(() => cheapest(false))

const plans = computed<PaywallPlan[]>(() =>
  [yearly.value, lifetime.value]
    .filter((o): o is CataloguePlan => !!o)
    .map(priced))

/**
 * Whichever kind this book actually has, preferring the yearly.
 *
 * Not a constant: each row renders only if a plan of that kind covers the book,
 * and there is no yearly that covers every book — `all` is sold outright. A
 * book on no track but `all` therefore has one row, and defaulting to the kind
 * that is not there would select nothing.
 *
 * Safe to read at setup: both plans come from the compiled-in catalogue and the
 * awaited book list, so the server and the client start from the same one.
 */
const defaultPlanId = computed<string>(() => (yearly.value ?? lifetime.value)?.plan ?? '')

const chosen = ref<string>(defaultPlanId.value)

const selected = computed<PaywallPlan | undefined>(
  () => plans.value.find(p => p.id === chosen.value) ?? plans.value[0],
)

/* ---------- the radio group ---------- */

/** By index, set from a function ref: a `v-for` ref array is not promised to
 *  keep the list's order, and focus has to land on the row that was chosen. */
const radios: (HTMLButtonElement | null)[] = []

function keepRadio(at: number, el: unknown): void {
  radios[at] = el instanceof HTMLButtonElement ? el : null
}

/**
 * Arrow keys move the selection, as native radios do: one tab stop for the
 * group (the checked row), and the arrows both select and focus.
 */
function onKey(event: KeyboardEvent, at: number): void {
  const count = plans.value.length
  const moves: Record<string, number> = {
    ArrowDown: at + 1,
    ArrowRight: at + 1,
    ArrowUp: at - 1,
    ArrowLeft: at - 1,
    Home: 0,
    End: count - 1,
  }
  const to = moves[event.key]

  if (to === undefined || !count) return
  event.preventDefault()

  const wrapped = (to + count) % count
  const plan = plans.value[wrapped]
  if (!plan) return

  chosen.value = plan.id
  radios[wrapped]?.focus()
}

/* ---------- who is reading ---------- */

const { isSignedIn } = useReader()

// `isSignedIn` reads a cookie hint on the client and is always false on the
// server, so anything rendered from it must wait for mount — otherwise a
// signed-in reader hydrates against markup that says the opposite. Same guard
// as the lesson page's `hydrated`.
const hydrated = ref(false)
onMounted(() => {
  hydrated.value = true
})

const route = useRoute()

/** Back to this lesson after signing in: they were reading. */
const signIn = computed<string>(() => `/login?redirect=${encodeURIComponent(route.fullPath)}`)

const ids = { title: useId(), plans: useId() }

// Served from public/, so it ships in the image rather than off the CDN. The
// same painting as the pricing section.
const IMAGE = '/pricing-lighthouse.jpg'
</script>

<template>
  <div class="wall">
    <!-- The last free paragraph fades into the page rather than stopping on a
         line, so the cut reads as more rather than as an ending. -->
    <div class="wall__fade" aria-hidden="true" />

    <section class="wall__card" :aria-labelledby="ids.title">
      <header class="wall__head">
        <div class="wall__plate">
          <img
            :src="IMAGE"
            alt=""
            width="1200"
            height="2135"
            loading="lazy"
            decoding="async"
          >
        </div>

        <div class="wall__words">
          <p class="wall__eyebrow">
            {{ book.title.toLowerCase() }} · chapter {{ chapter.number }}
          </p>
          <h2 :id="ids.title" class="wall__title">The rest of this chapter is for members</h2>
          <!-- The book, not a language: nothing in the catalogue says which
               language a book is written in, and most of them are not about
               one. -->
          <p class="wall__body">
            You have read the setup. The part that changes how you think about {{ book.title }} is below.
          </p>
        </div>
      </header>

      <div v-if="total" class="wall__progress">
        <p class="wall__counts">
          <span>{{ readCount }} of {{ total }} sections read</span>
          <span>{{ minutesLeft }} min left, members only</span>
        </p>

        <ol class="wall__route" :class="{ 'wall__route--dense': dense }" aria-label="Chapter progress">
          <li
            v-for="(section, at) in sections"
            :key="section.n"
            class="wall__stop"
            :class="[`wall__stop--${stateOf(section)}`, { 'wall__stop--after-current': at === afterCurrent }]"
          >
            <span class="wall__track">
              <span class="wall__dot" aria-hidden="true" />
              <span
                v-if="at < sections.length - 1"
                class="wall__line"
                :class="{ 'wall__line--locked': sections[at + 1]?.locked }"
                aria-hidden="true"
              />
            </span>
            <span class="wall__label">
              {{ section.n }}<template v-if="stateOf(section) === 'current'"> · here</template>
            </span>
            <span class="lh-sr">
              {{ section.title }}, {{ stateOf(section) === 'locked' ? 'members only' : stateOf(section) === 'current' ? 'where you are' : 'read' }}
            </span>
          </li>
        </ol>
      </div>

      <ol v-if="locked.length || book.moreLessons > 0" class="wall__sections">
        <li v-for="section in lockedShown" :key="section.n" class="wall__section">
          <span class="wall__n">{{ section.n }}</span>
          <span class="wall__name">{{ plain(section.title) }}</span>
          <span class="wall__minutes">{{ section.minutes }} min</span>
          <p v-if="section.peek" class="wall__peek">{{ section.peek }}</p>
        </li>
        <li v-if="book.moreLessons > 0">
          <NuxtLink :to="`/books/${book.slug}#toc`" class="wall__section wall__more">
            <span class="wall__n" aria-hidden="true">…</span>
            <span class="wall__name">
              {{ moreLessonsWords }}<template v-if="book.inProgress">, with more still being written</template>
            </span>
            <span class="wall__see">see contents →</span>
          </NuxtLink>
        </li>
      </ol>

      <div v-if="plans.length" class="wall__unlock">
        <p :id="ids.plans" class="wall__eyebrow">unlock it</p>

        <div class="wall__plans" role="radiogroup" :aria-labelledby="ids.plans">
          <button
            v-for="(plan, at) in plans"
            :key="plan.id"
            :ref="el => keepRadio(at, el)"
            type="button"
            role="radio"
            class="wall__plan"
            :class="{ 'wall__plan--on': plan.id === selected?.id }"
            :aria-checked="plan.id === selected?.id"
            :tabindex="plan.id === selected?.id ? 0 : -1"
            @click="chosen = plan.id"
            @keydown="onKey($event, at)"
          >
            <span class="wall__radio" aria-hidden="true" />
            <span class="wall__plan-text">
              <span class="wall__plan-name">{{ plan.name }}</span>
              <span class="wall__plan-note">{{ plan.note }}</span>
              <span v-if="plan.discount" class="wall__discount">
                <span class="wall__off">{{ plan.discount.amount }} off with</span>
                <span class="wall__code">{{ plan.discount.code }}</span>
              </span>
            </span>
            <span class="wall__price">
              <s v-if="plan.listPrice" class="wall__list">{{ plan.listPrice }}</s>
              <span class="wall__amount">{{ plan.price }}</span>
              <span class="wall__per">{{ plan.per }}</span>
            </span>
          </button>
        </div>
      </div>

      <div class="wall__actions">
        <div class="wall__links">
          <!-- Only to somebody who is not signed in — after mount, so the
               server and the first client render agree. -->
          <p v-if="!(hydrated && isSignedIn)" class="wall__member">
            already a member? <NuxtLink :to="signIn">sign in</NuxtLink>
          </p>
          <NuxtLink to="/pricing" class="wall__compare">compare plans</NuxtLink>
        </div>

        <NuxtLink v-if="selected" :to="selected.checkoutUrl" class="wall__cta">
          <FlameIcon />
          {{ selected.cta }}
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<style scoped>
/*
 * Every rule starts at `.wall`. The card is teleported into the lesson body,
 * where `.lesson-content[data-lesson-content] h2` and friends reach it; a
 * scoped `.wall__title` alone loses to those on specificity.
 */

.wall {
  position: relative;
}

/* Over the bottom 110px of the paragraph above, and the margin under it, from
   nothing to the page's own ground — in either theme. */
.wall .wall__fade {
  pointer-events: none;
  position: absolute;
  right: 0;
  bottom: 100%;
  left: 0;
  height: calc(110px + var(--space-6));
  background: linear-gradient(to bottom, transparent, var(--surface-page));
}

/* Dark in both themes, as the home pricing section is. The tokens are
   re-pointed here so everything inside reads them as it would on dark. */
.wall .wall__card {
  --ink: #f2f2f2;
  --ink-secondary: #c9c9c9;
  --ink-muted: #8a8a8a;
  --border: #3a3a3a;
  --border-strong: rgba(255, 255, 255, 0.18);
  --surface-sunken: rgba(255, 255, 255, 0.04);
  --accent-strong: #7dc0ff;

  position: relative;
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding: 40px;
  border: 0;
  border-radius: 16px;
  background: #202020;
  color: var(--ink);
}

/* No italics anywhere in the card — the lesson body turns `em` italic again,
   and the card sits inside it. */
.wall .wall__card,
.wall .wall__card * {
  font-style: normal;
}

/* The prose's own spacing and list markers, off. `:where` so this adds no
   weight of its own and every rule below still wins over it. */
.wall .wall__card :where(p, ol, li, h2) {
  margin: 0;
  padding: 0;
}

.wall .wall__card ol {
  list-style: none;
}

.wall .wall__card a {
  text-decoration: none;
}

/* ---------- header ---------- */

.wall .wall__head {
  display: grid;
  grid-template-columns: 132px 1fr;
  gap: 28px;
  align-items: center;
}

.wall .wall__plate {
  padding: 5px;
  border: 1px solid var(--border);
  border-radius: 12px;
  transform: rotate(-2deg);
}

.wall .wall__plate img {
  display: block;
  width: 100%;
  height: auto;
  margin: 0;
  aspect-ratio: 3 / 4;
  border-radius: 7px;
  object-fit: cover;
  object-position: 50% 40%;
}

.wall .wall__words {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.wall .wall__eyebrow {
  font: 400 12px/18px var(--font-mono);
  color: var(--ink-muted);
}

.wall .wall__title {
  font: 400 36px/42px var(--font-serif);
  letter-spacing: -0.01em;
  color: var(--ink);
  text-wrap: balance;
}

.wall .wall__body {
  font: 400 17px/26px var(--font-serif);
  color: var(--ink-secondary);
  text-wrap: pretty;
}

/* ---------- progress route ---------- */

.wall .wall__progress {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.wall .wall__counts {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 4px 16px;
  font: 400 12px/18px var(--font-mono);
  color: var(--ink-secondary);
}

.wall .wall__route {
  display: flex;
}

.wall .wall__stop {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.wall .wall__stop:last-child {
  flex: 0 0 auto;
}

.wall .wall__track {
  display: flex;
  align-items: center;
}

.wall .wall__dot {
  flex: none;
  box-sizing: border-box;
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.wall .wall__line {
  flex: 1;
  margin: 0 6px;
  border-top: 1px solid var(--ink-muted);
}

.wall .wall__line--locked {
  border-top: 1px dashed var(--border-strong);
}

.wall .wall__label {
  font: 400 11px/16px var(--font-mono);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.wall .wall__stop--read .wall__dot {
  background: var(--ink);
}

.wall .wall__stop--read .wall__label {
  color: var(--ink-secondary);
}

.wall .wall__stop--current .wall__dot {
  background: var(--accent);
  box-shadow: 0 0 0 4px rgba(90, 171, 243, 0.22);
}

.wall .wall__stop--current .wall__label {
  color: var(--accent-strong);
}

.wall .wall__stop--locked .wall__dot {
  border: 1px solid var(--border-strong);
  background: transparent;
}

.wall .wall__stop--locked .wall__label {
  color: var(--ink-muted);
}

/* Crowded: keep the label that says where the reader is, drop the ones that
   say where they have been, and the one "· here" would run into. Hidden, not
   removed, so the row keeps its height. */
.wall .wall__route--dense :is(.wall__stop--read, .wall__stop--after-current) .wall__label {
  visibility: hidden;
}

/* ---------- locked sections ---------- */

.wall .wall__sections {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.wall .wall__section {
  display: grid;
  grid-template-columns: 48px 1fr auto;
  gap: 4px 0;
  align-items: baseline;
  padding: 16px 20px;
  border-radius: 12px;
  background: var(--surface-sunken);
}

.wall .wall__n,
.wall .wall__minutes {
  font: 400 12px/24px var(--font-mono);
  font-variant-numeric: tabular-nums;
  color: var(--ink-muted);
}

.wall .wall__minutes {
  padding-left: 16px;
  white-space: nowrap;
}

.wall .wall__name {
  min-width: 0;
  font: 500 16px/24px var(--font-sans);
  color: var(--ink);
}

/* The rest of the book: an outline rather than a filled row, so it reads as a
   summary and not as one more section. */
.wall .wall__more {
  border: 1px dashed var(--border-strong);
  background: transparent;
  color: var(--ink);
  transition: background-color 0.2s ease-out;
}

.wall .wall__more:hover {
  background: var(--surface-sunken);
}

.wall .wall__more .wall__name {
  color: var(--ink-secondary);
}

.wall .wall__peek {
  grid-column: 2 / 4;
  font: 400 15px/24px var(--font-serif);
  color: var(--ink-muted);
}


.wall .wall__see {
  padding-left: 16px;
  font: 400 12px/18px var(--font-mono);
  color: var(--ink-secondary);
  white-space: nowrap;
}

/* ---------- plan picker ---------- */

.wall .wall__unlock {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.wall .wall__plans {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.wall .wall__plan {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 16px;
  align-items: center;
  width: 100%;
  padding: 16px 20px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: transparent;
  color: var(--ink);
  text-align: left;
  cursor: pointer;
  transition: border-color 0.2s ease-out;
}

.wall .wall__plan--on {
  border-color: var(--ink);
}

.wall .wall__plan:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}

.wall .wall__radio {
  box-sizing: border-box;
  width: 16px;
  height: 16px;
  border: 1px solid var(--border-strong);
  border-radius: 50%;
  transition: border-color 0.2s ease-out, border-width 0.2s ease-out;
}

/* A thick ring in `--ink` reads as filled. */
.wall .wall__plan--on .wall__radio {
  border: 5px solid var(--ink);
}

.wall .wall__plan-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.wall .wall__plan-name {
  font: 500 16px/22px var(--font-sans);
  color: var(--ink);
}

.wall .wall__plan-note {
  font: var(--text-caption);
  color: var(--ink-secondary);
}

/* Two pieces that each refuse to break, so at worst the chip moves to its own
   line whole — it never wraps inside itself. */
.wall .wall__discount {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 8px;
  margin-top: 6px;
  font: 400 12px/18px var(--font-mono);
  color: var(--ink-secondary);
}

.wall .wall__off,
.wall .wall__code {
  flex: none;
  white-space: nowrap;
}

.wall .wall__code {
  padding: 1px 8px;
  border: 1px dashed var(--border-strong);
  border-radius: 999px;
  color: var(--ink);
  letter-spacing: 0.04em;
}

.wall .wall__price {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

.wall .wall__list {
  font: var(--text-caption);
  color: var(--ink-muted);
  text-decoration: line-through;
}

.wall .wall__amount {
  font: 500 24px/28px var(--font-sans);
  letter-spacing: -0.015em;
  color: var(--ink);
}

.wall .wall__per {
  font: 400 12px/18px var(--font-mono);
  color: var(--ink-muted);
}

/* ---------- actions ---------- */

.wall .wall__actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 16px;
  padding-top: 24px;
  border-top: 1px dashed var(--border);
}

.wall .wall__links {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.wall .wall__member {
  font: var(--text-caption);
  color: var(--ink-secondary);
}

.wall .wall__member a {
  color: var(--accent-strong);
}

.wall .wall__compare {
  font: var(--text-caption);
  color: var(--ink-secondary);
  transition: color 0.2s ease-out;
}

.wall .wall__compare:hover {
  color: var(--ink);
}

/* The house Get Pro yellow, named in the spec. */
.wall .wall__cta {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 44px;
  margin-left: auto;
  padding: 0 18px;
  border: 0;
  border-radius: 6px;
  background: #f2a41f;
  color: #111111;
  font: 400 15px/1 var(--font-serif);
  white-space: nowrap;
  transition: opacity 0.2s ease-out;
}

.wall .wall__cta:hover {
  opacity: 0.9;
  color: #111111;
}

/* The flame draws in its own orange, which on this yellow is invisible. The
   stroke follows the text and the glow goes; `!important` because the flicker
   animates both and an animation outranks a rule — the motion itself still
   plays, and `FlameIcon` stops it under reduced motion. */
.wall .wall__cta :deep(.flame) {
  stroke: currentColor !important;
  filter: none !important;
}

/* ---------- narrow ---------- */

@media (max-width: 620px) {
  .wall .wall__card {
    padding: 24px;
  }

  .wall .wall__head {
    grid-template-columns: 1fr;
  }

  .wall .wall__plate {
    width: 112px;
  }

  /* Too narrow for a number under every stop: the one that says where the
     reader is stays, and the list below numbers the rest. */
  .wall .wall__stop:not(.wall__stop--current) .wall__label {
    visibility: hidden;
  }

  .wall .wall__plan {
    grid-template-columns: auto 1fr;
  }

  .wall .wall__price {
    grid-column: 2;
    align-items: flex-start;
  }
}
</style>
