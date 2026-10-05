/**
 * What every "Get Pro" button outside the pricing frame does.
 *
 * Goes to `/checkout` for the one plan on sale — the pricing frame's own
 * buttons pick between plans, these do not. That page signs a reader in first
 * if they need it and comes back with the plan intact, then hands them to
 * stripe, or to their billing page when they already pay.
 */

/** The plan "Get Pro" means: the $99 yearly foundation track. One place to
 *  change it when there is more than one thing to sell. */
export const PRO_PLAN = 'foundation_yearly'

/** Where a button that buys `plan` sends the reader. */
export function checkoutUrl(plan: string): string {
  return `/checkout?plan=${encodeURIComponent(plan)}`
}

export function useGetPro() {
  async function getPro(): Promise<void> {
    await navigateTo(checkoutUrl(PRO_PLAN))
  }

  return { getPro }
}
