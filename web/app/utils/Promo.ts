import type { PppOffer } from '@/types/Content'
import { offAmount } from './Money'

/** Everything the banner says, or nothing to say. */
export interface Promo {
  country: string
  amount: string
  code: string
  /** Stable for one plan's one code: dismissing it hides that code, and a new
   *  code is a new promo that shows again. */
  promoId: string
}

/** `BD` → `Bangladesh`; empty when there is no country or no name for it. */
function regionName(code: string | null): string {
  if (!code) return ''

  try {
    return new Intl.DisplayNames(['en'], { type: 'region' }).of(code) ?? ''
  }
  catch {
    return ''
  }
}

/**
 * The promo for `plan` in this offer — or null when the country, the amount or
 * the code is missing, because the banner says all three or nothing.
 */
export function promoFrom(offer: PppOffer | null, plan: string): Promo | null {
  const lead = offer?.offers.find(o => o.plan === plan)
  const country = regionName(offer?.country ?? null)
  if (!lead || !lead.code || !country) return null

  const amount = lead.amount_off !== undefined || lead.percent ? offAmount(lead) : ''
  if (!amount) return null

  return { country, amount, code: lead.code, promoId: `${lead.plan}:${lead.code}` }
}
