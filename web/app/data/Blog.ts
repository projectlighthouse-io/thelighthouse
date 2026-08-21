import type { BlogPost } from '@/types/Content'

// Static until the rust content endpoints exist — see docs/rebuild.md phase 2.
// Extracted from content/blog/*.md; drafts are excluded, same as production.

export const posts: BlogPost[] = [
  {
    slug: "why-an-o1-linked-list-is-80x-slower-than-an-on-array",
    title: "Why an O(1) Linked List is 80x Slower Than an O(N) Array",
    description: "A measured benchmark shows a sorted-array insert beating a linked-list insert by 44x, and a simple access-order change making the same linked list 80x slower. Big-O has nothing to do with either result.",
    publishedAt: "2026-04-11",
    tags: [],
    readMinutes: 20,
  },
  {
    slug: "why-hashmap-isnt-always-o1",
    title: "Why Hashmap Isn't Always O(1)",
    description: "Three measured benchmarks on an M3 Pro show a single hashmap insert 465,275x slower than the median, adversarial input scaling linearly, and arrays winning 60x.",
    publishedAt: "2026-04-14",
    tags: [],
    readMinutes: 20,
  },
]
