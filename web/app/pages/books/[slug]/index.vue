<script setup lang="ts">
import type { Book, Chapter, LessonSummary, TocSection } from '@/types/Content'
import { bookTopics } from '@/data/BookTopics'

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

const sections = computed<TocSection[]>(() => chapters.value.map((chapter, i) => ({
  key: chapter.id,
  eyebrow: `chapter ${String(i + 1).padStart(2, '0')}`,
  title: chapter.title,
  rows: lessons.value
    .filter(lesson => lesson.chapterId === chapter.id)
    .map(lesson => ({
      n: numberOf(lesson),
      title: lesson.title,
      blurb: lesson.description,
      to: `/books/${slug.value}/pages/${lesson.slug}`,
      locked: lesson.locked,
    })),
})))

/**
 * "book 06 — go · rust": where the book sits on the shelf, and the tracks
 * that carry it. The shelf's order is the api's, through the same listing
 * the books page renders.
 */
const { data: shelf } = await useShelf()

const eyebrow = computed<string>(() => {
  const at = (shelf.value ?? []).findIndex(b => b.slug === slug.value)
  const number = at === -1 ? 'book' : `book ${String(at + 1).padStart(2, '0')}`
  const onTracks = Object.keys(book.value?.tracks ?? {})

  return onTracks.length ? `${number} — ${onTracks.join(' · ')}` : number
})

const topics = computed<string[]>(() => bookTopics[slug.value] ?? [])

// Get Pro only where there is something to unlock, and never to a reader who
// already has it.
const { load: loadAccess, owns } = useAccess()
onMounted(loadAccess)

const hasPro = computed<boolean>(() => lessons.value.some(lesson => lesson.locked))
const showPro = computed<boolean>(() => hasPro.value && !owns(slug.value))

/**
 * What the cover plate shows.
 *
 * `images` is the book's own list and the thumbnail is the fallback, so a book
 * whose yaml has no `images:` yet still shows its cover rather than a gap.
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

// Straight to checkout for the plan on sale; sign-in first when signed out.
const { getPro, busy: proBusy, reason: proReason } = useGetPro()
</script>

<template>
  <div v-if="book" class="book">
    <DetailHead
      :eyebrow="eyebrow"
      :title="book.title"
      :description="book.description"
      :topics="topics"
      :cover="covers[0]"
      :covers="covers"
      :cover-alt="`${book.title} cover`"
    >
      <template v-if="firstLesson || showPro" #actions>
        <UiButton
          v-if="firstLesson"
          variant="inverse"
          size="lg"
          cta="pro"
          :to="`/books/${book.slug}/pages/${firstLesson}`"
        >
          Start reading →
        </UiButton>
        <!-- `owns` is false until the browser asks, so the server and the
             first client render agree and the button only ever disappears. -->
        <UiButton v-if="showPro" variant="ghost" size="lg" cta="free" flame :disabled="proBusy" @click="getPro">
          Get Pro
        </UiButton>
        <p v-if="proReason" class="lh-error" role="alert">{{ proReason }}</p>
      </template>
    </DetailHead>

    <div id="toc" class="lh-figure toc-wrap">
      <!-- A published book whose lessons are all still drafts. The api sends
           no chapters for one, so without this the page ends at the head and
           reads as broken rather than as early. -->
      <p v-if="isEmpty" class="lh-sub empty">
        No lessons published yet — this one is still being written. The
        <NuxtLink to="/roadmap" class="lh-inline">roadmap</NuxtLink> says what lands next.
      </p>

      <TocList v-else :sections="sections" />
    </div>

    <div class="end" aria-hidden="true">
      <img src="/lighthouse.svg" alt="" width="20" height="20">
    </div>
  </div>
</template>

<style scoped>
.toc-wrap { margin-top: var(--space-24); }

.empty { padding: 0 var(--space-4); }

.end {
  display: flex;
  justify-content: center;
  margin-top: var(--space-24);
}

.end img { width: 20px; height: 20px; opacity: 0.35; }
</style>
