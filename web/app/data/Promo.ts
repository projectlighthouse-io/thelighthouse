/**
 * The regional promo banner's words, in one place. The country, the amount and
 * the code are never here — they come from `/_api/billing/offer` per reader.
 *
 * Reads: "{lead} **{amount} off** for {country}, with `{code}`" — no full stop
 * after the code.
 */
export const PROMO_COPY = {
  lead: 'Good engineers are everywhere, and so are these books.',
  off: 'off',
  for: 'for',
  with: 'with',
  copied: 'copied',
} as const
