import type { Book } from '@/types/Content'

/**
 * Every published book, in the api's order — the listing the books page
 * renders, and the one other pages read a book's place on the shelf from.
 * One handler for its cache key, so the pages that share it agree.
 */
export function useShelf() {
  return useAsyncData('books', () =>
    $fetch<Book[]>('/_api/books').catch(() => [] as Book[]))
}
