import type { PppOffer } from '@/types/Content'
import { fromApi } from '#server/utils/Lighthouse'

/** One plan as the api reports it, narrowed to what the banner reads. */
interface ApiPlanCoupon {
  coupon: { code: string, percent: number } | null
}

/**
 * The purchasing-power offer for whoever is asking, if any.
 *
 * The rust api already decides this — `/api/billing/plans` carries a coupon per
 * plan when the caller's `cf-ipcountry` names a tier. It does not echo the
 * country back, so this asks it on the reader's behalf with the same header
 * and keeps the code the api validated. Nothing is computed here: no header,
 * or no coupon on any plan, is no offer.
 *
 * `no-store` for the same reason the api gives — the answer depends on who is
 * asking, and a shared cache would hand one country's code to the next.
 */
export default defineEventHandler(async (event): Promise<PppOffer | null> => {
  setResponseHeader(event, 'cache-control', 'private, no-store')

  const country = getRequestHeader(event, 'cf-ipcountry')?.trim().toUpperCase()

  if (!country || !/^[A-Z]{2}$/.test(country) || country === 'XX' || country === 'T1') {
    return null
  }

  const plans = await fromApi<ApiPlanCoupon[]>('/api/billing/plans', undefined, {
    'cf-ipcountry': country,
  }).catch(() => [] as ApiPlanCoupon[])

  // The deepest discount on offer, so the banner never advertises less than
  // the reader can actually get.
  const best = plans
    .map(plan => plan.coupon)
    .filter((coupon): coupon is NonNullable<typeof coupon> => coupon !== null)
    .sort((a, b) => b.percent - a.percent)[0]

  return best ? { country, code: best.code, percent: best.percent } : null
})
