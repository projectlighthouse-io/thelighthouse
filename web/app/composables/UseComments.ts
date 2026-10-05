/**
 * A lesson's public thread: every reader's shared notes, and the replies under
 * them.
 *
 * `GET /api/books/{book}/lessons/{lesson}/comments` needs no session — it is
 * the same bytes for everyone and the edge holds it for a minute. Newest
 * top-level comment first, one level of replies under each, oldest first.
 *
 * **Client-side only**, although nothing in it is private. The lesson page is
 * edge-cached for far longer than the thread's sixty seconds, so a server
 * render would freeze whatever the thread said when the page was cached and
 * then hydrate it into something different. Fetching after mount keeps the
 * thread exactly as fresh as the api's own cache, and the server render is a
 * loading state that the client starts from too, so the two always agree.
 *
 * Pages are appended, not replaced: "show older comments" keeps the reader's
 * place in a thread they are already reading.
 */

import type { LessonComment, CommentEntry } from '@/types/Comments'

/** The wire shape: snake_case, as the api serialises it. */
interface AuthorResponse {
  id: number
  name: string | null
  username: string | null
  avatar_url: string | null
}

interface EntryResponse {
  id: number
  selected_text: string | null
  body: string | null
  start_offset: number | null
  end_offset: number | null
  created_at: string | null
  author: AuthorResponse
}

interface CommentResponse extends EntryResponse {
  replies: EntryResponse[]
}

interface PageResponse<T> {
  items: T[]
  page: number
  per_page: number
  total: number
}

/** Which lesson's thread. A slug is unique only within a book. */
export interface CommentsScope {
  book: string
  lesson: string
}

/** Top-level comments per page. Replies ride along with their root. */
export const COMMENTS_PER_PAGE = 20

function toEntry(entry: EntryResponse): CommentEntry {
  return {
    id: entry.id,
    selectedText: entry.selected_text,
    body: entry.body,
    startOffset: entry.start_offset,
    endOffset: entry.end_offset,
    createdAt: entry.created_at,
    author: {
      id: entry.author.id,
      name: entry.author.name,
      username: entry.author.username,
      avatarUrl: entry.author.avatar_url,
    },
  }
}

function toComment(comment: CommentResponse): LessonComment {
  return { ...toEntry(comment), replies: comment.replies.map(toEntry) }
}

/**
 * `scope` is read through `toValue` on every request, for the same reason as
 * `useNotes`: the reader page moves between lessons without unmounting.
 */
export function useComments(scope: MaybeRefOrGetter<CommentsScope>) {
  const items = ref<LessonComment[]>([])

  /** Top-level comments in the thread, as the api last counted them, plus any
   *  this reader has posted since. What "12 comments" says. */
  const total = ref<number>(0)

  /** The last page the api answered. 0 before the first. */
  const page = ref<number>(0)

  /** The api's own count, kept apart from `total` because a comment inserted
   *  locally is not on any server page yet, and "is there more" is about the
   *  server's pages. */
  const serverTotal = ref<number>(0)

  // `loaded` apart from `items.length === 0`, so "nothing yet" is never shown
  // while the first request is still out.
  const loaded = ref<boolean>(false)
  const loading = ref<boolean>(false)
  const failed = ref<boolean>(false)

  const hasMore = computed<boolean>(
    () => page.value * COMMENTS_PER_PAGE < serverTotal.value,
  )

  /**
   * Which request is current. A lesson change while a page is in flight would
   * otherwise let the old lesson's answer land on the new lesson's thread.
   */
  let generation = 0

  async function fetchPage(wanted: number, append: boolean): Promise<void> {
    if (import.meta.server) return

    const mine = ++generation
    const here = toValue(scope)

    loading.value = true
    failed.value = false

    try {
      const response = await $fetch<PageResponse<CommentResponse>>(
        `/api/books/${encodeURIComponent(here.book)}/lessons/${encodeURIComponent(here.lesson)}/comments`,
        { query: { page: wanted, per_page: COMMENTS_PER_PAGE } },
      )

      if (mine !== generation) return

      const incoming = response.items.map(toComment)

      if (append) {
        // A comment posted here, or one written elsewhere since the first
        // page, pushes the server's pages down by one — so the next page can
        // start with a row already on screen. Skipped rather than doubled.
        const seen = new Set(items.value.map(comment => comment.id))
        const fresh = incoming.filter(comment => !seen.has(comment.id))

        total.value += response.total - serverTotal.value
        items.value = [...items.value, ...fresh]
      }
      else {
        items.value = incoming
        total.value = response.total
      }

      serverTotal.value = response.total
      page.value = response.page
    }
    catch {
      if (mine !== generation) return

      // Kept on screen when a later page fails: what was shown is still true.
      if (!append) {
        items.value = []
        total.value = 0
        serverTotal.value = 0
        page.value = 0
      }

      failed.value = true
    }
    finally {
      if (mine === generation) {
        loading.value = false
        loaded.value = true
      }
    }
  }

  /** From page one, dropping anything on screen — a new lesson, or a retry. */
  async function refresh(): Promise<void> {
    loaded.value = false
    await fetchPage(1, false)
  }

  /** The next page of older comments, under the ones already shown. */
  async function loadMore(): Promise<void> {
    if (loading.value || !hasMore.value) return

    await fetchPage(page.value + 1, true)
  }

  /** A comment the reader just posted, on top, without asking the api — its
   *  answer is cached for a minute and would not have it yet. */
  function prepend(comment: LessonComment): void {
    if (items.value.some(existing => existing.id === comment.id)) return

    items.value = [comment, ...items.value]
    total.value += 1
  }

  /** An edit the api accepted, applied wherever the comment sits. */
  function patch(id: number, body: string | null): void {
    items.value = items.value.map(comment => ({
      ...(comment.id === id ? { ...comment, body } : comment),
      replies: comment.replies.map(reply => (reply.id === id ? { ...reply, body } : reply)),
    }))
  }

  /** A delete the api accepted. A root takes its replies with it from view. */
  function drop(id: number): void {
    const wasRoot = items.value.some(comment => comment.id === id)

    items.value = items.value
      .filter(comment => comment.id !== id)
      .map(comment => ({
        ...comment,
        replies: comment.replies.filter(reply => reply.id !== id),
      }))

    if (wasRoot) total.value = Math.max(0, total.value - 1)
  }

  return {
    items, total, page, loaded, loading, failed, hasMore,
    refresh, loadMore, prepend, patch, drop,
  }
}
