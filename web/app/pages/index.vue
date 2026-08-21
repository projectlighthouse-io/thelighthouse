<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import { books } from '@/data/Books'
import { heroStats, horizonBooks } from '@/data/Home'
import { challenges, projects } from '@/data/Projects'

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

useJsonLd('shelf', {
  '@type': 'ItemList',
  'name': 'Books',
  'itemListElement': books.map((b, i) => ({
    '@type': 'ListItem',
    'position': i + 1,
    'url': `${SITE.url}/books/${b.slug}`,
    'name': b.title,
  })),
})

</script>

<template>
  <div>
    <MarketingHeroSection :stats="heroStats" />

    <section class="py-12">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="mx-auto mb-12 max-w-3xl text-center">
          <h2 class="mb-3 font-serif text-4xl text-ink sm:text-5xl">Books</h2>
          <p class="text-mono-body">Carefully crafted books to help you level up your skills</p>
        </div>

        <div class="lg:solid-gray-bg rounded-md p-0 lg:p-6">
          <div class="grid w-full gap-8 md:grid-cols-2 lg:grid-cols-3">
            <BookCard v-for="book in books" :key="book.slug" :book="book" />
          </div>
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
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <MarketingOnTheHorizon :books="horizonBooks" />
      </div>
    </section>

    <MarketingTestimonialsSection />

    <MarketingFaqSection />

    <MarketingAuthorLetter />
  </div>
</template>
