<script setup lang="ts">
interface SettingsLink {
  to: string
  label: string
}

// Nuxt layouts replace each other rather than nesting, so the template wraps
// the default layout explicitly. Without it, settings pages lose the site
// header and footer entirely.
const links: SettingsLink[] = [
  { to: '/settings/profile', label: 'profile' },
  { to: '/settings/public-profile', label: 'public profile' },
  { to: '/settings/tokens', label: 'api tokens' },
  { to: '/settings/two-factor', label: 'two-factor' },
  { to: '/settings/newsletter', label: 'newsletter' },
]
</script>

<template>
  <NuxtLayout name="default">
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
  </NuxtLayout>
</template>
