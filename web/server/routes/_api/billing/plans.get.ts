import { fromApi } from '#server/utils/Lighthouse'

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
 * `lighthouse-prices` reconciles against stripe, and this handler passes the
 * api's answer through untouched.
 */
export default defineEventHandler(async (event) => {
  setHeader(event, 'cache-control', 'private, no-store')

  // Cloudflare's header, forwarded by hand. Nothing reaches the api over
  // loopback unless this handler passes it on, and without it every request
  // looks to the api as though it came from nowhere.
  const country = getHeader(event, 'cf-ipcountry')

  // Passed through whole. It used to be reshaped field by field, which meant
  // every field the api grew had to be added here as well — and one was not:
  // the flag saying whether a plan was discounted was dropped on the way
  // through, so no reduced price ever rendered. The pages declare the shape
  // they read; this only forwards the country header.
  return await fromApi('/api/billing/plans', {}, country ? { 'cf-ipcountry': country } : {})
})
