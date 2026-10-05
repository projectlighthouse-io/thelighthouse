import { describe, expect, it } from 'vitest'

import { afterDiscount, afterOff, offLabel, money } from './Money'

describe('money', () => {
  it('drops the cents from a whole amount and keeps them otherwise', () => {
    expect(money(9900)).toBe('$99')
    expect(money(24900)).toBe('$249')
    expect(money(3960)).toBe('$39.60')
    expect(money(0)).toBe('$0')
  })
})

describe('afterDiscount', () => {
  it('matches the tiers the catalogue actually declares', () => {
    // The two live plans at the declared 60%, which is what /pricing shows a
    // reader in a ppp country. If these change, the page is advertising
    // something stripe will not charge.
    expect(afterDiscount(9900, 60)).toBe(3960)
    expect(afterDiscount(24900, 60)).toBe(9960)
  })

  it('rounds to a whole minor unit rather than fractions of a penny', () => {
    // 3333 * 0.67 is 2233.11 — a price tag cannot hold that, and neither can
    // stripe.
    expect(afterDiscount(3333, 33)).toBe(2233)
    expect(Number.isInteger(afterDiscount(9999, 37))).toBe(true)
  })

  it('leaves the amount alone when nothing is off', () => {
    expect(afterDiscount(9900, 0)).toBe(9900)
  })

  it('refuses to invent a negative price out of a nonsense percentage', () => {
    expect(afterDiscount(9900, 140)).toBe(0)
    expect(afterDiscount(9900, -20)).toBe(9900)
  })
})

describe('afterOff', () => {
  it('takes a fixed amount off the declared list prices exactly', () => {
    // The tiers pricing.yaml declares: $119 and $599 lists.
    expect(afterOff(11900, { amount_off: 2000 })).toBe(9900)
    expect(afterOff(11900, { amount_off: 7000 })).toBe(4900)
    expect(afterOff(59900, { amount_off: 25000 })).toBe(34900)
  })

  it('takes a percentage off as afterDiscount does', () => {
    expect(afterOff(9900, { percent: 60 })).toBe(afterDiscount(9900, 60))
  })

  it('never shows a price below nothing', () => {
    expect(afterOff(4900, { amount_off: 9000 })).toBe(0)
  })

  it('leaves the price alone for a tier that says neither', () => {
    expect(afterOff(11900, {})).toBe(11900)
  })
})

describe('offLabel', () => {
  it('reads as money or as a percentage, whichever the tier is', () => {
    expect(offLabel({ amount_off: 2000 })).toBe('$20 off')
    expect(offLabel({ percent: 40 })).toBe('40% off')
  })
})
