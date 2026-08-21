<script setup lang="ts">
definePageMeta({ layout: 'settings', middleware: 'auth' })

useSeo({
  title: 'Profile — projectlighthouse',
  description: 'Your account details.',
  noindex: true,
})
</script>

<template>
  <div>
    <h2 class="font-serif text-2xl text-ink">Profile</h2>
    <p class="text-mono-body mt-2">Your name and email come from the provider you signed in with.</p>

    <dl class="mt-8 space-y-5">
      <div>
        <dt class="font-mono text-xs tracking-wider uppercase text-faint">name</dt>
        <dd class="mt-1 text-ink">—</dd>
      </div>
      <div>
        <dt class="font-mono text-xs tracking-wider uppercase text-faint">email</dt>
        <dd class="mt-1 text-ink">—</dd>
      </div>
      <div>
        <dt class="font-mono text-xs tracking-wider uppercase text-faint">signed in with</dt>
        <dd class="mt-1 text-ink">—</dd>
      </div>
    </dl>

    <p class="mt-10 text-sm text-quiet">
      Values arrive from <code class="font-mono">GET /api/me</code> once auth lands — see
      docs/rebuild.md phase 3.
    </p>
  </div>
</template>
