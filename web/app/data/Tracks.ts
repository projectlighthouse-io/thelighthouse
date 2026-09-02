/**
 * The four things for sale.
 *
 * A track is a bundle of books, and it is the only unit sold — there is no
 * per-book purchase and no all-you-can-eat subscription any more.
 *
 * **The book slugs here are for display only.** What a track actually contains
 * is the `tracks:` map in each book's `book.yaml`, which rust reads when it
 * grants entitlements. If the two ever disagree, this file is the one that is
 * wrong, and the reader still gets what rust says they bought.
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
  /** Book slugs, in reading order, for the card. */
  books: string[]
  featured: boolean
}

export const tracks: Track[] = [
  {
    key: 'foundation',
    name: 'Foundation',
    blurb:
      'the three every other track is built on — how a machine runs a program, how it talks to another one, and how to hold data so the answer arrives in time.',
    books: ['os-fundamentals', 'networking-fundamentals', 'dsa-fundamentals'],
    featured: false,
  },
  {
    key: 'go',
    name: 'Go',
    blurb:
      'the Go books, and Foundation underneath them. concurrency and services on top of the systems knowledge that makes them make sense.',
    books: [
      'go-fundamentals',
      'go-intermediate',
      'shipping-go-web-services',
      'os-fundamentals',
      'networking-fundamentals',
      'dsa-fundamentals',
    ],
    featured: true,
  },
  {
    key: 'rust',
    name: 'Rust',
    blurb:
      'the Rust books, and Foundation underneath them. ownership and lifetimes are easier to hold when you already know what the machine is doing.',
    books: [
      'rust-from-zero',
      'rust-101s',
      'os-fundamentals',
      'networking-fundamentals',
      'dsa-fundamentals',
    ],
    featured: false,
  },
  {
    key: 'all',
    name: 'Everything',
    blurb:
      'every book, including the ones on no track — the container runtime, C, and interview preparation. everything published, and everything published next.',
    books: [],
    featured: false,
  },
]
