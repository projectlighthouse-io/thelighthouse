import type { Book } from '@/types/Content'

// Static until the rust content endpoints exist — see docs/rebuild.md phase 2.

const CDN = 'https://spaces.projectlighthouse.io/books'

export const books: Book[] = [
  {
    slug: 'c-programming',
    title: 'C Programming',
    description:
      'beneath the abstractions — the language that kernels, databases, and embedded systems are written in. manual memory, pointers, structs, and the compilation model, with no garbage collector and no safety net.',
    thumbnailUrl: `${CDN}/c-programming/thumbnail.webp`,
    pages: 36,
    price: null,
    inProgress: false,
  },
  {
    slug: 'crack-the-interview',
    title: 'Crack the Interview — 15 Algorithmic Patterns',
    description:
      'stop memorizing solutions — start recognizing the patterns underneath. 15 algorithmic patterns, each with real problems of increasing difficulty, solutions in Go, Rust, and C.',
    thumbnailUrl: `${CDN}/dsa/cract-the-interview.001.jpeg`,
    pages: 16,
    price: null,
    inProgress: true,
  },
  {
    slug: 'dsa-fundamentals',
    title: 'DSA Fundamentals',
    description:
      'data structures and algorithms from first principles — starting with how memory works and building up from there. arrays, linked lists, trees, graphs, heaps, hash tables, sorting, and searching.',
    thumbnailUrl: `${CDN}/dsa/dsa.webp`,
    pages: 60,
    price: null,
    inProgress: false,
  },
  {
    slug: 'go-fundamentals',
    title: 'Go Fundamentals',
    description:
      'from zero to writing real Go — not toy examples, but the kind of code that ships. variables, structs, pointers, error handling, and testing, grounded in patterns you will actually use.',
    thumbnailUrl: `${CDN}/art/lighthouse.001.jpeg`,
    pages: 22,
    price: null,
    inProgress: false,
  },
  {
    slug: 'go-intermediate',
    title: 'Go Intermediate',
    description:
      'picks up where Go Fundamentals leaves off — concurrency, the GMP scheduler, channels, sync primitives, and performance profiling. understand how goroutines actually work.',
    thumbnailUrl: `${CDN}/go-intermediate/go-intermediate.jpeg`,
    pages: 31,
    price: null,
    inProgress: false,
  },
  {
    slug: 'networking-fundamentals',
    title: 'Networking Fundamentals',
    description:
      'follow a packet from your keyboard to a server and back — what actually happens when you curl an endpoint. TCP/IP, sockets, routing, DNS, TLS, and HTTP, all from first principles.',
    thumbnailUrl: `${CDN}/networking-fundamentals.webp`,
    pages: 44,
    price: null,
    inProgress: false,
  },
  {
    slug: 'os-fundamentals',
    title: 'OS Fundamentals',
    description:
      'inside the operating system — how processes are created, how memory is managed, how the scheduler decides what runs next. the layer between your code and the hardware.',
    thumbnailUrl: `${CDN}/os-fundamentals.webp`,
    pages: 70,
    price: null,
    inProgress: false,
  },
  {
    slug: 'rust-101s',
    title: 'Rust 101s',
    description:
      'Rust language fundamentals from the ground up. ownership, borrowing, lifetimes, structs, enums, pattern matching, error handling, traits, and generics.',
    thumbnailUrl: `${CDN}/rust-101s/rust-101s.jpeg`,
    pages: 17,
    price: null,
    inProgress: false,
  },
  {
    slug: 'rust-from-zero',
    title: 'Rust from Zero',
    description:
      'learn Rust by building rune — a real CLI password manager with encrypted vaults and git-based sync. ownership, borrowing, lifetimes, traits, and the type system.',
    thumbnailUrl: `${CDN}/rust-from-zero/rust-from-zero.webp`,
    pages: 18,
    price: null,
    inProgress: false,
  },
]
