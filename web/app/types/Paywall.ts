/**
 * What the chapter paywall card is given.
 *
 * Everything countable on the card is derived from these at render time —
 * how many sections were read, how many minutes are left — rather than sent
 * alongside them, so the numbers cannot disagree with the list they count.
 */

/** One `##` section of the lesson, as the card shows it. */
export interface PaywallSection {
  /** `04.3` */
  n: string
  title: string
  minutes: number
  /** A locked section's opening, cut short by the api. Empty on an open one. */
  peek: string
  locked: boolean
}

export interface PaywallBook {
  title: string
  slug: string
  /** Chapters after this one. Zero hides the "and after this chapter" row. */
  moreChapters: number
  /** Whether more of the book is still being written. */
  inProgress: boolean
}

/** One plan on the picker, already priced for this reader. */
export interface PaywallPlan {
  id: string
  name: string
  note: string
  /** What the reader pays: the discounted price when a coupon applies. */
  price: string
  /** The undiscounted price, only when a discount brings it down. */
  listPrice?: string
  per: string
  discount?: { amount: string, code: string }
  cta: string
  checkoutUrl: string
}
