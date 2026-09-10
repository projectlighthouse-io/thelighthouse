<script setup lang="ts">
import type { Book, Chapter, LessonSummary } from '@/types/Content'

interface BookDetailResponse {
  book: Book
  chapters: Chapter[]
  lessons: LessonSummary[]
}

const route = useRoute()
const slug = computed<string>(() => String(route.params.slug))

// From ohara, through the rust api. During SSR this calls the handler directly,
// so it costs no HTTP round trip.
const { data, error } = await useAsyncData(
  () => `book:${slug.value}`,
  () => $fetch<BookDetailResponse>(`/_api/books/${slug.value}`),
  { watch: [slug] },
)

/**
 * Why the failure is read before the absence.
 *
 * `useAsyncData` does not throw — a request that failed leaves `data` null and
 * puts the reason in `error`. Checking only `data` therefore reports an api
 * that is down, a 500, or a timeout as "book not found", which sends
 * whoever reads it looking for missing content that is not missing.
 *
 * A rejection with no status is nitro never reaching the api at all, and 502 is
 * what that is: this process is the gateway, and its upstream did not answer.
 */
if (error.value) {
  const status = error.value.statusCode ?? 502
  const missing = status === 404

  throw createError({
    statusCode: status,
    statusMessage: missing ? 'Book not found' : 'The api is not answering',
    // Copy for this page's two failures, read by `error.vue` when it is
    // there. A 404 here is a book that is not on the shelf, which is a
    // different sentence from a route that does not exist, and a 5xx is the
    // catalogue being down — worth saying, because it is worth retrying.
    data: missing
      ? {
          headline: 'not on this shelf',
          detail: `There is no book at /books/${slug.value}. It may have been renamed, or never made it out of drafts.`,
        }
      : {
          headline: 'the catalogue is down',
          detail: 'The site is up; the service that knows what is in the books is not answering. Nothing is lost — it is worth trying again.',
          retry: true,
        },
    fatal: true,
  })
}

// Past the error check, so this is a genuinely empty answer.
if (!data.value) {
  throw createError({ statusCode: 404, statusMessage: 'Book not found', fatal: true })
}

const book = computed(() => data.value?.book)
const lessons = computed<LessonSummary[]>(() => data.value?.lessons ?? [])
const chapters = computed<Chapter[]>(() => data.value?.chapters ?? [])

const lessonsFor = (chapter: Chapter): LessonSummary[] =>
  lessons.value.filter(l => l.chapterId === chapter.id)

/** running lesson number across the whole book, not per chapter */
const numberOf = (lesson: LessonSummary): string =>
  String(lessons.value.indexOf(lesson) + 1).padStart(2, '0')

/**
 * Where "start reading" goes.
 *
 * The api's own `first_lesson` rather than `lessons[0]`, because the two are
 * answers to different questions: one is the first *published* lesson, the
 * other is whatever survived into this response. They agree today. The
 * fallback covers the older listing shape, which does not send the field, and
 * null means a book with nothing published — a page with no button, not a
 * button pointing at `/books/x/pages/undefined`.
 */
const firstLesson = computed<string | null>(
  () => book.value?.firstLesson ?? lessons.value[0]?.slug ?? null,
)

/** A book whose lessons are all still drafts. Renders a note, not a blank. */
const isEmpty = computed<boolean>(() => lessons.value.length === 0)

/**
 * What the slideshow shows.
 *
 * `images` is the book's own list and the thumbnail is the fallback, so a book
 * whose yaml has no `images:` yet still shows its cover rather than a gap. A
 * book with neither shows nothing at all — the component renders no frame.
 */
const covers = computed<string[]>(() => {
  const listed = book.value?.images ?? []
  if (listed.length) return listed

  return book.value?.thumbnailUrl ? [book.value.thumbnailUrl] : []
})

useSeo(() => ({
  title: `${book.value?.title} — projectlighthouse`,
  description: book.value?.description ?? '',
  image: book.value?.thumbnailUrl,
}))

useJsonLd('book', () => ({
  '@type': 'Book',
  'name': book.value?.title,
  'description': book.value?.description,
  'image': book.value?.thumbnailUrl,
  'url': `${SITE.url}/books/${slug.value}`,
  'bookFormat': 'https://schema.org/EBook',
  'numberOfPages': lessons.value.length,
  'inLanguage': 'en',
  'author': { '@type': 'Person', 'name': 'Aryan Ahmed' },
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
  'hasPart': chapters.value.map(c => ({ '@type': 'Chapter', 'name': c.title })),
}))

useJsonLd('crumbs', () => ({
  '@type': 'BreadcrumbList',
  'itemListElement': [
    { '@type': 'ListItem', 'position': 1, 'name': 'Books', 'item': `${SITE.url}/books` },
    { '@type': 'ListItem', 'position': 2, 'name': book.value?.title },
  ],
}))
</script>

<template>
  <div v-if="book" class="book mx-auto max-w-[1040px] bg-panel">
    <!-- `bg-panel`, not `bg-white`: the token is #ffffff in light and the dark
         panel in dark, so this slab inverts with the theme rather than staying
         a sheet of white on a dark page.

         The breadcrumb that used to sit above the title is gone. `/books` is
         one click away in the nav on every viewport, and the trail was a mono
         line of chrome above a display face that has to be the first thing
         read. The BreadcrumbList in the script stays: search results still
         want the trail, and that markup is what they read, not this. -->
    <header class="px-10 pt-11 max-[820px]:px-6 max-[820px]:pt-10">
      <!-- One column until 1080px, then the images take a fixed 232px beside
           the title. Fixed rather than fractional so the measure of the dek is
           set by the dek, not by how wide the window happens to be. -->
      <div
        class="grid grid-cols-1 items-start gap-14"
        :class="covers.length ? 'min-[1081px]:grid-cols-[minmax(0,1fr)_232px]' : ''"
      >
        <div>
          <h1 class="masthead-title">
            {{ book.title }}
          </h1>

          <p class="masthead-dek">
            {{ book.description }}
          </p>

          <div v-if="firstLesson" class="mt-6">
            <NuxtLink
              :to="`/books/${book.slug}/pages/${firstLesson}`"
              class="start"
            >
              Start reading
              <svg viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path
                  d="M3 8h9M8.5 4l4 4-4 4"
                  stroke="currentColor"
                  stroke-width="1.4"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </NuxtLink>
          </div>
        </div>

        <!-- The book's own images, not its thumbnail: `images` is what it has to
             show, and falls back to the one image every book has. -->
        <BookCoverSlideshow :images="covers" :title="book.title" />
      </div>
    </header>

    <!-- 720px, and left under the title rather than centred. The rows are a
         numbered list read top to bottom, so their left edge lines up with the
         masthead's; centring them would set the whole page adrift of the one
         vertical the title establishes. -->
    <main class="mt-[34px] max-w-[720px] px-10 pb-[90px] max-[820px]:px-6">
      <!-- A published book whose lessons are all still drafts. The api sends
           no chapters for one, so without this the page ends at the hero and
           reads as broken rather than as early. -->
      <p v-if="isEmpty" class="empty">
        No lessons published yet — this one is still being written. The
        <NuxtLink to="/roadmap">roadmap</NuxtLink> says what lands next.
      </p>

      <!-- No `v-if` on the rows: the api builds a chapter only from the
           lessons it has, so a chapter that reaches here always has some. -->
      <section v-for="chapter in chapters" v-else :key="chapter.id" class="part">
        <h2 class="part-title">
          {{ chapter.title }}
        </h2>

        <NuxtLink
          v-for="lesson in lessonsFor(chapter)"
          :key="lesson.slug"
          :to="`/books/${book.slug}/pages/${lesson.slug}`"
          class="ch"
        >
          <span class="ch-no">{{ numberOf(lesson) }}</span>
          <span>
            <span class="ch-title">
              {{ lesson.title }}
              <span class="pill" :class="lesson.locked ? 'pill-paid' : 'pill-free'">
                {{ lesson.locked ? 'paid' : 'free' }}
              </span>
            </span>
            <span v-if="lesson.description" class="ch-desc">
              {{ lesson.description }}
            </span>
          </span>
        </NuxtLink>
      </section>
    </main>
  </div>
</template>

<style scoped>
/* What is left here is this page's own: the start button and the empty state.
 * The chapter and lesson rows moved to `assets/css/contents.css` when the
 * project page started drawing the same list. */

.start {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    border-radius: 6px;
    padding: 9px 15px;
    font-size: 12.5px;
    font-weight: 500;
    background: var(--color-read-ink);
    color: var(--color-on-ink);
    transition:
        background 140ms,
        color 140ms;
}

.start:hover {
    background: var(--color-teal-deep);
}

.start svg {
    width: 13px;
    height: 13px;
}

.empty {
    font-family: 'Newsreader', Georgia, serif;
    font-optical-sizing: auto;
    font-size: 15.5px;
    line-height: 1.55;
    color: var(--color-read-mute);
    border-top: 1px solid var(--color-read-line);
    padding-top: 18px;
    margin: 0;
}

.empty a {
    color: var(--color-teal-deep);
    text-decoration: underline;
    text-underline-offset: 2px;
}

</style>
