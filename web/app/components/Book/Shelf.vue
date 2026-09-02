<script setup lang="ts">
import type { Book } from '@/types/Content'

/**
 * The shelf: every book the caller hands it, filtered by track and rendered as
 * rows.
 *
 * Takes the books rather than fetching them, because the page above already
 * has to — its `useSeo` and its `ItemList` are built from the same list, and a
 * second `useAsyncData` here would be a second cache key for one response.
 * Everything else the shelf needs — which track is showing, what the url says
 * about it, which cover is zoomed — is its own, and no page has to hold it.
 */
const props = defineProps<{ books: Book[] }>()

const books = computed<Book[]>(() => props.books)

/**
 * The track tabs.
 *
 * Named here rather than gathered from the books, so the order of the tabs is
 * a decision and not whatever the first response happened to contain. A track
 * a book claims but this does not name simply gets no tab — the book is still
 * on the shelf under `all`.
 *
 * Books on no track at all — Crack the Interview, Build Your Own Container,
 * C Programming — are not a track of their own. `all` is where a book with no
 * reading order belongs.
 */
const TRACKS = ['go', 'rust', 'systems'] as const

type Track = 'all' | typeof TRACKS[number]

const isTrack = (value: unknown): value is Track =>
  value === 'all' || (TRACKS as readonly string[]).includes(value as string)

const route = useRoute()
const router = useRouter()

/**
 * `?track=` is the tab, so a track is a link somebody can send.
 *
 * Read once for the initial value and written back on every change, which is
 * what makes the back button walk the tabs and `/books?track=systems` open on
 * the one it names. Anything unrecognised — a typo, a track that has been
 * renamed since the link was shared — falls back to `all` rather than showing
 * an empty shelf, because a stale link should still land on the books.
 */
const track = ref<Track>(
  isTrack(route.query.track) ? route.query.track : 'all',
)

watch(track, (chosen) => {
  void router.replace({
    query: chosen === 'all' ? {} : { track: chosen },
  })
})

const onTrack = (book: Book, key: Track): boolean =>
  key === 'all' || book.tracks[key] !== undefined

/** `all` first, then any track with something on it. */
const tabs = computed(() =>
  (['all', ...TRACKS] as const)
    .map(key => ({ key, count: books.value.filter(b => onTrack(b, key)).length }))
    .filter(tab => tab.key === 'all' || tab.count > 0))

/**
 * The search box, off.
 *
 * Kept rather than deleted: it works, and it is wanted back once the shelf is
 * long enough to need it. Typed `boolean` so the template's `v-if` is a switch
 * rather than a constant the compiler folds away.
 */
const searchEnabled: boolean = false

/**
 * Client-side over the list already in hand — there are nine books, and a
 * round trip to filter nine rows is a round trip to save nothing. Plain
 * substring over title and description: no fuzzy matching, because the thing
 * being searched is nine titles a reader can already see.
 */
const query = ref('')

const shown = computed<Book[]>(() => {
  const needle = query.value.trim().toLowerCase()
  const chosen = track.value

  const matching = books.value.filter(book =>
    onTrack(book, chosen)
    && (!needle || `${book.title} ${book.description}`.toLowerCase().includes(needle)),
  )

  // A track is a reading order, so showing one means showing it in that order:
  // Go Fundamentals before Go Intermediate, whatever the catalogue's own order
  // is. `all` keeps the order the api sent, which is the only order a mixed
  // shelf has. Sorted on a copy — `filter` already made one, but saying so
  // beats relying on it.
  if (chosen === 'all') return matching

  return [...matching].sort(
    (a, b) => (a.tracks[chosen] ?? 0) - (b.tracks[chosen] ?? 0),
  )
})

/** "11 books" unfiltered, "4 of 11" once a track or a search narrows it. */
const tally = computed<string>(() => {
  if (books.value.length === 0) return ''
  if (shown.value.length === 0) return 'nothing matches that'
  if (shown.value.length === books.value.length) return `${books.value.length} books`

  return `${shown.value.length} of ${books.value.length}`
})

/**
 * The cover, full size.
 *
 * A native `<dialog>` rather than a div with a z-index: Escape, the backdrop
 * and the focus trap are all already implemented in the browser, and the
 * lightbox is the whole feature.
 */
const lightbox = useTemplateRef<HTMLDialogElement>('lightbox')
const zoomed = ref<Book | null>(null)

function zoom(book: Book): void {
  zoomed.value = book
  lightbox.value?.showModal()
}

/** Only when the backdrop itself was hit — a click on the image is not a
 *  click on the dialog. */
function dismiss(event: MouseEvent): void {
  if (event.target === lightbox.value) lightbox.value?.close()
}

</script>

<template>
  <div class="shelf">
    <!-- One row, search left and tracks right, so the tracks stay right-aligned
         against the slab whether or not the search is rendered. -->
    <div class="controls">
      <label v-if="searchEnabled" class="search">
        <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <circle cx="7" cy="7" r="4.2" stroke="currentColor" stroke-width="1.4" />
          <path d="M10.2 10.2L13.5 13.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
        <input
          v-model="query"
          type="search"
          placeholder="Search books"
          autocomplete="off"
          aria-label="Search books"
        >
      </label>

      <div class="tracks">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          class="tab"
          :class="{ on: track === tab.key }"
          :aria-pressed="track === tab.key"
          @click="track = tab.key"
        >
          {{ tab.key }}<span>{{ tab.count }}</span>
        </button>
      </div>
    </div>

    <div class="mt-10">
      <p v-if="books.length === 0" class="empty">
        No books published yet — the <NuxtLink to="/roadmap">roadmap</NuxtLink>
        says what lands next.
      </p>

      <template v-else>
        <article v-for="book in shown" :key="book.slug" class="bk">
          <button
            v-if="book.thumbnailUrl"
            type="button"
            class="bk-thumb"
            :aria-label="`View the cover of ${book.title}`"
            @click="zoom(book)"
          >
            <img :src="book.thumbnailUrl" :alt="book.title" loading="lazy">
          </button>
          <span v-else class="bk-thumb bk-thumb--none" />

          <NuxtLink :to="`/books/${book.slug}`" class="bk-link">
            <span class="bk-title">{{ book.title }}</span>
            <span class="bk-desc">{{ book.description }}</span>
          </NuxtLink>

          <span class="bk-meta">
            <span>{{ book.pages }} pages</span>
            <span v-if="book.price" class="bk-price">{{ book.price }}</span>
          </span>
        </article>

        <p v-if="shown.length === 0" class="empty">Nothing matches that.</p>
      </template>
    </div>

    <p class="foot">{{ tally }}</p>

    <dialog ref="lightbox" class="lb" @click="dismiss">
      <img v-if="zoomed?.thumbnailUrl" :src="zoomed.thumbnailUrl" :alt="zoomed.title">
      <p class="lb-cap">{{ zoomed?.title }}</p>
    </dialog>
  </div>
</template>

<style scoped>
/* Written as css rather than utilities for the same reason the book page is:
 * 13px rows, a 300px cover gutter and a 10px mono meta line are all off the
 * scale, and as arbitrary values in brackets the markup stops reading. */

.shelf {
    font-size: 13px;
    line-height: 1.6;
    color: var(--color-read-ink);
}

/* Two columns even with one occupant: the tracks are right-aligned against the
 * slab, and a flex row would slide them left the moment the search is off. */
.controls {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 36px;
    align-items: center;
    margin-top: 48px;
}

.tracks {
    display: flex;
    gap: 2px;
    justify-content: flex-end;
    flex-wrap: wrap;
    grid-column: 2;
}

.tab {
    font-family: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace;
    font-size: 10.5px;
    letter-spacing: 0.04em;
    color: var(--color-read-mute);
    background: none;
    border: none;
    padding: 6px 11px;
    border-radius: 4px;
    cursor: pointer;
    transition:
        color 140ms,
        background 140ms;
}

.tab:hover {
    color: var(--color-teal-deep);
}

.tab.on {
    color: var(--color-read-ink);
    background: var(--color-read-bg-soft);
}

.tab span {
    color: var(--color-read-faint);
    margin-left: 5px;
    font-size: 9.5px;
}

.search {
    position: relative;
}

.search svg {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 13px;
    height: 13px;
    color: var(--color-read-faint);
}

.search input {
    width: 100%;
    border: none;
    background: none;
    font-size: 13px;
    color: var(--color-read-ink);
    padding: 8px 0 8px 22px;
    outline: none;
}

.search input::placeholder {
    color: var(--color-read-faint);
}

.bk {
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr) auto;
    gap: 32px;
    align-items: start;
    /* Uniform, with the negative margin cancelling it exactly — so the hover
     * band sits the same distance off the content on all four sides, and the
     * content itself still lines up with the masthead above. */
    padding: 22px;
    margin: 0 -22px;
    border-radius: 5px;
    transition: background 140ms;
}

.bk:hover {
    background: var(--color-read-bg-soft);
}

/* Landscape and `contain`, not a portrait tile and `cover`. The covers are
 * drawn wide, with the title lettering running edge to edge — cropped to a
 * 3/4 box they lose the first and last word of their own name. Letterboxing a
 * short one is the cheaper failure. No fill behind it, so a transparent cover
 * sits on the panel rather than on a grey card. */
.bk-thumb {
    width: 300px;
    aspect-ratio: 16 / 10;
    border-radius: 3px;
    overflow: hidden;
    background: none;
    padding: 0;
    border: none;
    display: block;
    cursor: zoom-in;
    transition: transform 220ms cubic-bezier(0.2, 0.75, 0.25, 1);
}

.bk-thumb--none {
    cursor: default;
}

.bk-thumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}

.bk:hover .bk-thumb {
    transform: translateY(-3px);
}

.bk-link {
    display: block;
    min-width: 0;
}

.bk-title {
    display: block;
    font-family: 'Newsreader', Georgia, serif;
    font-optical-sizing: auto;
    font-weight: 600;
    font-size: 24px;
    line-height: 1.2;
    transition: color 140ms;
}

.bk:hover .bk-title {
    color: var(--color-teal-deep);
}

/* The lesson's voice — Newsreader on read-ink, as `.reader-prose` sets it —
 * stepped down from its 19px to 16px. A blurb is a caption under a title, not
 * the thing being read, and at the reader's own size it competed with the
 * title beside it. Not shared with reader.css: that file is one system scoped
 * under `.reader-shell` and deliberately not imported globally, so borrowing
 * four declarations would mean shipping ~20kb to this route. */
.bk-desc {
    display: block;
    margin-top: 10px;
    font-family: 'Newsreader', Georgia, serif;
    font-optical-sizing: auto;
    font-size: 16px;
    line-height: 1.72;
    color: var(--color-read-ink-soft);
    max-width: 34em;
    text-wrap: pretty;
}

.bk-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    font-family: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: var(--color-read-faint);
    padding-top: 7px;
    white-space: nowrap;
}

.bk:hover .bk-meta {
    color: var(--color-read-mute);
}

.bk-price {
    color: var(--color-amber);
    background: var(--color-amber-soft);
    padding: 2px 7px;
    border-radius: 20px;
}

.empty {
    font-family: 'Newsreader', Georgia, serif;
    font-optical-sizing: auto;
    font-size: 14px;
    font-style: italic;
    color: var(--color-read-faint);
    padding: 22px 0 0 8px;
    margin: 0;
}

.empty a {
    color: var(--color-teal-deep);
    text-decoration: underline;
    text-underline-offset: 2px;
}

.foot {
    font-family: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace;
    font-size: 10px;
    color: var(--color-read-faint);
    margin-top: 34px;
    text-align: center;
}

.lb {
    margin: auto;
    max-width: 88vw;
    max-height: 88vh;
    border: none;
    padding: 0;
    background: none;
    overflow: visible;
    cursor: zoom-out;
}

.lb::backdrop {
    background: rgb(18 16 12 / 86%);
}

.lb img {
    max-width: 88vw;
    max-height: 78vh;
    border-radius: 3px;
    box-shadow: 0 24px 70px rgb(0 0 0 / 50%);
    display: block;
}

.lb-cap {
    margin: 14px 0 0;
    text-align: center;
    font-family: 'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace;
    font-size: 10.5px;
    letter-spacing: 0.04em;
    color: rgb(255 255 255 / 62%);
}

@media (max-width: 900px) {
    .controls {
        grid-template-columns: minmax(0, 1fr);
        gap: 14px;
    }

    .tracks {
        grid-column: 1;
        justify-content: flex-start;
        margin-left: -11px;
    }

    .bk {
        grid-template-columns: minmax(0, 1fr);
        gap: 14px;
    }

    .bk-thumb {
        width: 100%;
        max-width: 340px;
    }

    .bk-title {
        font-size: 21px;
    }

    .bk-desc {
        font-size: 15px;
    }

    .bk-meta {
        flex-direction: row;
        align-items: center;
        gap: 10px;
        padding-top: 2px;
    }
}
</style>
