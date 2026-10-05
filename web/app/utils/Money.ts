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
