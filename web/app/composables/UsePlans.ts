import type { Off } from '@/utils/Money'

/**
 * A plan's discount for the asker's country: the code, how much it takes off
 * (`percent` or `amount_off` in minor units), and whether it is the plan's
 * rest tier — the price for everywhere no other tier names. Applied for the
 * reader at checkout; the page shows it so the price they see is the one they
 * pay.
 */
export interface Coupon extends Off {
  code: string
  rest: boolean
}

/** One purchasable plan, as `/_api/billing/plans` reports it. */
export interface Offer {
  plan: string
  track: string
  books: string[]
  everything: boolean
  recurring: boolean
  amount: number | null
  currency: string | null
  /** The discount that comes off this plan, for the asker's country. */
  coupon: Coupon | null
}

/**
 * What the api offers this reader, plan by plan — read for the coupons.
 *
 * The amounts on the page come from the compiled `Catalogue.ts`; only the
 * purchasing-power coupon depends on who is asking, and that is known only in
 * the browser. `server: false` is load-bearing: `/pricing` is prerendered, and
 * a server-side read there runs once at build time with no country and no api.
 * Allowed to fail — no coupon is the full price.
 */
export function usePlans() {
  return useAsyncData('billing-coupons', () =>
    $fetch<{ plans: Offer[] }>('/_api/billing/plans')
      .then(response => response.plans)
      .catch(() => [] as Offer[]), { server: false, default: () => [] as Offer[] })
}
