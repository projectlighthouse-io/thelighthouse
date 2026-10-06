import { describe, expect, it } from 'vitest'

import type { CataloguePlan } from '@/data/Catalogue'
import type { PaywallSection } from '@/types/Paywall'
import { cheapestPlan, lessonProgress, moreChaptersWords, pricePlan, stopState } from './Paywall'

function section(n: string, minutes: number, locked: boolean): PaywallSection {
  return { n, title: `section ${n}`, minutes, peek: locked ? 'the opening…' : '', locked }
}

const sections = [
  section('04.1', 3, false),
  section('04.2', 4, false),
  section('04.3', 2, false),
  section('04.4', 6, true),
  section('04.5', 5, true),
]

const yearly: CataloguePlan = { plan: 'foundation_yearly', track: 'foundation', recurring: true, amount: 11900, currency: 'usd' }
const lifetime: CataloguePlan = { plan: 'all_lifetime', track: 'all', recurring: false, amount: 24900, currency: 'usd' }

describe('lessonProgress', () => {
  it('counts what was read and the minutes still locked', () => {
    expect(lessonProgress(sections)).toEqual({ readCount: 3, total: 5, minutesLeft: 11 })
  })

  it('follows the data rather than a fixed shape', () => {
    expect(lessonProgress(sections.slice(2))).toEqual({ readCount: 1, total: 3, minutesLeft: 11 })
    expect(lessonProgress([])).toEqual({ readCount: 0, total: 0, minutesLeft: 0 })
  })
})

describe('stopState', () => {
  it('marks the current section, the read ones before it and the locked ones', () => {
    expect(sections.map(s => stopState(s, '04.3'))).toEqual(['read', 'read', 'current', 'locked', 'locked'])
  })

  it('has no current stop when nothing free was reached', () => {
    expect(sections.map(s => stopState(s, ''))).not.toContain('current')
  })
})

describe('moreChaptersWords', () => {
  it('names the count and the book, singular at one', () => {
    expect(moreChaptersWords(3, 'Networking')).toBe('3 more chapters of Networking')
    expect(moreChaptersWords(1, 'Networking')).toBe('1 more chapter of Networking')
  })
})

describe('cheapestPlan', () => {
  const pricier: CataloguePlan = { ...yearly, plan: 'pricier_yearly', amount: 19900 }
  const offTrack: CataloguePlan = { ...yearly, plan: 'go_yearly', track: 'go', amount: 100 }
  const catalogue = [pricier, offTrack, yearly, lifetime]

  it('picks the cheapest plan of a kind on a track that carries the book', () => {
    expect(cheapestPlan(catalogue, ['foundation', 'all'], true)?.plan).toBe('foundation_yearly')
    expect(cheapestPlan(catalogue, ['foundation', 'all'], false)?.plan).toBe('all_lifetime')
  })

  it('offers nothing of a kind no carrying track sells', () => {
    expect(cheapestPlan(catalogue, ['all'], true)).toBeUndefined()
  })
})

describe('pricePlan', () => {
  it('shows the list price alone when there is no coupon', () => {
    const plan = pricePlan(lifetime, null, 'Everything', '/checkout?plan=all_lifetime')

    expect(plan.price).toBe('$249')
    expect(plan.listPrice).toBeUndefined()
    expect(plan.discount).toBeUndefined()
    expect(plan.per).toBe('once')
    expect(plan.cta).toBe('Get lifetime access')
    expect(plan.checkoutUrl).toBe('/checkout?plan=all_lifetime')
  })

  it('strikes the list price and names its own code under a coupon', () => {
    const plan = pricePlan(yearly, { code: 'PPP40', percent: 40 }, 'Foundations', '/checkout?plan=foundation_yearly')

    expect(plan.price).toBe('$71.40')
    expect(plan.listPrice).toBe('$119')
    expect(plan.discount).toEqual({ amount: '40%', code: 'PPP40' })
    expect(plan.per).toBe('/ year')
  })

  it('says a fixed discount as money', () => {
    const plan = pricePlan(lifetime, { code: 'TAKE70', amount_off: 7000 }, 'Everything', '/c')

    expect(plan.price).toBe('$179')
    expect(plan.discount).toEqual({ amount: '$70', code: 'TAKE70' })
  })

  it('keeps each plan to its own coupon', () => {
    const a = pricePlan(yearly, { code: 'YEAR10', percent: 10 }, 'Foundations', '/a')
    const b = pricePlan(lifetime, { code: 'LIFE20', percent: 20 }, 'Everything', '/b')

    expect([a.discount?.code, b.discount?.code]).toEqual(['YEAR10', 'LIFE20'])
    expect([a.cta, b.cta]).toEqual(['Get yearly access', 'Get lifetime access'])
  })

  it('takes the catalogue button text over the default', () => {
    expect(pricePlan({ ...lifetime, button_text: 'Own it all' }, null, 'Everything', '/c').cta).toBe('Own it all')
  })
})
