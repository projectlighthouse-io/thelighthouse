/** One purchasable plan, as `/_api/billing/plans` reports it. */
export interface Offer {
  plan: string
  track: string
  books: string[]
  everything: boolean
  recurring: boolean
  amount: number | null
  currency: string | null
  /** The purchasing-power code that comes off this plan, for the asker's country. */
  coupon: { code: string, percent: number } | null
}

/**
 * What is for sale and what it costs, from rust through nitro.
 *
 * One handler for the one cache key, so the pricing page's structured data
 * and the frame's visible prices are the same answer. Allowed to fail: a
 * build with no api behind it renders the tracks without amounts.
 */
export function usePlans() {
  return useAsyncData('billing-plans', () =>
    $fetch<{ plans: Offer[] }>('/_api/billing/plans')
      .then(response => response.plans)
      .catch(() => [] as Offer[]))
}
