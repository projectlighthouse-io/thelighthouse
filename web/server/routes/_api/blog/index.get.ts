import { posts } from '@/data/Blog'

/** Listing needs metadata only — the bodies stay on the server. */
export default defineEventHandler(() =>
  [...posts].sort((a, b) => b.publishedAt.localeCompare(a.publishedAt)),
)
