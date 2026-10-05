import type { PppOffer } from '@/types/Content'
import { fromApi } from '#server/utils/Lighthouse'

/** One plan as the api reports it, narrowed to what the banner reads. */
interface ApiPlanCoupon {
  plan: string
  coupon: { code: string, rest: boolean, percent?: number, amount_off?: number } | null
}

/**
 * Whether the banner is switched on: DISCOUNT_BANNER in the environment, on
 * unless it says false, 0, off or no. Read per request, so a restart applies a
 * change with no rebuild — and prerendered pages follow, since the banner asks
 * this route from the browser.
 */
function bannerOn(): boolean {
  const value = process.env.DISCOUNT_BANNER?.trim().toLowerCase()

  return !value || !['false', '0', 'off', 'no'].includes(value)
}

/**
 * The discount the top banner advertises for whoever is asking, if any.
 *
 * The rust api already decides this — `/api/billing/plans` carries each plan's
 * tier for the caller's `cf-ipcountry`, and the everyone-else tier when the
 * country is not named or not known. This asks it on the reader's behalf with
 * the same header. Nothing is computed here: no discount on any plan, or the
 * banner switched off with DISCOUNT_BANNER=false, is no offer.
 *
 * `no-store` for the same reason the api gives — the answer depends on who is
 * asking, and a shared cache would hand one country's prices to the next.
 */
export default defineEventHandler(async (event): Promise<PppOffer | null> => {
  setResponseHeader(event, 'cache-control', 'private, no-store')

  if (!bannerOn()) return null

  const header = getRequestHeader(event, 'cf-ipcountry')?.trim().toUpperCase()
  const country = header && /^[A-Z]{2}$/.test(header) && header !== 'XX' && header !== 'T1'
    ? header
    : null

  const { plans } = await fromApi<{ plans: ApiPlanCoupon[] }>(
    '/api/billing/plans',
    undefined,
    country ? { 'cf-ipcountry': country } : {},
  ).catch(() => ({ plans: [] as ApiPlanCoupon[] }))

  const discounted = plans.filter(plan => plan.coupon)
  if (!discounted.length) return null

  return {
    country,
    offers: discounted.map(plan => ({
      plan: plan.plan,
      code: plan.coupon?.code ?? '',
      rest: plan.coupon?.rest ?? true,
      percent: plan.coupon?.percent,
      amount_off: plan.coupon?.amount_off,
    })),
  }
})
