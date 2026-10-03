<script setup lang="ts">
import type { Book } from '@/types/Content'

/**
 * The `books` nav item, and the shelf it opens.
 *
 * A panel under the nav rather than a jump to `/books`: the shelf is the thing
 * most people came for, and making them load a page to find out what is on it
 * costs a navigation before they can choose. `/books` is still one click away.
 *
 * **Tracks are tabs over the list**, the same shape `/books` uses. Picking one
 * sorts the books into that track's reading order rather than only filtering
 * them — the order is the part of a track worth having, and it is why
 * `book.yaml` holds `{go: 4}` rather than `[go]`.
 *
 * **The books are fetched on first open, not during SSR.** This sits in the
 * layout, so a `useAsyncData` here would fetch the whole shelf on every page a
 * reader loads, to fill a panel most of them never open. `useState` holds the
 * answer across navigations, so it happens once a session at most. Nothing here
 * is a crawler's business either — `/books` is the indexable shelf, and this is
 * a menu.
 */
const open = ref<boolean>(false)
const root = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)

/**
 * The panel is teleported to `<body>`, so it is no longer inside `root` — and
 * the outside-click test has to ask it directly.
 */
const panel = ref<HTMLElement | null>(null)

const books = useState<Book[]>('books-menu', () => [])
const pending = ref<boolean>(false)
const failed = ref<boolean>(false)

/**
 * The tracks, named rather than gathered from the books: the order of the
 * column is a choice, not whatever the first response happened to contain.
 *
 * `foundation`, not `systems` — the content renamed the track. Anything still
 * naming the old one shows no tab at all, because a tab with a count of zero
 * is filtered out, which is how it went unnoticed on the home shelf.
 */
const TRACKS = ['go', 'rust', 'foundation'] as const

type Track = 'all' | typeof TRACKS[number]

const track = ref<Track>('all')

const onTrack = (book: Book, key: Track): boolean =>
  key === 'all' || book.tracks[key] !== undefined

/** `all` first, then any track with something on it. */
const tabs = computed(() =>
  (['all', ...TRACKS] as const)
    .map(key => ({ key, count: books.value.filter(b => onTrack(b, key)).length }))
    .filter(tab => tab.key === 'all' || tab.count > 0))

/**
 * The books for the chosen tab, in that track's order.
 *
 * `all` keeps the catalogue's own order — there is no reading order across the
 * whole shelf, and inventing one would be a claim the content does not make.
 */
const shown = computed<Book[]>(() => {
  const key = track.value

  if (key === 'all') return books.value

  return books.value
    .filter(book => book.tracks[key] !== undefined)
    .sort((a, b) => (a.tracks[key] ?? 0) - (b.tracks[key] ?? 0))
})

/**
 * The book the preview column is showing.
 *
 * Held by slug rather than by object, so it survives the list being refetched
 * — a stale object reference would keep painting a book that is no longer the
 * one in that row.
 */
const hovered = ref<string | null>(null)

/**
 * Falls back to the first book rather than to nothing.
 *
 * An empty third column on open is a hole the reader has to hover to fill, and
 * it makes the panel change width the first time they do. Showing the first
 * book means the column is doing its job before it is asked.
 */
const preview = computed<Book | null>(() =>
  shown.value.find(book => book.slug === hovered.value) ?? shown.value[0] ?? null)

/**
 * Whether the list has more below the fold, and whether the reader is already
 * at the bottom of it.
 *
 * A list that scrolls with no sign that it does reads as a list of five books.
 * Both flags are measured rather than assumed: `overflowing` so a short shelf
 * shows no hint at all, and `atEnd` so the hint goes away once it has been
 * acted on instead of pointing at nothing.
 */
const list = ref<HTMLElement | null>(null)
const overflowing = ref<boolean>(false)
const atEnd = ref<boolean>(false)

function measure(): void {
  const el = list.value

  if (!el) {
    overflowing.value = false
    return
  }

  // A pixel of slack: `scrollHeight` and `clientHeight` can land a fraction
  // apart on a fractional device pixel ratio, which would leave a hint showing
  // on a list with nothing under it.
  overflowing.value = el.scrollHeight > el.clientHeight + 1
  atEnd.value = el.scrollTop + el.clientHeight >= el.scrollHeight - 1
}

/** Re-measured when the panel opens and when the books land — the list has no
 *  height worth measuring until both have happened. */
watch([open, books, track], () => void nextTick(measure))

async function ensureBooks(): Promise<void> {
  if (books.value.length || pending.value) return

  pending.value = true
  failed.value = false

  try {
    books.value = await $fetch<Book[]>('/_api/books')
  }
  catch {
    failed.value = true
  }
  finally {
    pending.value = false
  }
}

function toggle(): void {
  open.value = !open.value

  if (open.value) void ensureBooks()
}

const close = (restoreFocus = false): void => {
  open.value = false

  // Only when dismissed from the keyboard. Yanking focus back after a click
  // outside would fight whatever the reader just clicked on.
  if (restoreFocus) trigger.value?.focus()
}

const onPointerDown = (event: MouseEvent): void => {
  if (!open.value) return
  if (root.value?.contains(event.target as Node)) return
  if (panel.value?.contains(event.target as Node)) return

  close()
}

const onKeydown = (event: KeyboardEvent): void => {
  if (open.value && event.key === 'Escape') close(true)
}

onMounted(() => {
  // `pointerdown`, not `click`: a click that starts inside the panel and ends
  // outside it should not count as dismissing the panel.
  document.addEventListener('pointerdown', onPointerDown)
  document.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onPointerDown)
  document.removeEventListener('keydown', onKeydown)
})

// A panel left hanging over the next page is a panel the reader has to dismiss
// twice.
const route = useRoute()
watch(() => route.fullPath, () => close())
</script>

<template>
  <div ref="root" class="relative">
    <button
      ref="trigger"
      type="button"
      class="inline-flex cursor-pointer items-center gap-1.5 px-4 py-1 font-sans text-xs text-ink transition hover:text-link-hover sm:text-sm"
      aria-haspopup="dialog"
      :aria-expanded="open"
      @click="toggle"
    >
      books
      <svg
        class="h-1.5 w-2.5 transition-transform duration-150"
        :class="open ? 'rotate-180' : ''"
        viewBox="0 0 10 6"
        fill="none"
        aria-hidden="true"
      >
        <path d="M1 1l4 4 4-4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
    </button>

    <!--
      Teleported, and that is what makes the panel sit on the page at all.

      `position: fixed` resolves against the viewport only while no ancestor
      carries a `transform` — and the nav above this is `-translate-x-1/2`
      inside a fixed header, which makes it the containing block for anything
      fixed inside it. Out at `<body>` there is no transform to inherit.
    -->
    <Teleport to="body">
      <!-- Opacity only, no transform: a translate utility on the enter class
           would replace the panel's own centring rather than compose with it. -->
      <Transition
        enter-active-class="transition-opacity duration-150 ease-out"
        leave-active-class="transition-opacity duration-150 ease-in"
        enter-from-class="opacity-0"
        leave-to-class="opacity-0"
      >
        <!--
          Sized by insets: `inset-x-6` leaves a gutter either side, `max-w-shelf`
          caps it on a wide monitor and `mx-auto` centres what is left, so no
          measurement here is a bracket value. `top-20` clears the `h-16` header,
          and `max-h-shelf` stops it short of the bottom of the window — see
          `theme.css` for why that is not `max-h-full`.

          **`h-128`, not `h-fit`.** Fitting the content meant the preview column
          sized the panel, so the whole thing grew and shrank as the pointer
          moved down the list — a longer blurb, a title that wrapped to two
          lines, or a book with no track badges each changed the height. A
          stated height makes hovering free: the list takes whatever is left
          over and scrolls inside it, and the preview is clipped rather than
          sizing anything.
        -->
        <div
          v-if="open"
          ref="panel"
          class="max-h-shelf fixed inset-x-6 top-20 z-50 mx-auto h-128 max-w-shelf"
          role="dialog"
          aria-label="Books"
        >
          <!--
            The pencil border lives on this inner element, not on the positioned
            one, and that split is load bearing.

            `.border-pencil-light` sets `position: relative` so its `::before`
            can anchor — and it is one class, exactly like `.fixed`, so the two
            are decided by source order. `pencil.css` is imported after
            tailwind, so it won: the panel was `relative`, sat in `<body>`'s
            normal flow, and pushed the whole page down. Keeping the two on
            separate elements means neither has to beat the other.
          -->
          <div class="border-pencil-light flex h-full flex-col overflow-hidden rounded-xl bg-panel shadow-2xl">
          <p v-if="pending" class="p-8 text-sm text-quiet">Loading the shelf…</p>

          <p v-else-if="failed" class="p-8 text-sm text-quiet">
            The shelf could not be loaded.
            <button type="button" class="cursor-pointer underline underline-offset-2" @click="ensureBooks">
              Try again
            </button>
          </p>

          <p v-else-if="books.length === 0" class="p-8 text-sm text-quiet">
            No books published yet.
          </p>

          <!--
            Flex rather than grid, so the preview's width is a scale value —
            `md:w-72` — instead of a bracketed grid template. The books take
            what is left, and are the column that should give up room when
            there is not enough.

            One rule between the two columns rather than a border on each, so
            the preview does not carry a line against the panel's edge.
          -->
          <div v-else class="flex min-h-0 flex-col divide-rule p-6 md:flex-row md:divide-x">
            <!--
              The books, and the only part that scrolls: `min-h-0` because a
              flex item will not shrink below its content without it, which is
              what made them spill out of the panel before.
            -->
            <section class="flex min-h-0 min-w-0 flex-1 flex-col md:pr-6">
              <header class="mb-4 flex items-center justify-between gap-3">
                <span class="flex items-baseline gap-2">
                  <h2 class="font-serif text-base text-ink">Books</h2>
                  <span class="font-mono text-xs text-read-faint">{{ shown.length }}</span>
                </span>

                <!--
                  The one filled button in the panel, so it reads as the action
                  rather than as another link — the shape `.btn-write` on the
                  blog already uses. Named group, so the arrow answers this
                  button's hover and not a book row's.
                -->
                <NuxtLink
                  to="/books"
                  class="group/all inline-flex shrink-0 items-center gap-1.5 rounded-full bg-ink px-4 py-1.5 font-sans text-xs whitespace-nowrap text-on-ink shadow-sm transition duration-150 hover:-translate-y-px hover:bg-ink-hover hover:shadow motion-reduce:transition-none motion-reduce:hover:translate-y-0"
                >
                  view all books
                  <!-- The slide is decoration; anybody who asked not to be
                       moved gets the colour and the lift dropped with it. -->
                  <span
                    class="transition-transform duration-150 group-hover/all:translate-x-1 motion-reduce:transition-none motion-reduce:group-hover/all:translate-x-0"
                    aria-hidden="true"
                  >→</span>
                </NuxtLink>
              </header>

              <!--
                The tabs the books page uses, in the same shape: `all` and any
                track with something on it, each carrying its count. Picking one
                sorts into that track's reading order, which is the part of a
                track worth having — a filter would only hide books.
              -->
              <nav class="mb-3 flex flex-wrap gap-1">
                <button
                  v-for="tab in tabs"
                  :key="tab.key"
                  type="button"
                  class="flex cursor-pointer items-baseline gap-1.5 rounded-md px-2.5 py-1 font-mono text-xs transition"
                  :class="track === tab.key
                    ? 'bg-read-bg-soft text-ink'
                    : 'text-quiet hover:bg-read-bg-soft hover:text-ink'"
                  :aria-pressed="track === tab.key"
                  @click="track = tab.key"
                >
                  {{ tab.key }}<span class="text-read-faint">{{ tab.count }}</span>
                </button>
              </nav>

              <div class="relative min-h-0 flex-1">
                <div
                  ref="list"
                  class="grid h-full grid-cols-1 gap-1 overflow-y-auto pr-2"
                  @scroll.passive="measure"
                >
                <NuxtLink
                  v-for="book in shown"
                  :key="book.slug"
                  :to="`/books/${book.slug}`"
                  class="hover-border-pencil group flex items-center gap-3 rounded-md px-4 py-2"
                  @mouseenter="hovered = book.slug"
                  @focus="hovered = book.slug"
                >
                  <!-- `contain`, because the covers are drawn wide with the
                       lettering running edge to edge and a crop eats it. -->
                  <span class="size-16 shrink-0 overflow-hidden rounded">
                    <img
                      v-if="book.thumbnailUrl"
                      :src="book.thumbnailUrl"
                      alt=""
                      width="128"
                      height="128"
                      loading="lazy"
                      class="size-full object-contain"
                    >
                  </span>

                  <!-- The title alone. Pages and price are in the preview
                       column for whichever book the reader is pointing at, and
                       eleven copies of them here is a second column of numbers
                       nobody reads down. -->
                  <span class="min-w-0 truncate text-sm font-light text-ink transition group-hover:text-teal-deep">
                    {{ book.title }}
                  </span>
                </NuxtLink>
                </div>

                <!-- The list fading out under its own last visible row, so the
                     cut looks like more rather than like an edge. Inert, and
                     gone once there is nothing left to scroll to. -->
                <div
                  v-show="overflowing && !atEnd"
                  class="pointer-events-none absolute inset-x-0 bottom-0 h-10 bg-gradient-to-t from-panel to-transparent"
                  aria-hidden="true"
                />
              </div>

              <!-- Said out loud as well as drawn: the fade reads as depth to
                   people who already know the pattern and as nothing to
                   everyone else. -->
              <p
                v-show="overflowing && !atEnd"
                class="pt-2 text-center font-mono text-xs text-read-faint"
              >
                scroll for more ↓
              </p>
            </section>

            <!--
              The preview, following whatever the reader is pointing at.

              `hidden md:block`, because it answers a hover and a touch screen
              has none — on a phone it would be a third of the panel showing a
              book nobody asked about. Keyboard users reach it through `@focus`
              on the rows, which is the same gesture arrowing down the list.
            -->
            <aside v-if="preview" class="hidden overflow-hidden md:block md:w-72 md:shrink-0 md:pl-6">
              <header class="mb-4 flex items-center gap-3">
                <h2 class="font-serif text-base text-ink">Preview</h2>
              </header>

              <NuxtLink :to="`/books/${preview.slug}`" class="group block">
                <span class="block h-32 overflow-hidden rounded">
                  <img
                    v-if="preview.thumbnailUrl"
                    :src="preview.thumbnailUrl"
                    :alt="preview.title"
                    width="320"
                    height="200"
                    class="size-full object-contain"
                  >
                </span>

                <span class="mt-4 block font-serif text-lg leading-tight text-ink transition group-hover:text-teal-deep">
                  {{ preview.title }}
                </span>

                <span class="mt-3 line-clamp-6 block font-serif text-sm leading-relaxed text-read-ink-soft">
                  {{ preview.description }}
                </span>

                <span class="mt-4 flex items-center gap-2 font-mono text-xs text-read-faint">
                  <span>{{ preview.pages }} pages</span>
                  <span v-if="preview.price" class="text-amber">{{ preview.price }}</span>
                  <span v-else class="text-teal-deep">free</span>
                </span>

                <span class="mt-4 block font-mono text-xs text-quiet transition group-hover:text-ink">
                  read book ———→
                </span>
              </NuxtLink>
            </aside>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
