import { describe, expect, it } from 'vitest'

import type { PppOffer } from '@/types/Content'
import { promoFrom } from './Promo'

const PLAN = 'foundation_yearly'

function offer(country: string | null, coupon: Partial<PppOffer['offers'][number]>): PppOffer {
  return { country, offers: [{ plan: PLAN, code: 'FIRSTLIGHT', rest: false, ...coupon }] }
}

describe('promoFrom', () => {
  it('names the country, formats the amount and keys the promo by plan and code', () => {
    expect(promoFrom(offer('BD', { amount_off: 2000 }), PLAN)).toEqual({
      country: 'Bangladesh',
      amount: '$20',
      code: 'FIRSTLIGHT',
      promoId: 'foundation_yearly:FIRSTLIGHT',
    })
  })

  it('says a percentage as one', () => {
    expect(promoFrom(offer('IN', { percent: 40 }), PLAN)?.amount).toBe('40%')
  })

  it('follows the data', () => {
    const promo = promoFrom(offer('NG', { code: 'NEWDAWN', amount_off: 3500 }), PLAN)

    expect(promo).toMatchObject({ country: 'Nigeria', amount: '$35', code: 'NEWDAWN' })
  })

  it('is nothing when any of the three is missing', () => {
    expect(promoFrom(null, PLAN)).toBeNull()
    expect(promoFrom(offer(null, { amount_off: 2000 }), PLAN)).toBeNull()
    expect(promoFrom(offer('BD', { code: '', amount_off: 2000 }), PLAN)).toBeNull()
    expect(promoFrom(offer('BD', {}), PLAN)).toBeNull()
    expect(promoFrom(offer('BD', { amount_off: 2000 }), 'all_lifetime')).toBeNull()
  })
})
