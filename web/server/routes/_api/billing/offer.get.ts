import type { PppOffer } from '@/types/Content'
import { fromApi } from '#server/utils/Lighthouse'

/** One plan as the api reports it, narrowed to what the banner reads. */
interface ApiPlanCoupon {
  coupon: { rest: boolean } | null
}

/**
 * Whether the asker's country has prices of its own, for the banner.
 *
 * The rust api already decides this — `/api/billing/plans` carries a coupon per
 * plan for the caller's `cf-ipcountry`. Only a tier that *names* the country
 * counts: the rest tier is the price everywhere else, and a banner saying so to
 * every visitor on every page would be noise. Nothing to advertise either —
 * checkout applies the tier — so the answer is just the country.
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

  const { plans } = await fromApi<{ plans: ApiPlanCoupon[] }>('/api/billing/plans', undefined, {
    'cf-ipcountry': country,
  }).catch(() => ({ plans: [] as ApiPlanCoupon[] }))

  return plans.some(plan => plan.coupon && !plan.coupon.rest) ? { country } : null
})
