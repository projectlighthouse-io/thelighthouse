import type { Book } from '@/types/Content'

// Static until the rust content endpoints exist — see docs/rebuild.md phase 2.
// The tracks mirror each book's own `tracks:` in ohara. Duplicated here
// rather than fetched because this list is the fallback for when nothing is
// fetched at all; when this file goes, so does the copy.

const CDN = 'https://spaces.projectlighthouse.io/books'

export const books: Book[] = [
  {
    slug: 'c-programming',
    title: 'C Programming',
    description:
      'beneath the abstractions — the language that kernels, databases, and embedded systems are written in. manual memory, pointers, structs, and the compilation model, with no garbage collector and no safety net.',
    thumbnailUrl: `${CDN}/c-programming/thumbnail.webp`,
    tracks: {},
    pages: 36,
    price: null,
  },
  {
    slug: 'crack-the-interview',
    title: 'Crack the Interview — 15 Algorithmic Patterns',
    description:
      'stop memorizing solutions — start recognizing the patterns underneath. 15 algorithmic patterns, each with real problems of increasing difficulty, solutions in Go, Rust, and C.',
    thumbnailUrl: `${CDN}/dsa/cract-the-interview.001.jpeg`,
    tracks: {},
    pages: 16,
    price: null,
  },
  {
    slug: 'dsa-fundamentals',
    title: 'DSA Fundamentals',
    description:
      'data structures and algorithms from first principles — starting with how memory works and building up from there. arrays, linked lists, trees, graphs, heaps, hash tables, sorting, and searching.',
    thumbnailUrl: `${CDN}/dsa/dsa.webp`,
    tracks: { go: 6, rust: 5, systems: 3 },
    pages: 60,
    price: null,
  },
  {
    slug: 'go-fundamentals',
    title: 'Go Fundamentals',
    description:
      'from zero to writing real Go — not toy examples, but the kind of code that ships. variables, structs, pointers, error handling, and testing, grounded in patterns you will actually use.',
    thumbnailUrl: `${CDN}/art/lighthouse.001.jpeg`,
    tracks: { go: 1 },
    pages: 22,
    price: null,
  },
  {
    slug: 'go-intermediate',
    title: 'Go Intermediate',
    description:
      'picks up where Go Fundamentals leaves off — concurrency, the GMP scheduler, channels, sync primitives, and performance profiling. understand how goroutines actually work.',
    thumbnailUrl: `${CDN}/go-intermediate/go-intermediate.jpeg`,
    tracks: { go: 2 },
    pages: 31,
    price: null,
  },
  {
    slug: 'networking-fundamentals',
    title: 'Networking Fundamentals',
    description:
      'follow a packet from your keyboard to a server and back — what actually happens when you curl an endpoint. TCP/IP, sockets, routing, DNS, TLS, and HTTP, all from first principles.',
    thumbnailUrl: `${CDN}/networking-fundamentals.webp`,
    tracks: { go: 5, rust: 4, systems: 2 },
    pages: 44,
    price: null,
  },
  {
    slug: 'os-fundamentals',
    title: 'OS Fundamentals',
    description:
      'inside the operating system — how processes are created, how memory is managed, how the scheduler decides what runs next. the layer between your code and the hardware.',
    thumbnailUrl: `${CDN}/os-fundamentals.webp`,
    tracks: { go: 4, rust: 3, systems: 1 },
    pages: 70,
    price: null,
  },
  {
    slug: 'rust-101s',
    title: 'Rust 101s',
    description:
      'Rust language fundamentals from the ground up. ownership, borrowing, lifetimes, structs, enums, pattern matching, error handling, traits, and generics.',
    thumbnailUrl: `${CDN}/rust-101s/rust-101s.jpeg`,
    tracks: { rust: 2 },
    pages: 17,
    price: null,
  },
  {
    slug: 'rust-from-zero',
    title: 'Rust from Zero',
    description:
      'learn Rust by building rune — a real CLI password manager with encrypted vaults and git-based sync. ownership, borrowing, lifetimes, traits, and the type system.',
    thumbnailUrl: `${CDN}/rust-from-zero/rust-from-zero.webp`,
    tracks: { rust: 1 },
    pages: 18,
    price: null,
  },
]
