<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

useSeo({
  title: 'Profile — projectlighthouse',
  description: 'Your projectlighthouse profile.',
  noindex: true,
})

// The route guard already resolved the session to let this page render, so
// this reads the state rather than asking again.
const { reader, initials } = useReader()
</script>

<template>
  <div class="mx-auto max-w-3xl px-2 py-16 sm:px-6 lg:px-8">
    <div class="flex items-center gap-5">
      <img
        v-if="reader?.avatar"
        :src="reader.avatar"
        alt=""
        class="size-16 shrink-0 rounded-full object-cover"
      >
      <div
        v-else
        class="flex size-16 shrink-0 items-center justify-center rounded-full bg-ink font-mono text-lg text-on-ink"
      >
        {{ initials }}
      </div>
      <div>
        <h1 class="font-serif text-3xl tracking-tight text-ink">
          {{ reader?.name ?? 'Your profile' }}
        </h1>
        <p class="text-mono-body mt-1">
          {{ reader ? `${reader.email} · signed in with ${reader.provider}` : 'loading…' }}
        </p>
      </div>
    </div>
  </div>
</template>
