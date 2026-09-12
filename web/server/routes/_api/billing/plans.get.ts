import { fromApi } from '#server/utils/Lighthouse'

/** One purchasable plan, as the api reports it. */
interface ApiPlan {
  plan: string
  track: string
  recurring: boolean
  amount: number | null
  currency: string | null
}

/**
 * What is for sale, what it costs, and where the asker is.
 *
 * Through nitro rather than straight from the browser, like the other public
 * reads, so the pricing page carries the amounts in its html rather than
 * showing them a moment after hydration.
 *
 * **`no-store`, and that is not an oversight.** The answer names the caller's
 * country, so a shared cache holding one country's answer would serve it to
 * the next country along. It used to be edge-cacheable and is not any more.
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
/** The discount a country is offered, if any. Advertised, not applied — the
 *  reader types `code` at stripe. */
interface Coupon {
  code: string
  percent: number
}

interface ApiCatalogue {
  country: string | null
  plans: ApiPlan[]
  coupon: Coupon | null
}

export default defineEventHandler(async (event) => {
  setHeader(event, 'cache-control', 'private, no-store')

  // Cloudflare's header, forwarded by hand. Nothing reaches the api over
  // loopback unless this handler passes it on, and without it every request
  // looks to the api as though it came from nowhere.
  const country = getHeader(event, 'cf-ipcountry')

  const answer = await fromApi<ApiCatalogue>(
    '/api/billing/plans',
    {},
    country ? { 'cf-ipcountry': country } : {},
  )

  return {
    country: answer.country,
    plans: answer.plans.map(plan => ({
      plan: plan.plan,
      track: plan.track,
      recurring: plan.recurring,
      amount: plan.amount,
      currency: plan.currency,
    })),
    coupon: answer.coupon,
  }
})
