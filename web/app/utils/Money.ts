/**
 * Prices, as a reader sees them.
 *
 * Display only. Every amount that is actually charged is computed in minor
 * units on the rust side and by stripe, so nothing here can put a rounding
 * error into somebody's card statement — the worst it can do is advertise a
 * number that is a penny out, which is what the rounding note below is about.
 */

/** `4900` reads as `$49`, `3960` as `$39.60`. Whole dollars lose the `.00`. */
export function money(minor: number): string {
  const whole = minor / 100

  return `$${Number.isInteger(whole) ? whole : whole.toFixed(2)}`
}

/**
 * What a percentage off leaves, in minor units.
 *
 * **An advertisement, not the charge.** Stripe holds the coupon and applies it
 * itself at checkout; this exists so the page can show the reduced price before
 * the reader has typed the code. The two agree because both round a
 * `percent_off` of the same minor-unit amount to the nearest minor unit, which
 * is what stripe documents — but stripe is the one that decides.
 *
 * `percent` is clamped rather than trusted. It reaches here from a declaration
 * by way of the api, and the failure of a bad one is a price tag reading
 * `$-40`, which is worse than a price tag reading full price.
 */
export function afterDiscount(minor: number, percent: number): number {
  const off = Math.min(Math.max(percent, 0), 100)

  return Math.round((minor * (100 - off)) / 100)
}

/** How much a tier takes off: a percentage, or minor units. */
export interface Off {
  percent?: number
  amount_off?: number
}

/**
 * What a tier leaves of a price, in minor units — a fixed amount or a
 * percentage, whichever the tier declares. An advertisement, as
 * `afterDiscount` is; stripe applies the same coupon at checkout. Never below
 * zero: a misdeclared amount reads as free rather than as a negative price.
 */
export function afterOff(minor: number, off: Off): number {
  if (off.amount_off !== undefined) return Math.max(minor - off.amount_off, 0)
  if (off.percent !== undefined) return afterDiscount(minor, off.percent)

  return minor
}

/** `$20` or `40%` — what comes off, without saying so. */
export function offAmount(off: Off): string {
  if (off.amount_off !== undefined) return money(off.amount_off)

  return `${off.percent ?? 0}%`
}

/** `$20 off` or `40% off`. */
export function offLabel(off: Off): string {
  return `${offAmount(off)} off`
}
