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
  { to: '/settings/two-factor', label: 'two-factor' },
  { to: '/settings/newsletter', label: 'newsletter' },
  { to: '/settings/billing', label: 'billing' },
]
</script>

<template>
  <div class="mx-auto max-w-5xl px-4 py-16 sm:px-6 lg:px-8">
    <h1 class="mb-2 font-serif text-3xl tracking-tight text-ink sm:text-4xl">Settings</h1>
    <p class="text-mono-body mb-10">manage your account and preferences.</p>

    <div class="grid gap-10 lg:grid-cols-[200px_1fr] lg:items-start">
      <nav class="flex flex-wrap gap-2 lg:sticky lg:top-24 lg:flex-col">
        <NuxtLink
          v-for="link in links"
          :key="link.to"
          :to="link.to"
          class="rounded-md px-3 py-2 font-mono text-sm text-quiet transition hover:text-ink"
          active-class="border-pencil-solid-black text-ink"
        >
          {{ link.label }}
        </NuxtLink>
      </nav>

      <div class="border-pencil-light min-w-0 rounded-md bg-panel p-8">
        <slot />
      </div>
    </div>
  </div>
</template>
