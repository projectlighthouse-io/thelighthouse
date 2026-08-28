<script setup lang="ts">
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
 * Which contents entry is highlighted.
 *
 * The heading nearest the top of the viewport without having passed it — the
 * section you are reading, not the one nearest the middle of the screen.
 *
 * Read off `getBoundingClientRect` rather than an `IntersectionObserver`. An
 * observer answers "is this heading in view", and the section being *read* is
 * usually the one whose heading has already scrolled out of it: a long section
 * leaves nothing intersecting at all, and the highlight sticks on whichever
 * heading happened to fire last. Comparing positions answers the question
 * being asked instead.
 */
// Seeded with the first entry so the server renders the same thing the browser
// will a frame later. Left empty it would paint a contents list with nothing
// highlighted until hydration, which is a flicker on every lesson open.
const activeId = ref<string>(data.value?.toc?.[0]?.id ?? '')

/** How far below the top of the viewport a heading counts as passed. */
const ACTIVATION_LINE = 96

onMounted(() => {
  let queued = false

  const update = (): void => {
    queued = false

    const ids = (data.value?.toc ?? []).map(item => item.id)
    const headings = ids
      .map(id => document.getElementById(id))
      .filter((el): el is HTMLElement => el !== null)

    if (headings.length === 0) {
      return
    }

    // The last heading that has crossed the line. Before the first one has,
    // the first entry stands — the reader is in the lesson's opening, which
    // belongs to it.
    let current = headings[0]!.id
    for (const heading of headings) {
      if (heading.getBoundingClientRect().top <= ACTIVATION_LINE) {
        current = heading.id
      }
    }

    // At the bottom of the page the final section can never cross the line —
    // there is no scroll left to bring it up. Without this the last entry is
    // unreachable however far you scroll.
    const atBottom
      = window.innerHeight + window.scrollY >= document.body.scrollHeight - 2
    if (atBottom) {
      current = headings[headings.length - 1]!.id
    }

    activeId.value = current
  }

  // Coalesced to one read per frame: scroll fires far more often than the
  // screen is painted, and every call here measures layout.
  const onScroll = (): void => {
    if (queued) {
      return
    }
    queued = true
    requestAnimationFrame(update)
  }

  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('resize', onScroll, { passive: true })

  // Re-measure when the lesson changes: same component, different headings.
  const stop = watch(() => data.value?.toc, () => nextTick(update), {
    immediate: true,
  })

  onBeforeUnmount(() => {
    stop()
    window.removeEventListener('scroll', onScroll)
    window.removeEventListener('resize', onScroll)
  })
})

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
  </div>
</template>
