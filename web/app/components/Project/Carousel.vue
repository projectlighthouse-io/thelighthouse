<script setup lang="ts">
import type { Project } from '@/types/Content'

/**
 * The projects on the home page, as a row that slides itself.
 *
 * **Against the books, not with them.** `reverse`, so the two bands on this
 * page travel opposite ways. Two rows moving in step read as one long thing
 * scrolling behind a page; moving against each other they read as two shelves,
 * which is what they are — and it gives the eye somewhere to catch.
 *
 * The card is `ProjectCard`, the same one `/projects` and the grid here used
 * before. Nothing about it changed for this.
 */
defineProps<{ projects: Project[] }>()
</script>

<template>
  <MarqueeRow :items="projects" reverse width="380px" :seconds-per-item="8">
    <template #default="{ item }">
      <ProjectCard :project="item" class="h-full" />
    </template>
  </MarqueeRow>
</template>
