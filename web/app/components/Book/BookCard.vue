<script setup lang="ts">
import type { Book } from '@/types/Content'

defineProps<{ book: Book }>()
</script>

<template>
  <article class="group border-pencil overflow-hidden rounded-md bg-panel">
    <div class="aspect-video overflow-hidden bg-paper-edge">
      <img
        :src="book.thumbnailUrl"
        :alt="book.title"
        width="480"
        height="270"
        loading="lazy"
        class="size-full object-cover dark:brightness-90"
      >
    </div>

    <div class="p-6">
      <div class="mb-3 flex items-center justify-between">
        <span
          v-if="book.inProgress"
          class="font-mono text-xs font-bold text-wip"
        >
          <span
            class="mr-1 inline-block size-1.5 rounded-full bg-wip align-middle"
          />in progress
        </span>
        <span
          v-else-if="book.price"
          class="rounded-full bg-ink px-3 py-1 text-xs font-semibold text-on-ink"
        >
          {{ book.price }}
        </span>
        <span v-else />
        <span class="text-sm text-quiet">{{ book.pages }} pages</span>
      </div>

      <h3 class="mb-2 text-xl font-semibold text-ink">{{ book.title }}</h3>

      <p class="mb-4 line-clamp-2 font-serif text-base leading-relaxed text-ink/90">
        {{ book.description }}
      </p>

      <div class="flex justify-end">
        <NuxtLink :to="`/books/${book.slug}`" class="btn-chalk text-sm font-medium text-ink">
          read book
          <svg class="ml-1 size-4 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </NuxtLink>
      </div>
    </div>
  </article>
</template>
