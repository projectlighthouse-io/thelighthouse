<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import { heroStats, horizonBooks } from '@/data/Home'
import type { Book, Project } from '@/types/Content'

useSeo({
  title: 'Software Engineering Fundamentals — projectlighthouse',
  description: 'Build your own Docker, HTTP server, DNS resolver and more from scratch. Interactive courses on Go, Rust, DSA, networking, and OS fundamentals. Hands-on projects validated on your own machine by an open-source CLI.',
})

useJsonLd('faq', {
  '@type': 'FAQPage',
  'mainEntity': faqs.map(f => ({
    '@type': 'Question',
    'name': f.question,
    'acceptedAnswer': { '@type': 'Answer', 'text': f.answer },
  })),
})


// The same listing the books and projects pages render, so the home bands
// cannot drift from them — which they did, silently, for as long as each read
// its own copy. The books band read a hand-kept array and was two books and
// every track behind before this.
const { data: allBooks } = await useAsyncData('home-books', () =>
  $fetch<Book[]>('/_api/books'))

const books = computed<Book[]>(() => allBooks.value ?? [])

// After `books`, and that is load bearing. `useJsonLd` registers a
// `watchEffect` that runs the moment it is called, so a getter reading a
// `const` declared further down throws on that first run — the head entry is
// then never created, and unhead disposes an undefined entry when the page
// unmounts. The visible symptoms were an unmount error on every navigation
// away from the home page, and no shelf markup in the html at all.
useJsonLd('shelf', () => ({
  '@type': 'ItemList',
  'name': 'Books',
  'itemListElement': books.value.map((b, i) => ({
    '@type': 'ListItem',
    'position': i + 1,
    'url': `${SITE.url}/books/${b.slug}`,
    'name': b.title,
  })),
}))

const { data: allProjects } = await useAsyncData('home-projects', () =>
  $fetch<Project[]>('/_api/projects'))

/**
 * Projects and challenges together, in one row.
 *
 * They were two bands with a heading each, and the split asked the reader to
 * care about a distinction that only matters once you are inside one — a
 * challenge is a short project. `/projects` still separates them.
 */
const projects = computed<Project[]>(() => allProjects.value ?? [])
</script>

<template>
  <div>
    <MarketingHeroSection :stats="heroStats" />

    <!--
      The heading and the link keep the page's column; the rail does not.

      A marquee that stops at the column edge reads as a box with things moving
      inside it. Running it to both edges of the window is what makes it read as
      a shelf passing by, and it is the only band on this page that wants that.
    -->
    <section class="py-12">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mx-auto mb-12 max-w-3xl text-center">
          <h2 class="mb-3 font-serif text-4xl text-ink sm:text-5xl">Books</h2>
          <p class="text-mono-body">Carefully crafted books to help you level up your skills</p>
        </div>
      </div>

      <!-- A rail, not the full shelf: browsing moved to the nav's dropdown,
           which has every book grouped by track and is one click from any
           page. What is left for this band is to show that there is a shelf
           and what it looks like. `/books` still lists them all. -->
      <BookCarousel :books="books" />

      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mt-10 text-center">
          <NuxtLink
            to="/books"
            class="inline-flex items-center font-mono text-sm text-ink transition hover:text-teal"
          >
            view all books <span class="ml-2">———→</span>
          </NuxtLink>
        </div>
      </div>
    </section>

    <section class="py-12">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <MarketingFounderEditionCta />
      </div>
    </section>

    <section id="projects-challenges" class="py-12">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mb-6 text-center font-mono text-xs tracking-[0.2em] text-teal">
          / PROJECTS &amp; CHALLENGES
        </div>

        <h2
          class="mx-auto max-w-4xl text-center font-serif text-5xl leading-[1.05] text-ink sm:text-6xl"
        >
          build real <span class="italic text-teal">systems</span>,<br>
          sharpen real <span class="italic text-teal">tools</span>.
        </h2>

        <p
          class="mx-auto mt-8 max-w-2xl text-center font-serif text-lg leading-relaxed italic text-ink"
        >
          two ways to put what you've learned into your hands. pick a system to build from scratch,
          or run a short drill on the unix tools that show up in every on-call.
        </p>

        <div class="mt-20">
          <div class="mb-2 font-mono text-xs text-teal">/ projects &amp; challenges</div>
          <div
            class="mb-10 flex flex-col items-start justify-between gap-4 md:flex-row md:items-end"
          >
            <h3 class="font-serif text-3xl text-ink sm:text-4xl">
              build a <span class="italic text-teal">system</span> from scratch.
            </h3>
            <p class="max-w-sm font-serif text-base text-quiet md:text-right">
              multi-day builds and short drills. each ends with a working thing
              you can run.
              <NuxtLink to="/projects" class="ml-1 italic text-teal">— ship one →</NuxtLink>
            </p>
          </div>
        </div>
      </div>

      <!-- Full bleed, like the books row: a band that stops at the column edge
           reads as a box with things moving inside it. Sliding the other way to
           the books, so the two read as two shelves rather than one long thing
           scrolling past. -->
      <ProjectCarousel :projects="projects" />

      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mt-16 text-center">
          <NuxtLink
            to="/projects"
            class="inline-flex items-center font-mono text-sm text-ink transition hover:text-teal"
          >
            view all projects &amp; challenges <span class="ml-2">———→</span>
          </NuxtLink>
        </div>
      </div>
    </section>

    <MarketingTestimonialsSection />

    <section class="pt-16 pb-12 sm:pt-24">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <MarketingOnTheHorizon :books="horizonBooks" />
      </div>
    </section>

    <MarketingFaqSection />

    <MarketingAuthorLetter />

    <!-- Last thing on the page, under the letter — the footer's copy is right
         there too, and one of the two being scrolled past is fine. -->
    <!-- Wider than the reading column above it: the envelope is a landscape
         thing — address on the left, stamp and button on the right — and it
         reads cramped in a column sized for prose. -->
    <!-- `pb-8`, not the `pb-20` the other bands carry: the panel already ends
         in 44px of its own padding, and stacking 80px more under it left a
         hole between the envelope and the wordmark. -->
    <section class="pb-8">
      <div class="mx-auto max-w-5xl px-4 sm:px-6 lg:px-8">
        <MarketingNewsletterForm />
      </div>
    </section>
  </div>
</template>
