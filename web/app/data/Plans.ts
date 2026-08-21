export interface Plan {
  key: string
  name: string
  blurb: string
  price: string
  period: string
  note: string
  features: string[]
  cta: string
  featured: boolean
}

// $49/year and the $119 saving are the real founder-edition numbers, taken from
// the laravel FounderEditionCta defaults. Monthly and lifetime prices live in
// the database, so the values below are PLACEHOLDERS until the api serves them.
export const plans: Plan[] = [
  {
    key: 'seeker',
    name: 'Seeker',
    blurb:
      "whether you're starting your programming journey or a seasoned programmer, we have free content for you to explore.",
    price: '0',
    period: '/ forever',
    note: 'free books, starter projects',
    features: [
      'free books and lessons',
      'starter projects, validated locally with luxctl',
      'private notes and bookmarks',
      'community engagement',
      'hand-crafted diagrams for deeper understanding',
      'slack — build the community, grow together',
    ],
    cta: 'free for everyone',
    featured: false,
  },
  {
    key: 'voyage',
    name: 'Voyage',
    blurb:
      'along with everything in seeker, get access to intermediate books, challenges, labs and more lab time.',
    price: '49',
    period: '/ year',
    note: '~$4.08/month · save $119',
    features: [
      'everything in seeker',
      'all intermediate level books and lessons',
      'more challenging data structures and algorithms',
      'build-your-own and challenge yourself projects',
      'the CLI workbench — grep, sed, awk, find, jq, shell',
      'XP tracking',
      'luxctl — your swiss knife for lighthouse',
    ],
    cta: 'Get Pro',
    featured: true,
  },
  {
    key: 'lifetime',
    name: 'Lifetime',
    blurb:
      'everything in voyage, forever. one payment, no renewals, every future update included.',
    price: '199',
    period: '/ once',
    note: 'placeholder price — served by the api in phase 5',
    features: [
      'everything in voyage',
      'no renewals, ever',
      'every future voyage release included',
      'founder pricing locked in',
    ],
    cta: 'Buy once',
    featured: false,
  },
]
