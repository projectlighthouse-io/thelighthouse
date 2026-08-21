<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

useSeo({
  title: 'Profile — projectlighthouse',
  description: 'Your projectlighthouse profile and reading stats.',
  noindex: true,
})

const stats = [
  { label: 'lessons read', value: '—' },
  { label: 'projects shipped', value: '—' },
  { label: 'notes taken', value: '—' },
  { label: 'day streak', value: '—' },
]
</script>

<template>
  <div class="mx-auto max-w-3xl px-4 py-16 sm:px-6 lg:px-8">
    <div class="flex items-center gap-5">
      <div class="size-16 shrink-0 rounded-full bg-paper-edge" />
      <div>
        <h1 class="font-serif text-3xl tracking-tight text-ink">Your profile</h1>
        <p class="text-mono-body mt-1">signed in with google or github.</p>
      </div>
    </div>

    <dl class="mt-12 grid grid-cols-2 gap-px overflow-hidden rounded-md bg-rule sm:grid-cols-4">
      <div v-for="stat in stats" :key="stat.label" class="bg-panel p-5">
        <dt class="font-mono text-xs text-faint">{{ stat.label }}</dt>
        <dd class="mt-1 font-serif text-2xl text-ink">{{ stat.value }}</dd>
      </div>
    </dl>

    <div class="mt-10">
      <NuxtLink
        to="/settings/profile"
        class="btn-chalk text-sm font-medium text-ink"
      >
        edit settings <span class="ml-1">———→</span>
      </NuxtLink>
    </div>
  </div>
</template>
