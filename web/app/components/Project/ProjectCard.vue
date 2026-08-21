<script setup lang="ts">
import type { Project } from '@/types/Content'

const props = defineProps<{ project: Project }>()

const taskLabel = computed<string>(() =>
  props.project.tasksCount === 1 ? '1 task' : `${props.project.tasksCount} tasks`,
)
</script>

<template>
  <article
    class="border-pencil-light group flex flex-col overflow-hidden rounded-md bg-panel p-7 transition-shadow hover:shadow-md"
  >
    <div class="mb-5 flex items-center justify-end font-mono text-xs">
      <span class="text-faint">{{ taskLabel }}</span>
    </div>

    <h3 class="mb-3 font-serif text-2xl leading-tight text-ink">{{ project.name }}</h3>

    <p class="mb-5 line-clamp-4 font-serif text-base leading-relaxed text-ink/85">
      {{ project.shortDescription }}
    </p>

    <div class="my-5 min-h-[120px] flex-1">
      <div class="flex h-full items-center font-mono text-xs text-faint">
        <span class="opacity-70">{{ project.slug }}</span>
      </div>
    </div>

    <div class="flex justify-end">
      <NuxtLink
        :to="`/projects/${project.slug}`"
        class="font-mono text-sm text-ink transition group-hover:text-teal"
      >
        view project <span class="ml-1 text-faint group-hover:text-teal">———→</span>
      </NuxtLink>
    </div>
  </article>
</template>
