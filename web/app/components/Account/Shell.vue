<script setup lang="ts">
/**
 * The frame every account page sits in: a heading, the account's own nav —
 * links in the header's style, the current one in ink — and the page.
 */
defineProps<{
  title: string
  sub?: string
}>()

interface AccountLink {
  to: string
  label: string
}

const links: AccountLink[] = [
  { to: '/dashboard', label: 'my shelf' },
  { to: '/profile', label: 'progress' },
  { to: '/notes', label: 'notes' },
  { to: '/settings/billing', label: 'billing' },
  { to: '/settings/profile', label: 'profile' },
  { to: '/settings/public-profile', label: 'public profile' },
  { to: '/settings/tokens', label: 'api tokens' },
]

const route = useRoute()
</script>

<template>
  <div class="account lh-figure lh-gap">
    <div class="lh-head">
      <p class="lh-eyebrow">account</p>
      <h1 class="lh-h1">{{ title }}</h1>
      <p v-if="sub" class="lh-sub">{{ sub }}</p>
    </div>

    <nav class="nav" aria-label="account">
      <NuxtLink
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        class="link"
        :class="{ 'is-active': route.path === link.to }"
        :aria-current="route.path === link.to ? 'page' : undefined"
      >
        {{ link.label }}
      </NuxtLink>
    </nav>

    <div class="body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.nav {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2) 28px;
  margin-top: var(--space-8);
  padding-bottom: var(--space-4);
  border-bottom: 1px dashed var(--border-dashed);
}

.link {
  font: var(--text-caption);
  color: var(--ink-secondary);
  text-decoration: none;
  transition: color var(--duration) var(--ease-out);
}

.link:hover,
.link.is-active { color: var(--ink); }

.body { margin-top: var(--space-12); }
</style>
