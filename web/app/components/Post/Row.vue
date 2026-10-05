<script setup lang="ts">
/**
 * One entry in a list of writing — a post, a change, a reference page: a mono
 * eyebrow (usually the date), an h3 title, a caption. A link when it goes
 * somewhere, a plain row when it does not.
 */
defineProps<{
  eyebrow: string
  title: string
  caption?: string
  to?: string
  datetime?: string
}>()

// Resolved here, not named as a string in `:is`: Nuxt registers NuxtLink by
// import, so the string renders an inert <nuxtlink> element.
const NuxtLink = resolveComponent('NuxtLink')
</script>

<template>
  <component :is="to ? NuxtLink : 'div'" :to="to" class="row" :class="{ 'is-link': to }">
    <span class="lh-eyebrow">
      <time v-if="datetime" :datetime="datetime">{{ eyebrow }}</time>
      <template v-else>{{ eyebrow }}</template>
    </span>
    <span class="lh-h3 title">{{ title }}</span>
    <span v-if="caption" class="lh-caption">{{ caption }}</span>
  </component>
</template>

<style scoped>
.row {
  display: grid;
  gap: 6px;
  padding: var(--space-4);
  border-radius: var(--radius-md);
  color: var(--ink);
  text-decoration: none;
  transition: background-color var(--duration) var(--ease-out);
}

.row.is-link:hover { background: var(--surface-sunken); color: var(--ink); }

.title { text-wrap: balance; }
</style>
