<script setup lang="ts">
import type { Note } from '@/composables/UseNotes'
import type { Chapter, LessonResponse, LessonSummary } from '@/types/Content'
// only this route needs the reader system
import '@/assets/css/reader.css'

const route = useRoute()
const bookSlug = computed<string>(() => String(route.params.slug))
const lessonSlug = computed<string>(() => String(route.params.lesson))

// Everything the page renders comes back already decided: which body, how far
// through the book, what comes next. The paid half never enters this component,
// which is the point — see docs/rebuild.md. During SSR this calls the handler
// directly, so it costs no HTTP round trip.
const { data, error } = await useAsyncData(
  () => `lesson:${bookSlug.value}:${lessonSlug.value}`,
  () => $fetch<LessonResponse>(`/_api/books/${bookSlug.value}/pages/${lessonSlug.value}`),
  { watch: [bookSlug, lessonSlug] },
)

/**
 * Why the failure is read before the absence.
 *
 * `useAsyncData` does not throw — a request that failed leaves `data` null and
 * puts the reason in `error`. Checking only `data` therefore reports an api
 * that is down, a 500, or a timeout as "lesson not found", which sends
 * whoever reads it looking for missing content that is not missing.
 *
 * A rejection with no status is nitro never reaching the api at all, and 502 is
 * what that is: this process is the gateway, and its upstream did not answer.
 */
if (error.value) {
  const status = error.value.statusCode ?? 502

  throw createError({
    statusCode: status,
    statusMessage:
      status === 404 ? 'Lesson not found' : 'The api is not answering',
    fatal: true,
  })
}

// Past the error check, so this is a genuinely empty answer.
if (!data.value) {
  throw createError({ statusCode: 404, statusMessage: 'Lesson not found', fatal: true })
}

const book = computed(() => data.value?.book)
const lesson = computed(() => data.value?.lesson)

/**
 * "chapter 03 · lesson 12". The lesson response says where the lesson falls
 * in the book; which chapter it is in comes from the book's contents — the
 * same request, and the same cache key, as the book page.
 */
const { data: contents } = await useAsyncData(
  () => `book:${bookSlug.value}`,
  () => $fetch<{ chapters: Chapter[], lessons: LessonSummary[] }>(`/_api/books/${bookSlug.value}`)
    .catch(() => null),
  { watch: [bookSlug] },
)

const eyebrow = computed<string>(() => {
  const pad = (n: number): string => String(n).padStart(2, '0')
  const lessonNo = `lesson ${pad(data.value?.position ?? 1)}`
  const chapterId = contents.value?.lessons.find(l => l.slug === lessonSlug.value)?.chapterId
  const chapterAt = contents.value?.chapters.findIndex(c => c.id === chapterId) ?? -1

  return chapterAt === -1 ? lessonNo : `chapter ${pad(chapterAt + 1)} · ${lessonNo}`
})

/**
 * The half of the lesson a reader has to have bought.
 *
 * **Fetched in the browser, never during SSR.** This document is the same
 * bytes for everyone and is held at the edge; rendering the paid half into it
 * would put one reader's entitlement in a shared cache. So the page ships
 * locked for everybody and unlocks itself afterwards, which is also why the
 * api answers this url `no-store`.
 *
 * A 404 is the ordinary answer for a reader who has not bought it, so it is
 * not logged or shown — the paywall below is what it looks like.
 */
interface Heading { id: string, text: string, locked?: boolean }

/**
 * The whole lesson, once the api has been asked as *this reader*.
 *
 * Nuxt renders this page on the server with no session cookie, and that
 * document is held at the edge for everyone — so the server-rendered copy is
 * always the locked one. A reader who has bought the book asks the same url
 * again from the browser, where the cookie goes, and the api answers with the
 * whole lesson and `no-store`.
 *
 * The extra request is the price of the document being cacheable, not of the
 * api having two urls. It has one.
 */
const unlockedHtml = ref<string | null>(null)
const unlockedToc = ref<Heading[] | null>(null)

/**
 * The whole contents list: the free half's headings, then the paid half's.
 *
 * The sidebar is built from the lesson response, which only ever describes the
 * free half — so without this a reader who paid gets the whole lesson and a
 * contents list that stops a third of the way down it.
 */
/**
 * The contents list, which covers the whole lesson either way.
 *
 * The api sends every heading and marks the ones this reader cannot reach, so
 * a locked lesson still shows what is in it — only the anchors change, because
 * a locked section has no element to scroll to.
 */
const toc = computed<Heading[]>(
  () => unlockedToc.value ?? data.value?.toc ?? [],
)

async function unlock(): Promise<void> {
  unlockedHtml.value = null
  unlockedToc.value = null

  if (!lesson.value?.locked) return

  await resolveReader()
  if (!isSignedIn.value) return

  try {
    const full = await $fetch<{
      html: string
      toc: Heading[]
      unlocked: boolean
    }>(`/api/books/${bookSlug.value}/lessons/${lessonSlug.value}`)

    if (!full.unlocked) return

    unlockedHtml.value = full.html
    unlockedToc.value = full.toc ?? []
  }
  catch {
    // Not entitled, or the api is unreachable. Either way the reader keeps the
    // free half and the paywall, which is the honest thing to show.
    unlockedHtml.value = null
    unlockedToc.value = null
  }
}

/**
 * The whole body the reader may see, free half first.
 *
 * One string rather than two elements so highlights and notes, whose offsets
 * are counted through the rendered text, span the join instead of restarting
 * at it.
 */
/** The body this reader may see: the whole lesson, or the free half. */
const readable = computed(() => unlockedHtml.value ?? data.value?.html ?? '')

/**
 * Which contents entry is highlighted: the section actually being read.
 *
 * Scroll position, not the url. The hash only says where the reader jumped
 * last, so it goes stale the moment they scroll past that section — which is
 * most of the time they spend on the page.
 *
 * The reason this was hash-driven before is real and is handled below: the
 * last heading can never cross a line near the top of the viewport, because
 * there is not a screenful of prose beneath it to scroll. Measuring alone
 * therefore answers `N-1` forever and the last entry never lights. The bottom
 * of the document is the special case that fixes it, and it is three lines.
 */
// Seeded with the first entry: neither a hash nor a scroll offset is sent to
// the server, so this is the only thing it can know. Left empty, the server
// would paint a contents list with nothing lit until hydration.
const activeId = ref<string>(data.value?.toc?.[0]?.id ?? '')

/**
 * Where a heading counts as reached — just past the 24px `scroll-margin-top`
 * the reader's headings carry, so a heading jumped to by its anchor lands on
 * the reading side of the line rather than a pixel above it.
 */
const TOP_LINE = 32

/**
 * The last heading to have crossed the line, or the last heading outright once
 * the page can scroll no further.
 *
 * Walks in document order and stops at the first heading still below the line
 * — everything after it is below too, so there is nothing to gain by
 * measuring the rest.
 */
const syncFromScroll = (): void => {
  const last = toc.value.at(-1)

  if (!last) return

  // Bottom of the document. The final section is on screen and nothing else
  // can be, whatever the measurement below would say.
  const bottom = window.scrollY + window.innerHeight
    >= document.documentElement.scrollHeight - 2

  if (bottom) {
    activeId.value = last.id
    return
  }

  let current = toc.value[0]?.id ?? ''

  for (const item of toc.value) {
    const heading = document.getElementById(item.id)

    if (!heading) continue
    if (heading.getBoundingClientRect().top > TOP_LINE) break

    current = item.id
  }

  activeId.value = current
}

/**
 * One measurement a frame, at most.
 *
 * A scroll event fires far faster than the screen repaints, and every one of
 * them would otherwise read layout — which forces the browser to flush pending
 * style and layout work before it can answer.
 */
let queued = 0

const onScroll = (): void => {
  if (queued) return

  queued = requestAnimationFrame(() => {
    queued = 0
    syncFromScroll()
  })
}

/** The hash, but only when it names a section this lesson actually has. */
const syncFromHash = (): void => {
  const id = decodeURIComponent(window.location.hash.slice(1))
  const known = toc.value.some(item => item.id === id)

  activeId.value = known ? id : (toc.value[0]?.id ?? '')
}

onMounted(() => {
  // The hash first, so a link opened at `#some-heading` is lit before the
  // browser has finished scrolling to it. Scrolling corrects it from there.
  syncFromHash()
  syncFromScroll()

  // Passive: this handler never calls `preventDefault`, and saying so lets the
  // browser scroll without waiting to find out.
  window.addEventListener('scroll', onScroll, { passive: true })
  // A resize moves every heading, and the reader can resize without scrolling.
  window.addEventListener('resize', onScroll, { passive: true })

  // Plain `<a href="#…">` clicks are handled by the browser, not the router,
  // so `route.hash` does not see them — this event does.
  window.addEventListener('hashchange', syncFromHash)

  onBeforeUnmount(() => {
    window.removeEventListener('scroll', onScroll)
    window.removeEventListener('resize', onScroll)
    window.removeEventListener('hashchange', syncFromHash)

    if (queued) cancelAnimationFrame(queued)
  })
})

// A different lesson has different headings, and the hash rarely survives the
// move. Fall back to its first entry rather than keeping the old one lit.
watch(() => data.value?.toc, async () => {
  activeId.value = toc.value[0]?.id ?? ''

  // After the new headings are in the dom — measuring before it would read the
  // outgoing lesson's.
  if (import.meta.client) {
    await nextTick()
    syncFromScroll()
  }
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

/**
 * Whether the browser has taken over.
 *
 * `isSignedIn` reads a cookie, so it is false on the server and true on the
 * first client render for a signed-in reader — and every branch keyed on it
 * then hydrates against markup that says the opposite. This starts false on
 * both sides and flips once mounted, so server and client agree and the
 * signed-in view arrives a tick later instead of as a mismatch.
 *
 * The document is edge-cached and identical for everyone, which is the other
 * reason the server must not render a reader-specific branch into it.
 */
const hydrated = ref(false)
onMounted(() => {
  hydrated.value = true
})

/** Signed in, as far as anything rendered is allowed to know. */
const reading = computed(() => hydrated.value && isSignedIn.value)

const scope = computed(() => ({
  book: bookSlug.value,
  lesson: lessonSlug.value,
}))

const {
  notes,
  loaded: notesLoaded,
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

/* ---------- the thread under the lesson ---------- */
const editingNoteId = ref<number | null>(null)
const savingNoteId = ref<number | null>(null)
const postingComment = ref<boolean>(false)
const commentError = ref<string | null>(null)

/**
 * A note with nothing selected — the comment box rather than the margin.
 *
 * The api takes it: `selected_text` and both offsets are nullable, and a note
 * with no anchor is a thought about the lesson rather than about one sentence
 * of it. It shows in the thread and draws no highlight, because there is no
 * passage to draw it on.
 */
const postComment = async (content: string): Promise<void> => {
  postingComment.value = true
  commentError.value = await createNote(content)
  postingComment.value = false
}

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

// After the free half is in the DOM, and again whenever the reader moves to
// another lesson or their session resolves.
onMounted(unlock)
watch([lessonSlug, isSignedIn], unlock)

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
watch(readable, async () => {
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
    <div class="reader-progress" aria-hidden="true">
      <span :style="{ width: `${data.percent}%` }" />
    </div>

    <div class="reader-layout">
      <aside v-if="toc.length" class="reader-toc" aria-label="on this page">
        <span class="lh-eyebrow">on this page</span>
        <nav class="reader-toc__list">
          <component
            :is="item.locked ? 'span' : 'a'"
            v-for="item in toc"
            :key="item.id"
            class="reader-toc__item"
            :class="{ 'is-active': item.id === activeId, 'is-locked': item.locked }"
            :href="item.locked ? undefined : `#${item.id}`"
          >
            {{ item.text }}
          </component>
        </nav>
      </aside>

      <article class="reader-article">
        <header class="reader-head">
          <p class="lh-eyebrow">
            <NuxtLink :to="`/books/${book.slug}`" class="lh-link">{{ book.title }}</NuxtLink>
            · {{ eyebrow }}
          </p>
          <h1 class="lh-h1">{{ lesson.title }}</h1>
          <p v-if="lesson.description" class="lh-lede">{{ lesson.description }}</p>

          <div class="reader-meta">
            <span class="lh-num">{{ data.readMinutes }} min read</span>
            <span class="lh-num">{{ data.position }} of {{ data.total }}</span>
            <!-- Only once there is one. A control that is present and inert
                 most of the time reads as broken rather than as empty. -->
            <template v-if="currentBookmark">
              <button type="button" @click="goToBookmark">your bookmark →</button>
              <button type="button" @click="removeBookmark">remove bookmark</button>
            </template>
          </div>
        </header>

        <div class="reader-body">
          <!-- eslint-disable-next-line vue/no-v-html -- authored markdown, rendered server side -->
          <div class="lesson-content" data-lesson-content v-html="readable" />
        </div>

        <div v-if="lesson.locked && !unlockedHtml" class="reader-paywall paywalled">
          <hr class="lh-dashed">
          <span class="lh-eyebrow">pro · {{ data.remainingSections }} more sections</span>
          <h2 class="lh-h2">The rest of this lesson is part of {{ book.title }}</h2>
          <p class="lh-sub">
            The remaining sections, and the project that goes with them, come with a plan
            that includes this book.
          </p>
          <div class="reader-paywall__actions">
            <UiButton variant="inverse" size="lg" cta="pro" flame to="/pricing">Get Pro</UiButton>
            <UiButton
              v-if="!isSignedIn"
              variant="ghost"
              size="lg"
              cta="free"
              :to="`/login?redirect=${encodeURIComponent(route.fullPath)}`"
            >
              I already own it
            </UiButton>
          </div>
        </div>

        <nav class="reader-pager" aria-label="lessons">
          <UiButton
            v-if="data.previous"
            variant="ghost"
            size="lg"
            :to="`/books/${book.slug}/pages/${data.previous.slug}`"
          >
            ← {{ data.previous.title }}
          </UiButton>
          <UiButton
            v-if="data.next"
            variant="ghost"
            size="lg"
            :to="`/books/${book.slug}/pages/${data.next.slug}`"
          >
            {{ data.next.title }} →
          </UiButton>
        </nav>

        <ReaderCommentsThread
          :notes="inReadingOrder"
          :loading="!notesLoaded && reading"
          :signed-in="reading"
          :submitting="postingComment"
          :submit-error="commentError"
          :editing-id="editingNoteId"
          :saving-id="savingNoteId"
          @submit="postComment"
          @jump="jumpToNote"
          @edit="editingNoteId = $event.id"
          @save="saveNoteEdit"
          @cancel-edit="editingNoteId = null"
          @remove="removeListedNote"
          @sign-in="signIn"
        />
      </article>
    </div>

    <!-- The three floating pieces belong to the viewport, not the column. -->
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
