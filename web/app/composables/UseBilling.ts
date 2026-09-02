/**
 * What the reader has paid for, and how they pay for more.
 *
 * The same shape as `UseReader`: rust owns the truth, this asks. Nothing here is
 * a gate — entitlement is decided in rust against the session cookie, and the
 * paid half of a lesson is a separate request that either answers or does not.
 * What this decides is which button to draw.
 *
 * **A track is what is for sale.** Foundation, Go, Rust, All — each one sold
 * two ways: `<track>_yearly` renews, `<track>_lifetime` is bought outright.
 * Which books a track contains is not known here and never should be; rust
 * reads it from the content.
 */

/** A track sold as a subscription, as the api reports it. */
export interface Membership {
  /** The plan bought, e.g. `rust_yearly`. Its prefix is the track. */
  plan: string
  status: 'active' | 'grace' | 'ended'
  provider: string
  /** ISO 8601, or null. The end of the period already paid for. */
  period_ends_at: string | null
  /** ISO 8601, or null. Set means cancelled but not yet over. */
  cancel_at: string | null
}

/** The api's refusals, which carry a stable code to branch on. */
interface Refusal {
  code: string
  error: string
}

/** The track a plan sells: everything before the last underscore. */
export function trackOf(plan: string): string {
  const cut = plan.lastIndexOf('_')

  return cut === -1 ? plan : plan.slice(0, cut)
}

/**
 * The reason out of a failed request, or a fallback.
 *
 * The api answers `{ code, error }` on every refusal, and `error` is written to
 * be shown. Anything else — a proxy error page, a dropped connection — has no
 * message worth showing, so it gets a generic one rather than a stack trace.
 */
function reasonFor(failure: unknown): string {
  const data = (failure as { data?: Refusal })?.data

  return data?.error ?? 'Something went wrong. Please try again.'
}

export function useBilling() {
  const membership = useState<Membership | null>('billing.membership', () => null)
  // Distinct from `membership === null`, which cannot tell "pays for nothing"
  // from "not asked yet" — the difference between drawing a buy button and
  // drawing nothing at all.
  const resolved = useState<boolean>('billing.resolved', () => false)

  const busy = ref(false)
  const reason = ref<string | null>(null)

  /** Paid up. Grace — a failed payment being retried — deliberately is not. */
  const subscribed = computed(() => membership.value?.status === 'active')

  /** Cancelled, but still inside the period already paid for. */
  const ending = computed(() => membership.value?.cancel_at !== null && membership.value !== null)

  /** The track the current subscription is for, if there is one. */
  const track = computed(() => (membership.value ? trackOf(membership.value.plan) : null))

  async function load(force = false): Promise<void> {
    // Matches `UseReader`: SSR renders every reader as anonymous, so asking here
    // would either leak one reader's state into a shared cache or make the
    // page uncacheable.
    if (import.meta.server) return
    if (resolved.value && !force) return

    try {
      // 204 for a reader who pays for nothing, which $fetch gives back as
      // undefined — hence the `?? null` rather than trusting the body.
      const found = await $fetch<Membership | null>('/api/billing/membership')
      membership.value = found ?? null
    }
    catch {
      // Unreachable is not "pays for nothing", but there is no third state to
      // draw. Rust decides access either way, so the worst case is a buy
      // button shown to somebody who has already bought.
      membership.value = null
    }
    finally {
      resolved.value = true
    }
  }

  /**
   * Start paying for a track, and hand the browser to the provider.
   *
   * Navigates away on success, so nothing after it runs. The api decides from
   * the plan's own interval whether this is a subscription or a purchase —
   * `_yearly` and `_lifetime` go through the same call.
   */
  async function checkout(plan: string): Promise<void> {
    if (busy.value) return

    busy.value = true
    reason.value = null

    try {
      const { url } = await $fetch<{ url: string }>(
        `/api/billing/stripe/checkout`,
        { method: 'POST', headers: csrfHeader(), body: { plan } },
      )

      // A full navigation, not `navigateTo`: this leaves the app for the
      // provider's own page.
      window.location.href = url
    }
    catch (failure) {
      reason.value = reasonFor(failure)
      busy.value = false
    }
  }

  /** Stop a subscription renewing, keeping the period already paid for. */
  async function cancel(immediately = false): Promise<void> {
    await change('cancel', { immediately })
  }

  /** Undo a cancellation that has not taken effect yet. */
  async function resume(): Promise<void> {
    await change('resume')
  }

  /** Move to another track, or to another billing period on the same one. */
  async function swap(plan: string): Promise<void> {
    await change('swap', { plan })
  }

  /**
   * The three that mutate an existing subscription and answer with its new
   * state — or with 204, when the change ended it.
   */
  async function change(action: string, body?: object): Promise<void> {
    if (busy.value) return

    busy.value = true
    reason.value = null

    try {
      const updated = await $fetch<Membership | null>(
        `/api/billing/stripe/${action}`,
        { method: 'POST', headers: csrfHeader(), body: body ?? {} },
      )

      membership.value = updated ?? null
      resolved.value = true
    }
    catch (failure) {
      reason.value = reasonFor(failure)
    }
    finally {
      busy.value = false
    }
  }

  return {
    membership,
    resolved,
    busy,
    reason,
    subscribed,
    ending,
    track,
    load,
    checkout,
    cancel,
    resume,
    swap,
  }
}
