/** One purchasable plan, as `/_api/billing/plans` reports it. */
export interface Offer {
  plan: string
  track: string
  books: string[]
  everything: boolean
  recurring: boolean
  amount: number | null
  currency: string | null
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
    $fetch<Offer[]>('/_api/billing/plans').catch(() => [] as Offer[]))
}
