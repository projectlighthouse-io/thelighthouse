/**
 * The things for sale.
 *
 * A track is a bundle of books, and it is the only unit sold — there is no
 * per-book purchase and no all-you-can-eat subscription any more.
 *
 * **Only the words are here.** Which books a track contains comes from the api,
 * derived from the `tracks:` map in each `book.yaml` — the same source rust
 * grants entitlements from, so the page and the grant cannot disagree. This
 * file used to list the slugs too, and said itself that it was the copy that
 * would be wrong.
 *
 * The amounts are deliberately absent: they come from `/api/billing/plans`,
 * which reads the config `lighthouse-prices` generates from the declaration
 * stripe was reconciled against. One number, one place.
 *
 * **A track appears here only once it has a price and books.** Go and Rust were
 * retired — their books are sold through Everything — and Containers, Platform
 * Engineering and Architect are not written yet. Listing one early renders a
 * card with no amount and a button that cannot be pressed, which is what this
 * file did before. The api refuses to boot on the other half of that mistake: a
 * declared plan selling a track no book is on.
 */

export interface Track {
  /** Matches the plan-id prefix and the ohara track name. */
  key: 'foundation' | 'all'
  name: string
  blurb: string
  featured: boolean
}

export const tracks: Track[] = [
  {
    key: 'foundation',
    name: 'Foundations',
    blurb:
      'the three every other track is built on — how a machine runs a program, how it talks to another one, and how to hold data so the answer arrives in time.',
    featured: false,
  },
  {
    key: 'all',
    name: 'Everything',
    blurb:
      'every book on every track, including the ones on none — Go, Rust, C, the container runtime and interview preparation. everything published, and everything published next.',
    featured: true,
  },
]
