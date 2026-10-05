import { describe, expect, it } from 'vitest'

import { afterDiscount, money } from './Money'

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
