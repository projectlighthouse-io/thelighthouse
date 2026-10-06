import type { CataloguePlan } from '@/data/Catalogue'
import type { PaywallPlan, PaywallSection } from '@/types/Paywall'
import type { Off } from './Money'
import { afterOff, money } from './Money'

/**
 * What the paywall card counts and prices, apart from the card.
 *
 * Every number the card shows comes out of here, from data it was given, so a
 * change to the lesson, the book or the catalogue changes the card — and these
 * are the functions the tests hold to that.
 */

export type StopState = 'read' | 'current' | 'locked'

/** How far through the lesson the reader is, counted from its sections. */
export function lessonProgress(sections: PaywallSection[]): { readCount: number, total: number, minutesLeft: number } {
  return {
    readCount: sections.filter(s => !s.locked).length,
    total: sections.length,
    minutesLeft: sections.filter(s => s.locked).reduce((sum, s) => sum + s.minutes, 0),
  }
}

export function stopState(section: PaywallSection, current: string): StopState {
  if (section.locked) return 'locked'

  return section.n === current ? 'current' : 'read'
}

/** `3 more chapters of Networking` — singular at one. */
export function moreChaptersWords(count: number, title: string): string {
  return `${count} more ${count === 1 ? 'chapter' : 'chapters'} of ${title}`
}

/** The cheapest plan of a kind whose track carries the book. */
export function cheapestPlan(
  catalogue: CataloguePlan[],
  tracks: string[],
  recurring: boolean,
): CataloguePlan | undefined {
  return catalogue
    .filter(o => o.recurring === recurring && tracks.includes(o.track))
    .sort((a, b) => a.amount - b.amount)[0]
}

/** A coupon as the card needs it: what comes off, and the code to type. */
export interface PlanCoupon extends Off {
  code: string
}

/** `$70` or `30%` — what comes off, without the " off" the card writes itself. */
function discountAmount(coupon: Off): string {
  return coupon.amount_off !== undefined ? money(coupon.amount_off) : `${coupon.percent ?? 0}%`
}

/**
 * One row of the plan picker, priced with the coupon for *this* plan — a
 * stripe coupon is restricted to one plan's product, so two rows can carry
 * different codes. The list price is struck only when the coupon brings it down.
 */
export function pricePlan(
  o: CataloguePlan,
  coupon: PlanCoupon | null,
  trackName: string,
  checkoutUrl: string,
): PaywallPlan {
  const reduced = coupon && o.amount ? afterOff(o.amount, coupon) : null

  return {
    id: o.plan,
    name: o.recurring ? 'Yearly' : 'Lifetime',
    note: o.recurring
      ? `${trackName}, and everything shipped to it while you subscribe`
      : `${trackName}, paid once and yours for good`,
    price: money(reduced ?? o.amount),
    listPrice: reduced !== null ? money(o.amount) : undefined,
    per: o.recurring ? '/ year' : 'once',
    discount: coupon ? { amount: discountAmount(coupon), code: coupon.code } : undefined,
    cta: o.button_text ?? (o.recurring ? 'Get yearly access' : 'Get lifetime access'),
    checkoutUrl,
  }
}
