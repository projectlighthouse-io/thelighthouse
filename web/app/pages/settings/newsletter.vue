<script setup lang="ts">
definePageMeta({ layout: 'default', middleware: 'auth' })

useSeo({
  title: 'Newsletter preferences — projectlighthouse',
  description: 'Choose which emails you receive.',
  noindex: true,
})
</script>

<template>
  <SettingsShell>
    <h2 class="font-serif text-2xl text-ink">Newsletter preferences</h2>
    <p class="text-mono-body mt-2">One email when something worth reading ships. No drip campaigns.</p>

    <form class="mt-8 space-y-5" @submit.prevent>
      <label class="flex items-start gap-3">
        <input type="checkbox" class="mt-1 size-4 accent-ink">
        <span>
          <span class="block text-ink">New books and lessons</span>
          <span class="block text-sm text-quiet">when a chapter or book goes live</span>
        </span>
      </label>
      <label class="flex items-start gap-3">
        <input type="checkbox" class="mt-1 size-4 accent-ink">
        <span>
          <span class="block text-ink">Platform changes</span>
          <span class="block text-sm text-quiet">occasional notes on what changed</span>
        </span>
      </label>
      <button
        type="submit"
        class="rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
      >
        Save preferences
      </button>
    </form>
  </SettingsShell>
</template>
