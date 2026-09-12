/**
 * The four things for sale.
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
 */

export interface Track {
  /** Matches the plan-id prefix and the ohara track name. */
  key: 'foundation' | 'go' | 'rust' | 'all'
  name: string
  blurb: string
  featured: boolean
}

export const tracks: Track[] = [
  {
    key: 'foundation',
    name: 'Foundation',
    blurb:
      'the three every other track is built on — how a machine runs a program, how it talks to another one, and how to hold data so the answer arrives in time.',
    featured: false,
  },
  {
    key: 'go',
    name: 'Go',
    blurb:
      'the Go books, and Foundation underneath them. concurrency and services on top of the systems knowledge that makes them make sense.',
    featured: true,
  },
  {
    key: 'rust',
    name: 'Rust',
    blurb:
      'the Rust books, and Foundation underneath them. ownership and lifetimes are easier to hold when you already know what the machine is doing.',
    featured: false,
  },
  {
    key: 'all',
    name: 'Everything',
    blurb:
      'every book, including the ones on no track — the container runtime, C, and interview preparation. everything published, and everything published next.',
    featured: false,
  },
]
