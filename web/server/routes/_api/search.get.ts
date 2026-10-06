import type { ApiBookDetail, ApiBookSummary } from '#server/utils/Lighthouse'
import { fromApi } from '#server/utils/Lighthouse'
import type { SearchEntry } from '@/types/Content'

/**
 * Every book, chapter and lesson as one flat list, for the command palette.
 *
 * The browser asks once, on the palette's first open, and filters in memory
 * from then on. Building it costs the listing plus one detail per book, so the
 * result is held in nitro's cache for the same five minutes `/books/**` is held
 * at the edge — the api is asked at most once a window, whoever is searching.
 *
 * ponytail: one detail fetch per book. Fine for a shelf of tens; a flat
 * `/api/search` on the rust side if the shelf reaches hundreds.
 */
export default defineCachedEventHandler(async () => {
  const books = await fromApi<ApiBookSummary[]>('/api/books')
  const details = await Promise.all(
    books.map(book => fromApi<ApiBookDetail>(`/api/books/${book.slug}`)),
  )

  return details.flatMap((book): SearchEntry[] => {
    const shelf = book.title.toLowerCase()

    return [
      { kind: 'book', title: book.title, description: book.description ?? '', group: 'books', to: `/books/${book.slug}` },
      ...book.chapters.flatMap((chapter): SearchEntry[] => {
        const first = chapter.lessons[0]

        return [
          // A chapter has no page of its own; its first lesson is where it starts.
          ...(first
            ? [{ kind: 'chapter' as const, title: chapter.title, description: '', group: shelf, to: `/books/${book.slug}/pages/${first.slug}` }]
            : []),
          ...chapter.lessons.map(lesson => ({
            kind: 'lesson' as const,
            title: lesson.title,
            description: lesson.description ?? '',
            group: `${shelf} / ${chapter.title.toLowerCase()}`,
            to: `/books/${book.slug}/pages/${lesson.slug}`,
          })),
        ]
      }),
    ]
  })
}, { name: 'search', maxAge: 300, swr: true })
