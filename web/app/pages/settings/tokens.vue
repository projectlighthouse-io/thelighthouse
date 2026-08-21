<script setup lang="ts">
definePageMeta({ layout: 'settings', middleware: 'auth' })

useSeo({
  title: 'API tokens — projectlighthouse',
  description: 'API tokens for the luxctl CLI.',
  noindex: true,
})
</script>

<template>
  <div>
    <h2 class="font-serif text-2xl text-ink">API tokens</h2>
    <p class="text-mono-body mt-2">
      luxctl authenticates with a bearer token. Create one, copy it once, keep it somewhere safe.
    </p>

    <div class="border-pencil-light mt-8 rounded-md p-6 text-center">
      <p class="text-sm text-quiet">No tokens yet.</p>
      <button
        type="button"
        class="mt-4 rounded-md bg-ink px-5 py-2.5 text-sm font-medium text-on-ink transition hover:bg-ink-hover"
      >
        Create token
      </button>
    </div>

    <p class="mt-6 text-sm text-quiet">
      A token is shown once at creation and never again. If you lose it, revoke it and make another.
    </p>
  </div>
</template>
