<script setup lang="ts">
/**
 * The settings chrome: the heading, the section nav, and whatever a page puts
 * inside it.
 *
 * A component rather than a layout, deliberately. Nuxt layouts replace each
 * other instead of nesting, so a settings *layout* had to render
 * `<NuxtLayout name="default">` inside itself to keep the site header — and a
 * nested `NuxtLayout` wraps an async-loaded layout in a second layout
 * transition. On the first render that async component is still a comment
 * placeholder, which a `<Transition>` cannot animate, and vue warns about it on
 * every settings page.
 *
 * Composing at the component level has none of that: the page uses the default
 * layout like every other page, and this is just markup inside it.
 */

interface SettingsLink {
  to: string
  label: string
}

const links: SettingsLink[] = [
  { to: '/settings/profile', label: 'profile' },
  { to: '/settings/public-profile', label: 'public profile' },
  { to: '/settings/tokens', label: 'api tokens' },
  { to: '/settings/billing', label: 'billing' },
]
</script>

<template>
  <div class="border-pencil-light mx-auto my-16 max-w-3xl rounded-md bg-panel px-4 py-10 sm:px-6 lg:px-8">
    <h1 class="mb-2 font-serif text-3xl tracking-tight text-ink sm:text-4xl">Settings</h1>
    <p class="mb-10 text-sm leading-relaxed text-quiet">manage your account and preferences.</p>

    <div class="grid gap-8 lg:grid-cols-[180px_1fr] lg:items-start lg:gap-10">
      <nav class="flex flex-wrap gap-2 lg:sticky lg:top-24 lg:flex-col">
        <NuxtLink
          v-for="link in links"
          :key="link.to"
          :to="link.to"
          class="rounded-md px-3 py-2 text-sm text-quiet transition hover:bg-paper-warm hover:text-ink"
          active-class="bg-paper-warm font-medium text-ink"
        >
          {{ link.label }}
        </NuxtLink>
      </nav>

      <div class="min-w-0 lg:border-l lg:border-rule lg:pl-10">
        <slot />
      </div>
    </div>
  </div>
</template>
