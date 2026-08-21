<script setup lang="ts">
import type { Project } from '@/types/Content'

const props = defineProps<{ challenges: Project[] }>()

const selected = ref<number>(0)
const active = computed<Project | undefined>(() => props.challenges[selected.value])
</script>

<template>
  <div class="border-pencil-light grid gap-0 rounded-md bg-panel lg:grid-cols-[minmax(0,20rem)_1fr]">
    <!-- master -->
    <ul class="max-h-[28rem] overflow-y-auto border-b border-rule lg:border-r lg:border-b-0">
      <li v-for="(challenge, i) in challenges" :key="challenge.slug">
        <button
          type="button"
          class="w-full cursor-pointer px-6 py-4 text-left transition-colors"
          :class="i === selected ? 'bg-paper' : 'hover:bg-paper'"
          @click="selected = i"
        >
          <div class="font-serif text-base text-ink">{{ challenge.name }}</div>
          <div class="mt-1 font-mono text-xs text-faint">{{ challenge.tasksCount }} tasks</div>
        </button>
      </li>
    </ul>

    <!-- detail -->
    <div v-if="active" class="flex flex-col p-8">
      <div class="mb-2 font-mono text-xs text-teal">/ {{ active.slug }}</div>
      <h4 class="font-serif text-2xl text-ink">{{ active.name }}</h4>
      <p class="mt-4 font-serif text-base leading-relaxed text-ink/85">
        {{ active.shortDescription }}
      </p>

      <div class="mt-8 overflow-hidden rounded-md bg-term-bg p-5">
        <pre class="overflow-x-auto font-mono text-xs leading-relaxed text-term-text"><span class="text-term-dim">$</span> luxctl challenges start {{ active.slug }}
<span class="text-term-dim">→ scenario ready in ./{{ active.slug }}</span>

<span class="text-term-dim">$</span> luxctl tasks submit
<span class="text-term-green">✓ {{ active.tasksCount }}/{{ active.tasksCount }} checks passed</span></pre>
      </div>

      <div class="mt-6 flex justify-end">
        <NuxtLink
          :to="`/projects/${active.slug}`"
          class="font-mono text-sm text-ink transition hover:text-teal"
        >
          take on the challenge <span class="ml-1 text-faint">———→</span>
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
