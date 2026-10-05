/**
 * A reader's own notes, according to rust.
 *
 * `GET /api/notes` is scoped to the session cookie, so there is no user id to
 * pass and no way to ask for somebody else's. Pass `book` and `lesson` and it
 * narrows to one lesson — the reader page's case, where the notes are drawn on
 * the passages they were taken against; pass neither and it is the whole shelf,
 * which is what `/notes` wants.
 *
 * **Client-side only**, like `useReader` and for the same reason: `/books/**`
 * is edge-cached and its html has to be identical for everyone, so a signed-in
 * reader's notes cannot be part of the server render. They arrive after mount
 * and the page draws them then.
 */

import { csrfHeader } from '@/composables/UseReader'

export interface Note {
  id: number
  selectedText: string | null
  noteContent: string | null
  /** ISO-8601, UTC. `null` for rows migrated without a timestamp. */
  createdAt: string | null
  /** A uuid the content repo mints, not a number — see the api's
   *  `20260828020000_content_owns_its_ids.sql`. Opaque: compared, never
   *  parsed. */
  lessonId: string
  /** A slug is unique only within a book, so both are needed to name a lesson. */
  lessonSlug: string
  bookSlug: string
  /** Whether this note shows in the lesson's thread for other readers. */
  isPublic: boolean
  /** Set when this note is a reply. `null` for a top-level note. */
  parentId: number | null
  /** Character offsets over the rendered lesson body, when it was taken
   *  against a passage. `null` on a note written with nothing selected. */
  startOffset: number | null
  endOffset: number | null
}

/** The wire shape, which is snake_case because the columns are. */
interface NoteResponse {
  id: number
  selected_text: string | null
  note_content: string | null
  created_at: string | null
  lesson_id: string
  lesson_slug: string
  book_slug: string
  is_public: boolean
  parent_id: number | null
  start_offset: number | null
  end_offset: number | null
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

/** Which lesson to narrow to, or nothing for all of a reader's notes. */
export interface NotesScope {
  book: string
  lesson: string
}

/**
 * A lesson's worth. High enough that a reader page draws every highlight it
 * has rather than the first page of them, and the api clamps it anyway.
 */
const PER_LESSON = 50

/** Shown when the api refused but said nothing a reader can act on. */
const GENERIC_FAILURE = 'That could not be saved. Please try again.'

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
    startOffset: note.start_offset,
    endOffset: note.end_offset,
  }
}

/**
 * `scope` is read through `toValue` on every call rather than captured once:
 * the reader page moves between lessons without unmounting — prev and next are
 * `NuxtLink`s into the same route — so a scope frozen at setup would keep
 * writing new notes against the lesson the reader arrived on.
 */
export function useNotes(scope?: MaybeRefOrGetter<NotesScope | undefined>) {
  const notes = ref<Note[]>([])
  const total = ref<number>(0)

  // Distinct from `notes.length === 0`, which cannot tell "no notes" from "not
  // asked yet" — and the difference is an empty state shown to somebody who has
  // notes, for as long as the request takes.
  const loaded = ref<boolean>(false)
  const pending = ref<boolean>(false)

  // Distinct again from an empty list: the notes page offers a retry, and
  // "could not be loaded" is the only state that should show it. The reader
  // page ignores this and draws nothing, which is the same as it did before.
  const failed = ref<boolean>(false)

  /** Which page of the reader's own notes. The lesson scope never leaves 1 —
   *  fifty notes against one lesson is already more than anyone writes. */
  const page = ref<number>(1)

  /** The search box, sent as `q`. Empty means no filter, which is what the api
   *  reads an absent `q` as. */
  const search = ref<string>('')

  const pages = computed<number>(
    () => Math.max(1, Math.ceil(total.value / PER_LESSON)),
  )

  async function load(): Promise<void> {
    if (import.meta.server) return

    pending.value = true
    failed.value = false

    try {
      const response = await $fetch<PageResponse<NoteResponse>>('/api/notes', {
        query: {
          per_page: PER_LESSON,
          page: page.value,
          // Omitted when empty rather than sent as `q=`: the api reads a blank
          // term as no filter either way, and leaving it off keeps the url the
          // reader sees honest about what was asked.
          ...(search.value.trim() ? { q: search.value.trim() } : {}),
          ...toValue(scope),
        },
      })

      notes.value = response.items.map(toNote)
      total.value = response.total

      // The api clamps the page it was asked for, so this is the page actually
      // answered — a `goTo` past the end lands on the last one rather than on
      // an empty list numbered something that does not exist.
      page.value = response.page
    }
    catch {
      // Including a 401: a signed-out reader has no notes to draw, and the
      // page is perfectly readable without them.
      notes.value = []
      total.value = 0
      failed.value = true
    }
    finally {
      pending.value = false
      loaded.value = true
    }
  }

  /** Moves to a page, if it is one. Clamped here as well as by the api so an
   *  out-of-range click costs no request at all. */
  async function goTo(to: number): Promise<void> {
    const wanted = Math.min(Math.max(1, Math.trunc(to)), pages.value)

    if (wanted === page.value) return

    page.value = wanted
    await load()
  }

  // Typing restarts at page one: page four of "everything" is not page four of
  // the three notes that match, and staying there shows an empty list.
  //
  // Debounced, so a search is one request per pause rather than one per
  // keystroke. `loaded` is deliberately left alone — the list on screen stays
  // until the new answer replaces it, rather than flashing an empty state
  // between every letter.
  let debounce: ReturnType<typeof setTimeout> | null = null

  watch(search, () => {
    if (debounce) clearTimeout(debounce)

    debounce = setTimeout(() => {
      page.value = 1
      void load()
    }, 250)
  })

  onScopeDispose(() => {
    if (debounce) clearTimeout(debounce)
  })

  /**
   * Saves a note against a passage, or against the lesson when `anchor` is
   * absent.
   *
   * Returns the api's message on refusal and `null` on success — a thrown
   * error is not a useful thing to hand a template. The saved note is pushed
   * from the api's answer, so what the page draws is the row as stored.
   */
  async function create(
    content: string,
    anchor?: { text: string, start: number, end: number },
    isPublic = true,
  ): Promise<Refused | null> {
    const here = toValue(scope)
    if (!here) return { message: GENERIC_FAILURE, fields: {} }

    try {
      const saved = await $fetch<NoteResponse>('/api/notes', {
        method: 'POST',
        headers: csrfHeader(),
        body: {
          book: here.book,
          lesson: here.lesson,
          note_content: content,
          selected_text: anchor?.text ?? null,
          start_offset: anchor?.start ?? null,
          end_offset: anchor?.end ?? null,
          is_public: isPublic,
        },
      })

      notes.value.push(toNote(saved))
      total.value += 1

      return null
    }
    catch (error) {
      return refusedBy(error, GENERIC_FAILURE)
    }
  }

  /**
   * Rewrites one note's body.
   *
   * The api answers with the row as stored, so what lands in the list is what
   * the database holds rather than what was typed — a note the api trimmed or
   * refused does not sit on screen looking saved.
   */
  async function edit(id: number, content: string): Promise<Refused | null> {
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
      return refusedBy(error, GENERIC_FAILURE)
    }
  }

  /** Deletes one note, and drops it from the list the page is drawing. */
  async function remove(id: number): Promise<Refused | null> {
    try {
      await $fetch(`/api/notes/${id}`, {
        method: 'DELETE',
        headers: csrfHeader(),
      })

      notes.value = notes.value.filter(note => note.id !== id)
      total.value = Math.max(0, total.value - 1)

      return null
    }
    catch (error) {
      return refusedBy(error, GENERIC_FAILURE)
    }
  }

  return {
    notes, page, pages, total, loaded, pending, failed, search,
    load, goTo, create, edit, remove,
  }
}
