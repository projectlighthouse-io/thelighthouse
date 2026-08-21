<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

const search = ref<string>('')

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

    <div class="border-pencil-light mt-10 rounded-md bg-panel py-16 text-center">
      <p class="text-sm text-quiet">No notes yet.</p>
      <p class="mx-auto mt-3 max-w-sm text-sm text-quiet">
        Select any passage while reading to save it here. The notes island is the one real piece of
        client state in the rebuild — see docs/rebuild.md phase 6.
      </p>
    </div>
  </div>
</template>
