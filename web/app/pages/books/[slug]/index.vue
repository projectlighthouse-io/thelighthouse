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

  throw createError({
    statusCode: status,
    statusMessage:
      status === 404 ? 'Book not found' : 'The api is not answering',
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

const firstLesson = computed<LessonSummary | undefined>(() => lessons.value[0])
const hasLocked = computed<boolean>(() => lessons.value.some(l => l.locked))

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
  <div v-if="book" class="mx-auto max-w-7xl bg-panel px-4 sm:px-6 lg:px-8">
    <!-- `bg-panel`, not `bg-white`: the token is #ffffff in light and the dark
         panel in dark, so this slab inverts with the theme rather than staying
         a sheet of white on a dark page. -->
    <nav class="pt-10 pb-8 font-mono text-sm text-faint">
      <NuxtLink to="/books" class="hover:text-ink">books</NuxtLink>
      <span class="mx-3 text-crumb">/</span>
      <span class="text-quiet">{{ book.title.toLowerCase() }}</span>
    </nav>

    <section class="grid gap-12 pb-16 lg:grid-cols-[1fr_420px] lg:items-start">
      <div>
        <!-- Set as the reader sets a lesson title — see masthead.css. -->
        <h1 class="masthead-title">
          {{ book.title }}
        </h1>

        <p class="masthead-dek">
          {{ book.description }}
        </p>

        <div class="mt-10 flex flex-col items-stretch gap-3 sm:flex-row sm:items-center sm:gap-4">
          <NuxtLink
            v-if="firstLesson"
            :to="`/books/${book.slug}/pages/${firstLesson.slug}`"
            class="rounded-md bg-ink px-5 py-3 text-center text-base font-medium text-on-ink transition hover:bg-ink-hover sm:w-auto"
          >
            Start reading
          </NuxtLink>
          <NuxtLink
            v-if="hasLocked"
            to="/pricing"
            class="rounded-md border border-stroke bg-panel px-5 py-3 text-center text-base font-medium text-ink transition hover:bg-paper-warm sm:w-auto"
          >
            Unlock the whole book
          </NuxtLink>
        </div>
      </div>

      <!-- The book's own images, not its thumbnail: `images` is what it has to
           show, and falls back to the one image every book has. -->
      <BookCoverSlideshow :images="covers" :title="book.title" />
    </section>

    <section class="grid gap-12 pb-20 lg:grid-cols-3 lg:items-start">
      <div class="min-w-0 lg:col-span-2">
        <div v-for="chapter in chapters" :key="chapter.id" class="mb-16 last:mb-0">
          <header class="mb-6">
            <h2
              class="font-editorial text-ink font-medium text-display-sm tracking-editorial"
            >
              {{ chapter.title }}
            </h2>
          </header>

          <ul>
            <li
              v-for="lesson in lessonsFor(chapter)"
              :key="lesson.slug"
              class="border-b border-dashed border-rule-soft py-5 last:border-b-0"
              :class="lesson.locked ? 'bg-locked-bg' : ''"
            >
              <NuxtLink
                :to="`/books/${book.slug}/pages/${lesson.slug}`"
                class="block px-2 no-underline"
              >
                <div class="flex items-baseline gap-6">
                  <span class="w-10 shrink-0 font-mono text-sm tabular-nums text-numeral">
                    {{ numberOf(lesson) }}
                  </span>
                  <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-3">
                      <h3
                        class="font-editorial text-xl text-ink sm:text-[1.375rem] font-semibold tracking-editorial"
                      >
                        {{ lesson.title }}
                      </h3>
                      <span
                        v-if="lesson.locked"
                        class="inline-flex items-center gap-1 rounded-full border border-lock-line bg-lock-bg px-2.5 py-0.5 font-mono text-xs text-lock"
                      >
                        voyage
                      </span>
                      <span
                        v-else
                        class="inline-flex items-center rounded-full border border-free-line bg-free-bg px-2.5 py-0.5 font-mono text-xs text-free"
                      >
                        free
                      </span>
                    </div>
                    <p
                      v-if="lesson.description"
                      class="mt-2 max-w-2xl text-sm leading-relaxed text-quiet"
                    >
                      {{ lesson.description }}
                    </p>
                  </div>
                </div>
              </NuxtLink>
            </li>
          </ul>
        </div>
      </div>

      <aside class="hidden lg:block">
        <div class="sticky top-24 rounded-lg bg-note p-7">
          <div class="font-mono text-xs tracking-wider uppercase text-rose">
            what you'll walk away with
          </div>
          <ul class="mt-5 space-y-4">
            <li
              v-for="chapter in chapters.slice(0, 6)"
              :key="chapter.id"
              class="flex gap-3 text-sm leading-relaxed text-ink"
            >
              <span class="mt-2 size-1.5 shrink-0 rounded-full bg-rose" />
              <span>{{ chapter.title }}</span>
            </li>
          </ul>
          <div
            v-if="chapters.length > 6"
            class="mt-6 border-t border-dashed border-rule-dashed pt-5 text-sm leading-relaxed italic text-quiet"
          >
            and {{ chapters.length - 6 }} more chapters.
          </div>
        </div>
      </aside>
    </section>
  </div>
</template>
