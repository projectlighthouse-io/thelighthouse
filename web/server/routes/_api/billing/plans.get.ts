import { fromApi } from '#server/utils/Lighthouse'

/** One purchasable plan, as the api reports it. */
interface ApiPlan {
  plan: string
  track: string
  /** Book slugs the plan unlocks, for the pricing card's list. */
  books?: string[]
  /** Every book, including ones published after purchase. */
  everything?: boolean
  recurring: boolean
  amount: number | null
  currency: string | null
}

/**
 * What is for sale, and what it costs.
 *
 * Through nitro rather than straight from the browser, like the other public
 * reads: the amounts are the same for everyone, so this can be fetched during
 * SSR and prerendered into the pricing page instead of appearing a moment
 * after hydration.
 *
 * The reader-specific half of billing — a membership, a checkout, a
 * cancellation — deliberately does *not* come through here. Those carry the
 * session cookie and go to the api directly, the way `/api/notes` does; a
 * nitro proxy in front of them would have to forward credentials, and there is
 * nothing to gain by it.
 *
 * **The amounts are not computed here.** They come from the declaration
 * `lighthouse-prices` reconciles against stripe, so this handler translates
 * naming and nothing else.
 */
export default defineEventHandler(async () => {
  const plans = await fromApi<ApiPlan[]>('/api/billing/plans')

  return plans.map(plan => ({
    plan: plan.plan,
    track: plan.track,
    books: plan.books ?? [],
    everything: plan.everything ?? false,
    recurring: plan.recurring,
    amount: plan.amount,
    currency: plan.currency,
  }))
})
