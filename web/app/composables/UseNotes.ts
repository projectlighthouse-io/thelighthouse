/**
 * A reader's own notes, according to rust.
 *
 * `GET /api/notes` is scoped to the session cookie, so there is no user id to
 * pass and no way to ask for somebody else's. The api decides what comes back;
 * this only decides how to ask.
 *
 * **Client-side only**, like `useAuth` and for the same reason: `/notes` is
 * `ssr: false`, so there is no server render to forward a cookie for. When a
 * signed-in page does start rendering on the server it will need
 * `useRequestHeaders(['cookie'])` — see docs/rebuild.md.
 */

export interface Note {
  id: number
  selectedText: string | null
  noteContent: string | null
  /** ISO-8601, UTC. `null` for rows migrated without a timestamp. */
  createdAt: string | null
  lessonSlug: string
  lessonTitle: string | null
  bookSlug: string
  bookTitle: string | null
}

/** The wire shape, which is snake_case because the columns are. */
interface NoteResponse {
  id: number
  selected_text: string | null
  note_content: string | null
  created_at: string | null
  lesson_slug: string
  lesson_title: string | null
  book_slug: string
  book_title: string | null
}

interface PageResponse {
  notes: NoteResponse[]
  page: number
  per_page: number
  total: number
}

/** How long the search waits after the last keystroke before asking rust. */
const DEBOUNCE_MS = 250

function toNote(note: NoteResponse): Note {
  return {
    id: note.id,
    selectedText: note.selected_text,
    noteContent: note.note_content,
    createdAt: note.created_at,
    lessonSlug: note.lesson_slug,
    lessonTitle: note.lesson_title,
    bookSlug: note.book_slug,
    bookTitle: note.book_title,
  }
}

export function useNotes() {
  const notes = ref<Note[]>([])
  const page = ref<number>(1)
  const perPage = ref<number>(20)
  const total = ref<number>(0)

  // Distinct from `notes.length === 0`, which cannot tell "no notes" from "not
  // asked yet" — and the difference is an empty state shown to somebody who has
  // notes, for as long as the request takes.
  const loaded = ref<boolean>(false)
  const pending = ref<boolean>(false)
  const failed = ref<boolean>(false)

  const search = ref<string>('')

  const pages = computed<number>(() => Math.max(1, Math.ceil(total.value / perPage.value)))

  /**
   * The request in flight, so a reply that has been overtaken by a newer one
   * cannot land on top of it. Typing fast otherwise leaves whichever response
   * happened to be slowest.
   */
  let latest = 0

  async function load(): Promise<void> {
    if (import.meta.server) return

    const ticket = ++latest
    pending.value = true
    failed.value = false

    try {
      const response = await $fetch<PageResponse>('/api/notes', {
        query: { page: page.value, q: search.value || undefined },
      })

      if (ticket !== latest) return

      notes.value = response.notes.map(toNote)
      perPage.value = response.per_page
      total.value = response.total
      // The api clamps, so this is what the page actually is rather than what
      // was asked for.
      page.value = response.page
    }
    catch {
      if (ticket !== latest) return

      // Including a 401: the auth middleware is what sends a signed-out reader
      // to /login, and duplicating that here would race it.
      notes.value = []
      total.value = 0
      failed.value = true
    }
    finally {
      if (ticket === latest) {
        pending.value = false
        loaded.value = true
      }
    }
  }

  function goTo(next: number): void {
    page.value = Math.min(Math.max(1, next), pages.value)
    void load()
  }

  let timer: ReturnType<typeof setTimeout> | null = null

  // A request per keystroke is a request per keystroke. Debounced, and back to
  // page one — page three of the old results is not page three of the new ones.
  watch(search, () => {
    if (timer) clearTimeout(timer)

    timer = setTimeout(() => {
      page.value = 1
      void load()
    }, DEBOUNCE_MS)
  })

  onScopeDispose(() => {
    if (timer) clearTimeout(timer)
  })

  return { notes, page, perPage, total, pages, loaded, pending, failed, search, load, goTo }
}
