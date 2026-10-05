/**
 * Where a reader left off in one lesson.
 *
 * **One per reader per lesson, and placing a second moves the first.** That is
 * the whole difference from a note: notes accumulate, a bookmark is a single
 * mark that travels down the page as the reader does. The api enforces it with
 * a unique constraint, so nothing here has to delete before it saves.
 *
 * Client-side only, like `useNotes` and for the same reason — see its header.
 */

import { csrfHeader } from '@/composables/UseReader'

export interface Bookmark {
  id: number
  selectedText: string
  /** Character offsets over the rendered lesson body. Never null: a bookmark
   *  with no anchor is not a place, and the columns are `NOT NULL`. */
  startOffset: number
  endOffset: number
  createdAt: string | null
}

/** The wire shape, which is snake_case because the columns are. */
interface BookmarkResponse {
  id: number
  selected_text: string
  start_offset: number
  end_offset: number
  created_at: string | null
}

function toBookmark(bookmark: BookmarkResponse): Bookmark {
  return {
    id: bookmark.id,
    selectedText: bookmark.selected_text,
    startOffset: bookmark.start_offset,
    endOffset: bookmark.end_offset,
    createdAt: bookmark.created_at,
  }
}

/**
 * The slugs are read on every call rather than captured once, for the reason
 * `useNotes` gives: prev and next move the reader between lessons without
 * unmounting the page.
 */
export function useBookmark(
  book: MaybeRefOrGetter<string>,
  lesson: MaybeRefOrGetter<string>,
) {
  const current = ref<Bookmark | null>(null)
  const loaded = ref<boolean>(false)

  const url = (): string => `/api/bookmarks/${toValue(book)}/${toValue(lesson)}`

  /**
   * Asks whether this reader has a bookmark here.
   *
   * A 404 is the ordinary answer for "no bookmark", not a failure — as is the
   * 401 a signed-out reader gets. Both leave `current` null, which is what the
   * page draws for either.
   */
  async function load(): Promise<void> {
    if (import.meta.server) return

    try {
      current.value = toBookmark(await $fetch<BookmarkResponse>(url()))
    }
    catch {
      current.value = null
    }
    finally {
      loaded.value = true
    }
  }

  /** Places the bookmark, moving it if there already was one. */
  async function place(anchor: {
    text: string
    start: number
    end: number
  }): Promise<Bookmark | null> {
    try {
      const saved = await $fetch<BookmarkResponse>(url(), {
        method: 'PUT',
        headers: csrfHeader(),
        body: {
          selected_text: anchor.text,
          start_offset: anchor.start,
          end_offset: anchor.end,
        },
      })

      current.value = toBookmark(saved)

      return current.value
    }
    catch {
      return null
    }
  }

  /**
   * Takes the bookmark off.
   *
   * Cleared locally whatever the api said. A delete that failed leaves the row
   * there, and the next load will show it again — but leaving the mark drawn
   * after the reader asked for it to go reads as a broken button, and the
   * truthful state is one refresh away either way.
   */
  async function clear(): Promise<void> {
    try {
      await $fetch(url(), { method: 'DELETE', headers: csrfHeader() })
    }
    finally {
      current.value = null
    }
  }

  return { current, loaded, load, place, clear }
}
