<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import { heroStats, horizonBooks } from '@/data/Home'
import type { Book, Project } from '@/types/Content'

useSeo({
  title: 'projectlighthouse — Software Engineering Fundamentals',
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

const projects = computed<Project[]>(() =>
  (allProjects.value ?? []).filter(p => !p.isChallenge))
const challenges = computed<Project[]>(() =>
  (allProjects.value ?? []).filter(p => p.isChallenge))
</script>

<template>
  <div>
    <MarketingHeroSection :stats="heroStats" />

    <!-- Full width, unlike every other band on this page: the shelf is a grid
         of ~290px cards, and inside the 7xl column it fits four and then stops
         — the rest of a wide monitor stays empty while the band runs long. -->
    <section class="py-12">
      <div class="px-2 sm:px-6 lg:px-8">
        <div class="mx-auto mb-12 max-w-3xl text-center">
          <h2 class="mb-3 font-serif text-4xl text-ink sm:text-5xl">Books</h2>
          <p class="text-mono-body">Carefully crafted books to help you level up your skills</p>
        </div>

        <!-- `HomeShelf`, not the `Shelf` `/books` renders: the same books and
             the same tabs, laid out two cards wide instead of eleven
             full-width rows. Two columns halve the band's height, which is
             what the scroll box was there to do — so the shelf sits on the
             page in full and the page scrolls, rather than a panel scrolling
             inside it.

             `bg-panel` for the same reason `/books` carries it: the cards tint
             on hover, and without a ground of their own they do it over the
             body's dotted paper. -->
        <div class="rounded-md bg-panel p-6 max-[820px]:p-4">
          <BookHomeShelf :books="books" />
        </div>

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
      <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
        <MarketingFounderEditionCta />
      </div>
    </section>

    <section id="projects-challenges" class="py-12">
      <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
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

        <!-- projects band -->
        <div class="mt-20">
          <div class="mb-2 font-mono text-xs text-teal">/ 01 · projects</div>
          <div
            class="mb-10 flex flex-col items-start justify-between gap-4 md:flex-row md:items-end"
          >
            <h3 class="font-serif text-3xl text-ink sm:text-4xl">
              build a <span class="italic text-teal">system</span> from scratch.
            </h3>
            <p class="max-w-sm font-serif text-base text-quiet md:text-right">
              multi-day builds. each ends with a working thing you can run.
              <NuxtLink to="/projects" class="ml-1 italic text-teal">— ship one →</NuxtLink>
            </p>
          </div>

          <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            <ProjectCard v-for="project in projects" :key="project.slug" :project="project" />
          </div>
        </div>

        <!-- challenges band -->
        <div class="mt-24">
          <div class="mb-2 font-mono text-xs text-teal">/ 02 · challenges</div>
          <div
            class="mb-10 flex flex-col items-start justify-between gap-4 md:flex-row md:items-end"
          >
            <h3 class="font-serif text-3xl text-ink sm:text-4xl">
              drill a <span class="italic text-teal">tool</span> until it's reflex.
            </h3>
            <p class="max-w-sm font-serif text-base text-quiet md:text-right">
              short scenarios. each is about a single unix tool every senior engineer reaches for.
              <NuxtLink to="/projects" class="ml-1 italic text-teal">— pick one →</NuxtLink>
            </p>
          </div>

          <ChallengeList :challenges="challenges" />
        </div>

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

    <section class="pt-16 pb-12 sm:pt-24">
      <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
        <MarketingOnTheHorizon :books="horizonBooks" />
      </div>
    </section>

    <MarketingTestimonialsSection />

    <MarketingFaqSection />

    <MarketingAuthorLetter />
  </div>
</template>
