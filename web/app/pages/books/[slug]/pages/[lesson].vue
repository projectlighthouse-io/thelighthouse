<script setup lang="ts">
import type { Note } from '@/composables/UseNotes'
import type { LessonResponse } from '@/types/Content'
// only this route needs the reader system
import '@/assets/css/reader.css'

const route = useRoute()
const bookSlug = computed<string>(() => String(route.params.slug))
const lessonSlug = computed<string>(() => String(route.params.lesson))

// Everything the page renders comes back already decided: which body, how far
// through the book, what comes next. The paid half never enters this component,
// which is the point — see docs/rebuild.md. During SSR this calls the handler
// directly, so it costs no HTTP round trip.
const { data } = await useAsyncData(
  () => `lesson:${bookSlug.value}:${lessonSlug.value}`,
  () => $fetch<LessonResponse>(`/_api/books/${bookSlug.value}/pages/${lessonSlug.value}`),
  { watch: [bookSlug, lessonSlug] },
)

if (!data.value) {
  throw createError({ statusCode: 404, statusMessage: 'Lesson not found', fatal: true })
}

const book = computed(() => data.value?.book)
const lesson = computed(() => data.value?.lesson)

/**
 * Which contents entry is highlighted: whatever the url points at.
 *
 * The url already holds the answer — clicking an entry puts `#some-heading`
 * there, and that is the section chosen. Deriving it from scroll position
 * instead meant measuring which heading had passed a line near the top of the
 * viewport, and the last entry can never pass it: there is not a screenful of
 * content below it, so the page stops scrolling while the heading is still
 * halfway down. The measurement then answered `N-1` and overwrote the click.
 *
 * No scroll listener, no bottom-of-page special case, no frame budget. A hash
 * that matches no entry — a stale link, a heading since renamed — leaves the
 * first one highlighted rather than none.
 */
// Seeded with the first entry: a hash is never sent to the server, so this is
// the only thing it can know, and it is what `syncFromHash` falls back to
// anyway. Left empty, the server would paint a contents list with nothing lit
// until hydration.
const activeId = ref<string>(data.value?.toc?.[0]?.id ?? '')

/** The hash, but only when it names a section this lesson actually has. */
const syncFromHash = (): void => {
  const id = decodeURIComponent(window.location.hash.slice(1))
  const known = (data.value?.toc ?? []).some(item => item.id === id)

  activeId.value = known ? id : (data.value?.toc?.[0]?.id ?? '')
}

onMounted(() => {
  syncFromHash()

  // Plain `<a href="#…">` clicks are handled by the browser, not the router,
  // so `route.hash` does not see them — this event does.
  window.addEventListener('hashchange', syncFromHash)
  onBeforeUnmount(() => window.removeEventListener('hashchange', syncFromHash))
})

// A different lesson has different headings, and the hash rarely survives the
// move. Fall back to its first entry rather than keeping the old one lit.
watch(() => data.value?.toc, () => {
  activeId.value = data.value?.toc?.[0]?.id ?? ''
})

/* ---------- annotations: highlights, notes, the bookmark ----------
 *
 * All of it after mount, none of it in the render. `/books/**` is edge-cached
 * and its html has to be identical for every visitor, so a reader's own marks
 * cannot be part of what the server sends — see `useReader`, which resolves the
 * session the same way and for the same reason.
 *
 * The marks are painted into the `v-html` body rather than rendered by vue.
 * Vue does not own that subtree, so there is nothing to render *into*: the
 * prose arrives as a string of markup and the highlights are put on top of it
 * by `utils/Anchor`. Everything below is the bookkeeping that makes that
 * repeatable — what is painted, and what to take off before painting again. */
const { isSignedIn, resolve: resolveReader } = useReader()

const scope = computed(() => ({
  book: bookSlug.value,
  lesson: lessonSlug.value,
}))

const {
  notes,
  load: loadNotes,
  create: createNote,
  edit: editNote,
  remove: removeNote,
} = useNotes(scope)

/**
 * The list under the lesson reads oldest first, while the api answers newest
 * first — right for `/notes`, where the last thing written is the thing being
 * looked for, and backwards here. Down the page is down the lesson, so the
 * notes follow the prose they were taken against.
 *
 * Sorted on the anchor, not on the timestamp: a reader who goes back to
 * annotate the introduction wrote that note last and means it to sit first.
 * Unanchored notes have no place in the prose, so they go at the end.
 */
const inReadingOrder = computed<Note[]>(() =>
  [...notes.value].sort(
    (a, b) => (a.startOffset ?? Infinity) - (b.startOffset ?? Infinity),
  ),
)

// Destructured rather than kept as objects: refs reached through a plain
// object are not unwrapped in a template, and `bookmark.current.value` in
// markup is a `.value` that only exists because of how this was called.
const {
  current: currentBookmark,
  load: loadBookmark,
  place: saveBookmark,
  clear: clearBookmark,
} = useBookmark(bookSlug, lessonSlug)

const {
  anchor: selected,
  spot: menuAt,
  open: menuOpen,
  clear: clearSelection,
} = useSelection()

const noteDialogOpen = ref<boolean>(false)
const savingNote = ref<boolean>(false)
const noteError = ref<string | null>(null)

/** The note whose popover is open, and where to put it. */
const openNote = ref<Note | null>(null)
const openNoteAt = ref<{ x: number, y: number }>({ x: 0, y: 0 })
const removingNote = ref<boolean>(false)

/**
 * Every mark currently on the page, by the note it belongs to.
 *
 * Held rather than found again with a query selector, because a passage can be
 * several `<mark>`s and "which of these belong to note 12" is not a question
 * the DOM can answer without an attribute nobody else needs.
 */
const painted = new Map<number, HTMLElement[]>()
let bookmarkMarks: HTMLElement[] = []

const body = (): Element | null =>
  document.querySelector('[data-lesson-content]')

/** Takes every mark off, so the next pass starts from the prose as rendered. */
const unpaintAll = (): void => {
  for (const marks of painted.values()) unpaint(marks)
  painted.clear()

  unpaint(bookmarkMarks)
  bookmarkMarks = []
}

/**
 * Draws what the api sent, from scratch.
 *
 * Cheap enough to do wholesale — a lesson has tens of notes, not thousands —
 * and a full repaint cannot leave a mark behind for a note that was deleted,
 * which an incremental one eventually does.
 */
const repaint = (): void => {
  const container = body()
  if (!container) return

  unpaintAll()

  for (const note of notes.value) {
    // A note written with nothing selected — the api allows it — has no place
    // on the page to draw it.
    if (note.startOffset === null || note.endOffset === null) continue

    const marks = paint(
      container,
      { start: note.startOffset, end: note.endOffset },
      'reader-mark-note',
    )
    if (!marks.length) continue

    for (const mark of marks) {
      mark.addEventListener('click', (event) => {
        event.stopPropagation()
        openNote.value = note
        openNoteAt.value = { x: event.clientX, y: event.clientY + 12 }
      })
    }

    painted.set(note.id, marks)
  }

  if (currentBookmark.value) {
    bookmarkMarks = paint(
      container,
      {
        start: currentBookmark.value.startOffset,
        end: currentBookmark.value.endOffset,
      },
      'reader-mark-bookmark',
    )
  }
}

const saveNote = async (content: string, isPublic: boolean): Promise<void> => {
  if (!selected.value) return

  savingNote.value = true
  noteError.value = await createNote(content, selected.value, isPublic)
  savingNote.value = false

  if (noteError.value) return

  noteDialogOpen.value = false
  clearSelection()
  repaint()
}

const deleteOpenNote = async (): Promise<void> => {
  if (!openNote.value) return

  removingNote.value = true
  const failed = await removeNote(openNote.value.id)
  removingNote.value = false

  if (failed) return

  openNote.value = null
  repaint()
}

/** Places the bookmark where the selection is, moving it if there was one. */
const placeBookmark = async (): Promise<void> => {
  if (!selected.value) return

  await saveBookmark(selected.value)
  clearSelection()
  repaint()
}

const removeBookmark = async (): Promise<void> => {
  await clearBookmark()
  repaint()
}

const signIn = (): void => {
  navigateTo({ path: '/login', query: { redirect: route.fullPath } })
}

/** Back to the passage the reader marked. */
const goToBookmark = (): void => {
  bookmarkMarks[0]?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

/* ---------- the list under the lesson ---------- */
const editingNoteId = ref<number | null>(null)
const savingNoteId = ref<number | null>(null)

/**
 * Up to the highlight a listed note belongs to.
 *
 * `painted` is the only thing that knows which elements are note 12's — the
 * marks carry no id, because nothing else would have read one.
 */
const jumpToNote = (note: Note): void => {
  painted.get(note.id)?.[0]?.scrollIntoView({
    behavior: 'smooth',
    block: 'center',
  })
}

const saveNoteEdit = async (note: Note, content: string): Promise<void> => {
  savingNoteId.value = note.id
  const failed = await editNote(note.id, content)
  savingNoteId.value = null

  // Left open on failure, with what was typed still in the field. Closing it
  // would look like the edit saved.
  if (failed) return

  editingNoteId.value = null
}

const removeListedNote = async (note: Note): Promise<void> => {
  savingNoteId.value = note.id
  const failed = await removeNote(note.id)
  savingNoteId.value = null

  if (failed) return

  // The highlight in the prose goes with it.
  repaint()
}

// The popover stops propagation, so anything that reaches the document is a
// click somewhere else, and somewhere else means close it.
const closePopover = (): void => {
  openNote.value = null
}

onMounted(() => document.addEventListener('mousedown', closePopover))
onBeforeUnmount(() => document.removeEventListener('mousedown', closePopover))

/**
 * Loads a lesson's annotations and draws them.
 *
 * `resolve()` first so `csrfHeader` has a token by the time anything is
 * written; the two loads then run together because neither needs the other.
 */
const loadAnnotations = async (): Promise<void> => {
  await resolveReader()
  if (!isSignedIn.value) return

  await Promise.all([loadNotes(), loadBookmark()])
  // The body is `v-html` from data that is already here, so it is in the DOM
  // by now — but a repaint before it is would silently draw nothing, and
  // waiting a tick costs nothing.
  await nextTick()
  repaint()
}

onMounted(loadAnnotations)

/**
 * Prev and next stay on this route, so the component is reused and none of the
 * above runs again on its own. The marks belong to the lesson that is gone.
 *
 * Keyed on the body rather than on `lessonSlug`, which is what the reader
 * changed: the slug moves the moment the link is clicked, while the prose it
 * names arrives one fetch later. Painting offsets from the new lesson onto the
 * old lesson's text puts every highlight in the wrong place — and it is not a
 * crash, so it would only ever be noticed by looking.
 */
watch(() => data.value?.html, async () => {
  unpaintAll()
  openNote.value = null
  await loadAnnotations()
})

// A mark holds a reference to a listener that closes over this page's state.
onBeforeUnmount(unpaintAll)

useSeo(() => ({
  title: `${lesson.value?.title} — ${book.value?.title}`,
  description: lesson.value?.description ?? '',
  type: 'article',
  image: book.value?.thumbnailUrl,
}))

// Google cannot tell a paywall from cloaking without this. isAccessibleForFree
// plus a hasPart pointing at the gated selector is the sanctioned way to say
// "the gap is deliberate" — see docs/rebuild.md.
useJsonLd('lesson', () => ({
  '@type': 'Article',
  'headline': lesson.value?.title,
  'description': lesson.value?.description,
  'inLanguage': 'en',
  'author': { '@type': 'Person', 'name': 'Aryan Ahmed' },
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
  'isPartOf': { '@type': 'Book', 'name': book.value?.title, 'url': `${SITE.url}/books/${bookSlug.value}` },
  'isAccessibleForFree': !lesson.value?.locked,
  ...(lesson.value?.locked
    ? {
        hasPart: {
          '@type': 'WebPageElement',
          'isAccessibleForFree': false,
          'cssSelector': '.paywalled',
        },
      }
    : {}),
}))

useJsonLd('crumbs', () => ({
  '@type': 'BreadcrumbList',
  'itemListElement': [
    { '@type': 'ListItem', 'position': 1, 'name': 'Books', 'item': `${SITE.url}/books` },
    { '@type': 'ListItem', 'position': 2, 'name': book.value?.title, 'item': `${SITE.url}/books/${bookSlug.value}` },
    { '@type': 'ListItem', 'position': 3, 'name': lesson.value?.title },
  ],
}))
</script>

<template>
  <div v-if="data && book && lesson" class="reader-shell">
    <div class="reader-progress"><span :style="{ width: `${data.percent}%` }" /></div>

    <div class="reader-subbar">
      <div class="reader-subbar__inner">
        <NuxtLink class="reader-subbar__book" :to="`/books/${book.slug}`">
          {{ book.title }}
        </NuxtLink>
        <span class="reader-subbar__sep">›</span>
        <span class="reader-subbar__cur">{{ lesson.title }}</span>
        <span class="reader-subbar__spacer" />

        <!-- Only once there is one. A control that is present and inert most of
             the time reads as broken rather than as empty. -->
        <button
          v-if="currentBookmark"
          type="button"
          class="reader-iconbtn is-active"
          @click="goToBookmark"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
            <path d="M6 3h12v18l-6-4.5L6 21z" />
          </svg>
          your bookmark
        </button>
        <button
          v-if="currentBookmark"
          type="button"
          class="reader-iconbtn"
          @click="removeBookmark"
        >
          remove
        </button>

        <span class="reader-subbar__rt">{{ data.readMinutes }} min read</span>
      </div>
    </div>

    <!-- No aside: this lesson has nothing to put in the third column. -->
    <div class="reader-layout reader-layout--no-aside">
      <aside class="reader-toc">
        <div class="reader-toc__label">On this page</div>
        <nav class="reader-toc__list">
          <a
            v-for="item in data.toc"
            :key="item.id"
            class="reader-toc__item"
            :class="{ 'is-active': item.id === activeId }"
            :href="`#${item.id}`"
          >
            {{ item.text }}
          </a>
        </nav>
      </aside>

      <article class="reader-article">
        <h1>{{ lesson.title }}</h1>
        <p v-if="lesson.description" class="reader-dek">{{ lesson.description }}</p>

        <div class="reader-prose" style="margin-top: 44px">
          <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
          <div class="lesson-content" data-lesson-content v-html="data.html" />
        </div>

        <div
          v-if="lesson.locked"
          class="paywalled mt-12 rounded-md border-2 border-dashed border-rule bg-paper p-8 text-center"
        >
          <p class="font-mono text-xs tracking-[0.2em] uppercase text-teal">keep reading</p>
          <h2 class="mt-3 font-serif text-2xl text-ink">
            The rest of this chapter is part of {{ book.title }}
          </h2>
          <p class="mx-auto mt-3 max-w-md text-mono-body">
            {{ data.remainingSections }} more sections, and the project that goes with them.
          </p>
          <div class="mt-6 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <NuxtLink
              to="/pricing"
              class="rounded-md bg-ink px-5 py-3 text-base font-medium text-on-ink transition hover:bg-ink-hover"
            >
              Get the book
            </NuxtLink>
            <NuxtLink
              to="/login"
              class="rounded-md border border-stroke bg-panel px-5 py-3 text-base font-medium text-ink transition hover:bg-paper-warm"
            >
              I already own it
            </NuxtLink>
          </div>
        </div>

        <ReaderNotesList
          :notes="inReadingOrder"
          :editing-id="editingNoteId"
          :saving-id="savingNoteId"
          @jump="jumpToNote"
          @edit="editingNoteId = $event.id"
          @save="saveNoteEdit"
          @cancel-edit="editingNoteId = null"
          @remove="removeListedNote"
        />

        <!-- Stacked on a phone: two lesson titles do not fit side by side. -->
        <div
          class="mt-12 flex flex-col items-stretch justify-between gap-3 sm:flex-row sm:items-center sm:gap-4"
        >
          <NuxtLink
            v-if="data.previous"
            :to="`/books/${book.slug}/pages/${data.previous.slug}`"
            class="btn-chalk text-sm text-ink"
          >
            ← {{ data.previous.title }}
          </NuxtLink>
          <span v-else />
          <NuxtLink
            v-if="data.next"
            :to="`/books/${book.slug}/pages/${data.next.slug}`"
            class="btn-chalk text-sm text-ink"
          >
            {{ data.next.title }} →
          </NuxtLink>
        </div>
      </article>

    </div>

    <!-- The three floating pieces. Outside `.reader-layout` because all three
         are `position: fixed` and belong to the viewport, not to the column. -->
    <ReaderSelectionMenu
      v-if="menuOpen && selected"
      :x="menuAt.x"
      :y="menuAt.y"
      :length="selected.text.length"
      :signed-in="isSignedIn"
      :bookmarked="!!currentBookmark"
      @note="noteDialogOpen = true"
      @bookmark="placeBookmark"
      @sign-in="signIn"
    />

    <ReaderNoteDialog
      v-if="noteDialogOpen && selected"
      :passage="selected.text"
      :saving="savingNote"
      :error="noteError"
      @save="saveNote"
      @cancel="noteDialogOpen = false; noteError = null; clearSelection()"
    />

    <ReaderNotePopover
      v-if="openNote"
      :note="openNote"
      :x="openNoteAt.x"
      :y="openNoteAt.y"
      :removing="removingNote"
      @remove="deleteOpenNote"
      @close="openNote = null"
    />
  </div>
</template>
