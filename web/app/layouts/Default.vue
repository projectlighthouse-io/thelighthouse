<script setup lang="ts">
interface NavLink {
  to: string
  label: string
  external?: boolean
}

const navLinks: NavLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/syntax', label: 'syntax' },
  { to: '/pricing', label: 'pricing' },
]

const footerLinks: NavLink[] = [
  { to: '/books', label: 'books' },
  { to: '/projects', label: 'projects' },
  { to: '/roadmap', label: 'roadmap' },
  { to: '/connecting-the-dots', label: 'connecting the dots' },
  { to: '/terms', label: 'terms' },
  { to: '/privacy', label: 'privacy' },
  { to: '/changelog', label: 'changelog' },
  { to: '/support', label: 'support' },
  { to: 'https://www.linkedin.com/company/projectlighthouse-io', label: 'linkedin', external: true },
  { to: 'https://projectlighthouse.substack.com/', label: 'substack', external: true },
  { to: '/blog', label: 'blog' },
]

const { user, isSignedIn } = usePreviewAuth()

// site-level identity, emitted once for every page that uses this layout
useJsonLd('site', {
  '@type': 'WebSite',
  'name': SITE.name,
  'url': SITE.url,
  'publisher': {
    '@type': 'Organization',
    'name': SITE.name,
    'url': SITE.url,
    'logo': `${SITE.url}/projectlighthouse.png`,
    'sameAs': [
      'https://www.linkedin.com/company/projectlighthouse-io',
      'https://projectlighthouse.substack.com/',
    ],
  },
})

// hardcoded until the rust api exposes it
const appVersion = { version: 'v0.9', build: 'rebuild' }
const year = new Date().getFullYear()
</script>

<template>
  <div class="flex min-h-screen flex-col">
    <header class="dotted-bg fixed top-0 right-0 left-0 z-50">
      <div class="mx-auto max-w-full px-4 sm:px-6 lg:px-8">
        <div class="relative flex h-16 items-center justify-between">
          <NuxtLink to="/" class="flex items-center gap-2">
            <img src="/lighthouse.svg" alt="projectlighthouse logo" class="h-8 w-8 object-contain">
            <span class="text-base tracking-tight text-ink">projectlighthouse</span>
          </NuxtLink>

          <nav class="absolute left-1/2 hidden -translate-x-1/2 items-center gap-2 sm:flex">
            <NuxtLink
              v-for="link in navLinks"
              :key="link.to"
              :to="link.to"
              class="px-4 py-1 font-sans text-xs text-ink transition hover:text-link-hover sm:text-sm"
            >
              {{ link.label }}
            </NuxtLink>
          </nav>

          <div class="flex items-center gap-1">
            <ChromeThemeToggle />

            <NuxtLink
              v-if="isSignedIn"
              to="/profile"
              class="flex items-center gap-2 rounded-md px-2 py-1.5 transition hover:bg-paper-warm"
              :title="user?.email"
            >
              <span
                class="flex size-7 items-center justify-center rounded-full bg-ink font-mono text-xs text-on-ink"
              >{{ user?.initials }}</span>
              <span class="hidden font-sans text-sm text-ink sm:inline">{{ user?.name }}</span>
            </NuxtLink>

            <NuxtLink
              v-else
              to="/login"
              class="rounded-lg border border-stroke bg-ink px-4 py-2 text-sm font-semibold text-on-ink transition hover:bg-ink-hover sm:px-6"
            >
              join
            </NuxtLink>
          </div>
        </div>
      </div>
    </header>

    <main class="mt-16 flex flex-1 flex-col">
      <slot />
    </main>

    <!-- ascii wordmark — hidden on mobile, too wide to be legible -->
    <div class="hidden w-full sm:block">
      <ChromeBinaryLogo />
    </div>

    <footer class="mt-auto px-4 py-8 sm:px-6 lg:px-8">
      <div class="mx-auto w-full max-w-7xl text-center">
        <nav
          class="mb-6 grid grid-cols-2 gap-x-4 gap-y-2 sm:flex sm:flex-wrap sm:items-center sm:justify-center sm:gap-x-6 sm:gap-y-2"
        >
          <template v-for="link in footerLinks" :key="link.to">
            <a
              v-if="link.external"
              :href="link.to"
              target="_blank"
              rel="noopener noreferrer"
              class="text-sm text-quiet transition hover:text-ink"
            >
              {{ link.label }}
            </a>
            <NuxtLink
              v-else
              :to="link.to"
              class="text-sm text-quiet transition hover:text-ink"
            >
              {{ link.label }}
            </NuxtLink>
          </template>
        </nav>

        <div class="font-mono text-xs text-whisper">
          <span>{{ appVersion.version }}</span>
          <span class="mx-1">&middot;</span>
          <span>{{ appVersion.build }}</span>
        </div>
        <p class="mt-1 text-sm text-quiet">
          &copy; {{ year }} projectlighthouse. all rights reserved.
        </p>
      </div>
    </footer>
  </div>
</template>
