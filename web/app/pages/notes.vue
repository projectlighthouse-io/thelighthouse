<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

const { notes, page, pages, total, loaded, pending, failed, search, load, goTo } = useNotes()

onMounted(load)

/** The day, in the reader's own locale. The time of day is not worth the row. */
function on(iso: string | null): string {
  if (!iso) return ''

  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

useSeo({
  title: 'My notes — projectlighthouse',
  description: 'Every passage you highlighted, with your notes beside it.',
  noindex: true,
})
</script>

<template>
  <div class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
    <h1 class="mb-2 font-serif text-3xl tracking-tight text-ink sm:text-4xl">My notes</h1>
    <p class="text-mono-body mb-8">
      every passage you highlighted, with whatever you wrote next to it.
    </p>

    <input
      v-model="search"
      type="search"
      placeholder="search your notes…"
      class="w-full rounded-md border border-rule bg-panel px-4 py-2.5 font-mono text-sm text-ink placeholder:text-faint focus:border-stroke focus:outline-none"
    >

    <!-- Nothing at all until the first answer. An empty state drawn while the
         request is still out is a lie shown to everybody who has notes. -->
    <div v-if="!loaded" class="mt-10 py-16 text-center">
      <p class="text-sm text-quiet">Loading…</p>
    </div>

    <div v-else-if="failed" class="border-pencil-light mt-10 rounded-md bg-panel py-16 text-center">
      <p class="text-sm text-quiet">Your notes could not be loaded.</p>
      <button
        type="button"
        class="mt-4 rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
        @click="load"
      >
        Try again
      </button>
    </div>

    <div
      v-else-if="notes.length === 0"
      class="border-pencil-light mt-10 rounded-md bg-panel py-16 text-center"
    >
      <p class="text-sm text-quiet">
        {{ search ? 'No notes match that.' : 'No notes yet.' }}
      </p>
      <p v-if="!search" class="mx-auto mt-3 max-w-sm text-sm text-quiet">
        Select any passage while reading to save it here.
      </p>
    </div>

    <div v-else :class="['mt-10 space-y-6', pending && 'opacity-60']">
      <article
        v-for="note in notes"
        :key="note.id"
        class="border-pencil-light rounded-md bg-panel px-5 py-4"
      >
        <NuxtLink
          :to="`/books/${note.bookSlug}/lessons/${note.lessonSlug}`"
          class="font-mono text-xs text-quiet transition hover:text-ink"
        >
          {{ note.bookTitle ?? note.bookSlug }} — {{ note.lessonTitle ?? note.lessonSlug }}
        </NuxtLink>

        <blockquote
          v-if="note.selectedText"
          class="mt-3 border-l-2 border-rule pl-4 font-serif text-ink"
        >
          {{ note.selectedText }}
        </blockquote>

        <p v-if="note.noteContent" class="text-mono-body mt-3">
          {{ note.noteContent }}
        </p>

        <p class="mt-3 font-mono text-xs text-faint">{{ on(note.createdAt) }}</p>
      </article>

      <div v-if="pages > 1" class="flex items-center justify-between pt-2">
        <button
          type="button"
          :disabled="page <= 1"
          class="rounded-md border border-rule px-4 py-2 font-mono text-sm text-ink transition hover:border-stroke disabled:opacity-40"
          @click="goTo(page - 1)"
        >
          Previous
        </button>

        <span class="font-mono text-xs text-quiet">
          page {{ page }} of {{ pages }} — {{ total }} notes
        </span>

        <button
          type="button"
          :disabled="page >= pages"
          class="rounded-md border border-rule px-4 py-2 font-mono text-sm text-ink transition hover:border-stroke disabled:opacity-40"
          @click="goTo(page + 1)"
        >
          Next
        </button>
      </div>
    </div>
  </div>
</template>
