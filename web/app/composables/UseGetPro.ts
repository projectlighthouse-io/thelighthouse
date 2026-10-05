/**
 * What every "Get Pro" button outside the pricing frame does.
 *
 * Straight to stripe checkout for the one plan on sale — the pricing frame's
 * own buttons pick between plans, these do not — and to sign-in first for a
 * reader without a session, who comes back to the page they were on. The same
 * rule `PricingFrame`'s `buy` follows.
 */

/** The plan "Get Pro" means: the $99 yearly foundation track. One place to
 *  change it when there is more than one thing to sell. */
export const PRO_PLAN = 'foundation_yearly'

export function useGetPro() {
  const { checkout, busy, reason } = useBilling()
  const { isSignedIn } = useReader()
  const route = useRoute()

  async function getPro(): Promise<void> {
    if (busy.value) return

    if (!isSignedIn.value) {
      await navigateTo({ path: '/login', query: { redirect: route.fullPath } })

      return
    }

    await checkout(PRO_PLAN)
  }

  return { getPro, busy, reason }
}
