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
  /** The note's own column — unchanged when a lesson is renamed. */
  lessonId: number
  /** A slug is unique only within a book, so both are needed to name a lesson. */
  lessonSlug: string
  bookSlug: string
  /** Whether this note shows in the lesson's thread for other readers. */
  isPublic: boolean
  /** Set when this note is a reply. `null` for a top-level note. */
  parentId: number | null
}

/** The wire shape, which is snake_case because the columns are. */
interface NoteResponse {
  id: number
  selected_text: string | null
  note_content: string | null
  created_at: string | null
  lesson_id: number
  lesson_slug: string
  book_slug: string
  is_public: boolean
  parent_id: number | null
}

/**
 * The envelope every listing endpoint answers with — `items`, not a name per
 * endpoint, so a second listing reuses this rather than copying it.
 */
interface PageResponse<T> {
  items: T[]
  page: number
  per_page: number
  total: number
}

/** How long the search waits after the last keystroke before asking rust. */
const DEBOUNCE_MS = 250

/** Shown when the api refused but said nothing a reader can act on. */
const GENERIC_FAILURE = 'That could not be saved. Please try again.'

/**
 * The api's own message for a refused write, or something generic.
 *
 * Rust answers a validation failure with `{ error }` — those messages are
 * written for the person who typed the note, so showing them beats replacing
 * them with a guess. Anything else (a 403, a 429, the network) has no message
 * worth surfacing verbatim.
 */
function problem(error: unknown): string {
  const data = (error as { data?: { error?: unknown } } | undefined)?.data

  return typeof data?.error === 'string' ? data.error : GENERIC_FAILURE
}

function toNote(note: NoteResponse): Note {
  return {
    id: note.id,
    selectedText: note.selected_text,
    noteContent: note.note_content,
    createdAt: note.created_at,
    lessonId: note.lesson_id,
    lessonSlug: note.lesson_slug,
    bookSlug: note.book_slug,
    isPublic: note.is_public,
    parentId: note.parent_id,
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
      const response = await $fetch<PageResponse<NoteResponse>>('/api/notes', {
        query: { page: page.value, q: search.value || undefined },
      })

      if (ticket !== latest) return

      notes.value = response.items.map(toNote)
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

  /**
   * Rewrites one note's body.
   *
   * The api answers with the row as stored, so what lands in the list is what
   * the database holds rather than what was typed — a note the api trimmed or
   * refused does not sit on screen looking saved.
   *
   * Returns the api's message on refusal, or `null` on success. A thrown error
   * is not a useful thing to hand a template.
   */
  async function edit(id: number, content: string): Promise<string | null> {
    try {
      const updated = await $fetch<NoteResponse>(`/api/notes/${id}`, {
        method: 'PATCH',
        headers: csrfHeader(),
        body: { note_content: content },
      })

      const at = notes.value.findIndex(note => note.id === id)
      if (at !== -1) notes.value[at] = toNote(updated)

      return null
    }
    catch (error) {
      return problem(error)
    }
  }

  /**
   * Deletes one note, and reloads rather than splicing.
   *
   * Splicing would leave the page one row short and `total` a row high until
   * something else refetched, and on the last page it would leave an empty
   * page the reader is still standing on. A reload costs one request and is
   * always right.
   */
  async function remove(id: number): Promise<string | null> {
    try {
      await $fetch(`/api/notes/${id}`, { method: 'DELETE', headers: csrfHeader() })

      // Step back if that was the only row on this page.
      if (notes.value.length === 1 && page.value > 1) page.value -= 1

      await load()

      return null
    }
    catch (error) {
      return problem(error)
    }
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

  return {
    notes,
    page,
    perPage,
    total,
    pages,
    loaded,
    pending,
    failed,
    search,
    load,
    goTo,
    edit,
    remove,
  }
}
