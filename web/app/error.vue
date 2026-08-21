<script setup lang="ts">
import type { NuxtError } from '#app'

const props = defineProps<{ error: NuxtError }>()

const isMissing = computed<boolean>(() => props.error.statusCode === 404)
</script>

<template>
  <NuxtLayout>
    <div class="mx-auto flex max-w-2xl flex-col items-center px-4 py-32 text-center sm:px-6">
      <p class="font-mono text-xs tracking-[0.2em] uppercase text-teal">
        {{ error.statusCode }}
      </p>

      <h1 class="font-fredericka mt-4 text-5xl text-ink sm:text-6xl">
        {{ isMissing ? 'nothing here' : 'something broke' }}
      </h1>

      <p class="mt-6 font-serif text-lg leading-relaxed text-quiet">
        {{ isMissing
          ? 'That page moved, or never existed. The shelf is still where you left it.'
          : 'That is on us, not you. Try again in a moment.' }}
      </p>

      <div class="mt-10 flex flex-col gap-3 sm:flex-row">
        <NuxtLink
          to="/books"
          class="rounded-md bg-ink px-5 py-3 text-base font-medium text-on-ink transition hover:bg-ink-hover"
        >
          Browse the shelf
        </NuxtLink>
        <NuxtLink
          to="/"
          class="rounded-md border border-stroke bg-panel px-5 py-3 text-base font-medium text-ink transition hover:bg-paper-warm"
        >
          Go home
        </NuxtLink>
      </div>
    </div>
  </NuxtLayout>
</template>
