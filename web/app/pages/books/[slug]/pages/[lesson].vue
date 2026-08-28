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
