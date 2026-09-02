<script setup lang="ts">
import '@/assets/css/reader.css'

const description
  = 'How projectlighthouse is put together — the books, the projects, the CLI, and how they connect.'

useSeo({
  title: 'Connecting the dots — projectlighthouse',
  description,
  type: 'article',
})

useJsonLd('about', {
  '@type': 'AboutPage',
  'name': 'Connecting the dots',
  'url': `${SITE.url}/connecting-the-dots`,
})
</script>

<template>
  <div class="reader-shell">
    <div class="legal-layout">
      <article class="reader-article">
        <div class="reader-eyebrow">how it fits together</div>
        <h1>Connecting the dots</h1>
        <p class="reader-dek">
          Books teach the idea. Projects make you build it. luxctl checks your work. That is the
          whole loop.
        </p>

        <div class="reader-prose mt-11">
          <div class="lesson-content">
            <p>
              Every book here starts from something you can hold in your head — two computers and a
              wire, a process and its memory, a byte on a disk — and builds up from there. No
              framework first, no magic left standing.
            </p>

            <h2>Books</h2>
            <p>
              Each book is a sequence of lessons meant to be read in order. Early lessons in every
              book are free; the rest come with the track that book is on. Notes and highlights are
              yours and stay private.
            </p>

            <h2>Projects</h2>
            <p>
              A project is a real thing you build on your own machine — an HTTP server from raw
              sockets, a DNS resolver, a container runtime. Broken into tasks, each with its own
              validator.
            </p>

            <h2>Challenges</h2>
            <p>
              Shorter drills on a single unix tool: grep, sed, awk, find, jq, the shell. The kind of
              thing you reach for at 3am and want to already know.
            </p>

            <h2>luxctl</h2>
            <p>
              An open-source CLI that fetches your project, runs the validator locally, and reports
              back. Nothing you write leaves your machine — only the result does.
            </p>
          </div>
        </div>
      </article>
    </div>
  </div>
</template>

<style scoped>
.legal-layout {
  max-width: 760px;
  margin: 0 auto;
  padding: 72px 40px 96px;
}

@media (max-width: 880px) {
  .legal-layout {
    padding: 40px 20px 80px;
  }
}
</style>
