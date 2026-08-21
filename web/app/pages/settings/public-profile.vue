<script setup lang="ts">
definePageMeta({ layout: 'settings', middleware: 'auth' })

useSeo({
  title: 'Public profile — projectlighthouse',
  description: 'What other learners see on your profile.',
  noindex: true,
})
</script>

<template>
  <div>
    <h2 class="font-serif text-2xl text-ink">Public profile</h2>
    <p class="text-mono-body mt-2">What other learners see. Off by default.</p>

    <form class="mt-8 space-y-6" @submit.prevent>
      <label class="block">
        <span class="font-mono text-xs tracking-wider uppercase text-faint">username</span>
        <input
          type="text"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-ink focus:border-stroke focus:outline-none"
          placeholder="your-handle"
        >
      </label>
      <label class="block">
        <span class="font-mono text-xs tracking-wider uppercase text-faint">bio</span>
        <textarea
          rows="3"
          class="mt-2 w-full rounded-md border border-rule bg-panel px-4 py-2.5 text-ink focus:border-stroke focus:outline-none"
        />
      </label>
      <button
        type="submit"
        class="rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
      >
        Save
      </button>
    </form>
  </div>
</template>
