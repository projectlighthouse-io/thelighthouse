<script setup lang="ts">
import { faqs } from '@/data/Faqs'
import { manuscripts } from '@/data/Desk'
import { testimonials } from '@/data/Testimonials'
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

const projects = computed<Project[]>(() => allProjects.value ?? [])

// Counted from the same listings the bands render, never typed in.
const stats = computed(() => [
  { value: books.value.length, label: 'books' },
  { value: books.value.reduce((n, book) => n + book.pages, 0), label: 'pages' },
  { value: projects.value.filter(p => !p.isChallenge).length, label: 'projects' },
])
</script>

<template>
  <div class="home">
    <MarketingHero :stats="stats" />

    <section id="shelf" class="lh-text lh-gap">
      <div class="lh-head">
        <h2 class="lh-h2">Books</h2>
        <p class="lh-sub">Carefully crafted books to help you level up your skills</p>
      </div>
    </section>

    <div class="lh-shelf shelf">
      <BookGrid :books="books" fade />
    </div>

    <MarketingBuildAndDrill :projects="projects" fade />

    <MarketingPricingFrame />

    <MarketingOnTheDesk :manuscripts="manuscripts" />

    <MarketingMailbox :letters="testimonials" />

    <MarketingFaqSection />

    <MarketingAuthorLetter />
  </div>
</template>

<style scoped>
.shelf { margin-top: var(--space-12); }
</style>
